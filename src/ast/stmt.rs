use super::Ast;
use crate::errors::CompileError;
use crate::func::LocalVar;
use crate::node::{Node, NodeKind};

impl Ast<'_> {
    // labeled_stmt ::= ident ":" stmt
    //                | "case" const_expr ":" stmt // TODO: 未実装
    //                | "default" ":" stmt // TODO: 未実装
    fn labeled_stmt(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        if let Some((name, span)) = self.consume_ident() {
            if self.consume_punct(":").is_some() {
                let expr = self.stmt()?.ok_or_else(|| CompileError::InvalidStmt {
                    msg: "ラベルの後に文がありません".to_string(),
                    span,
                })?;
                return Ok(Some(Box::new(Node::new(NodeKind::Label { name, expr }, span))));
            } else {
                // ラベル名ではなかった場合、トークンを元に戻す
                self.retreat_token();
            }
        }
        Ok(None)
    }

    // compound_stmt ::= "{" decl* stmt* "}"
    pub(super) fn compound_stmt(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        if let Some(span) = self.consume_punct("{") {
            self.push_scope(); // 新しいスコープに入る
            let mut body = Vec::new();
            while self.consume_punct("}").is_none() {
                if let Some(decls) = self.decl()? {
                    for decl in decls {
                        if decl.ty.is_typedef() {
                            self.register_typedef(&decl.name, decl.ty, decl.span)?;
                        } else {
                            let symbol_id = self.register_var(&decl, self.current_func)?;
                            self.get_current_func_mut()?.locals.push(LocalVar::new(symbol_id));
                        }
                    }
                    continue;
                } else if let Some(stmt) = self.stmt()? {
                    body.push(*stmt);
                } else {
                    return Err(CompileError::InvalidStmt {
                        msg: "ブロック内で無効な文が見つかりました".to_string(),
                        span,
                    });
                }
            }
            self.pop_scope(); // スコープを抜ける
            return Ok(Some(Box::new(Node::new(NodeKind::Block { body }, span))));
        }
        Ok(None)
    }

    // selection_stmt ::= "if" "(" expr ")" stmt ("else" stmt)?
    //                  | "switch" "(" expr ")" stmt // TODO: 未実装
    fn selection_stmt(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        if let Some(span) = self.consume_keyword("if") {
            let label = self.next_label();
            self.expect_punct("(")?;
            let cond = self.expr()?.ok_or_else(|| CompileError::InvalidStmt {
                msg: "if文の条件式がありません".to_string(),
                span,
            })?;
            self.expect_punct(")")?;
            let then = self.stmt()?.ok_or_else(|| CompileError::InvalidStmt {
                msg: "if文のthen文がありません".to_string(),
                span,
            })?;
            let els = if self.consume_keyword("else").is_some() { self.stmt()? } else { None };
            return Ok(Some(Box::new(Node::new(NodeKind::If { cond, then, els, label }, span))));
        }
        Ok(None)
    }

    // iteration_stmt ::= "while" "(" expr ")" stmt
    //                  | "do" stmt "while" "(" expr ")" ";"
    //                  | "for" "(" expr? ";" expr? ";" expr? ")" stmt
    //                  | "for" "(" decl expr? ";" expr? ")" stmt // TODO: 未実装
    fn iteration_stmt(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        if let Some(span) = self.consume_keyword("while") {
            let label = self.next_label();
            let node = self.with_loop_scope(label, |this| {
                this.expect_punct("(")?;
                let cond = this.expr()?.ok_or_else(|| CompileError::InvalidStmt {
                    msg: "while文の条件式がありません".to_string(),
                    span,
                })?;
                this.expect_punct(")")?;
                let then = this.stmt()?.ok_or_else(|| CompileError::InvalidStmt {
                    msg: "while文のthen文がありません".to_string(),
                    span,
                })?;
                Ok(Box::new(Node::new(NodeKind::While { cond, then, label }, span)))
            })?;
            return Ok(Some(node));
        }

        if let Some(span) = self.consume_keyword("do") {
            let label = self.next_label();
            let node = self.with_loop_scope(label, |this| {
                let then = this.stmt()?.ok_or_else(|| CompileError::InvalidStmt {
                    msg: "do-while文のthen文がありません".to_string(),
                    span,
                })?;
                this.expect_keyword("while")?;
                this.expect_punct("(")?;
                let cond = this.expr()?.ok_or_else(|| CompileError::InvalidStmt {
                    msg: "do-while文の条件式がありません".to_string(),
                    span,
                })?;
                this.expect_punct(")")?;
                this.expect_punct(";")?;
                Ok(Box::new(Node::new(NodeKind::Do { then, cond, label }, span)))
            })?;
            return Ok(Some(node));
        }

        if let Some(span) = self.consume_keyword("for") {
            let label = self.next_label();
            let node = self.with_loop_scope(label, |this| {
                this.expect_punct("(")?;
                // 初期化式
                let init = if this.consume_punct(";").is_none() {
                    let expr = this.expr()?;
                    this.expect_punct(";")?;
                    expr
                } else {
                    None
                };
                // 条件式
                let cond = if this.consume_punct(";").is_none() {
                    let expr = this.expr()?;
                    this.expect_punct(";")?;
                    expr
                } else {
                    None
                };
                // 更新式
                let inc = if this.consume_punct(")").is_none() {
                    let expr = this.expr()?;
                    this.expect_punct(")")?;
                    expr
                } else {
                    None
                };
                let then = this.stmt()?.ok_or_else(|| CompileError::InvalidStmt {
                    msg: "for文のthen文がありません".to_string(),
                    span,
                })?;
                Ok(Box::new(Node::new(
                    NodeKind::For { init, cond, inc, then, label },
                    span,
                )))
            })?;
            return Ok(Some(node));
        }
        Ok(None)
    }

    // jump_stmt ::= "goto" ident ";"
    //             | "continue" ";"
    //             | "break" ";"
    //             | "return" expr? ";"
    fn jump_stmt(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        if let Some(span) = self.consume_keyword("goto") {
            let (name, _) = self.consume_ident().ok_or(CompileError::InvalidStmt {
                msg: "goto文の後にラベル名が必要です".to_string(),
                span,
            })?;
            self.expect_punct(";")?;
            return Ok(Some(Box::new(Node::new(NodeKind::Goto { name }, span))));
        }

        if let Some(span) = self.consume_keyword("continue") {
            let label = self.current_loop_label().ok_or(CompileError::InvalidStmt {
                msg: "continue文がループの外で使われています".to_string(),
                span,
            })?;
            self.expect_punct(";")?;
            return Ok(Some(Box::new(Node::new(NodeKind::Continue { label }, span))));
        }

        if let Some(span) = self.consume_keyword("break") {
            let label = self.current_loop_label().ok_or(CompileError::InvalidStmt {
                msg: "break文がループの外で使われています".to_string(),
                span,
            })?;
            self.expect_punct(";")?;
            return Ok(Some(Box::new(Node::new(NodeKind::Break { label }, span))));
        }

        if let Some(span) = self.consume_keyword("return") {
            if self.consume_punct(";").is_some() {
                return Ok(Some(Box::new(Node::new(NodeKind::Return { expr: None }, span))));
            }
            let node = self.expr()?;
            self.expect_punct(";")?;
            return Ok(Some(Box::new(Node::new(NodeKind::Return { expr: node }, span))));
        }
        Ok(None)
    }

    // stmt ::= labeled_stmt
    //        | compound_stmt
    //        | selection_stmt
    //        | iteration_stmt
    //        | jump_stmt
    //        | expr_stmt
    fn stmt(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        // labeled stmt
        if let Some(node) = self.labeled_stmt()? {
            return Ok(Some(node));
        }
        // selection stmt
        if let Some(node) = self.selection_stmt()? {
            return Ok(Some(node));
        }
        // iteration stmt
        if let Some(node) = self.iteration_stmt()? {
            return Ok(Some(node));
        }
        // compound stmt
        if let Some(node) = self.compound_stmt()? {
            return Ok(Some(node));
        }
        // jump stmt
        if let Some(node) = self.jump_stmt()? {
            return Ok(Some(node));
        }
        self.expr_stmt()
    }

    // expr_stmt ::= expr? ";"
    fn expr_stmt(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        if let Some(span) = self.consume_punct(";") {
            Ok(Some(Box::new(Node::new(NodeKind::Nop, span))))
        } else {
            let expr_node = self.expr()?;
            self.expect_punct(";")?;
            Ok(expr_node)
        }
    }
}
