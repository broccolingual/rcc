use crate::ast::Ast;
use crate::errors::CompileError;
use crate::function::LocalVar;
use crate::node::{Node, NodeKind};

impl Ast<'_> {
    // labeled_stmt ::= ident ":" stmt
    //                | "case" const_expr ":" stmt // TODO: 未実装
    //                | "default" ":" stmt // TODO: 未実装
    fn labeled_stmt(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        if let Some((name, token)) = self.consume_ident() {
            let span = token.span;
            if self.consume_punct(":").is_some() {
                let expr = self.stmt()?.ok_or_else(|| CompileError::InvalidStmt {
                    msg: "ラベルの後に文がありません".to_string(),
                    span,
                })?;
                return Ok(Some(Box::new(Node::new(
                    NodeKind::Label { name, expr },
                    span,
                ))));
            } else {
                // ラベル名ではなかった場合、トークンを元に戻す
                self.retreat_token();
            }
        }
        Ok(None)
    }

    // compound_stmt ::= "{" decl* stmt* "}"
    pub(super) fn compound_stmt(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        if let Some(token) = self.consume_punct("{") {
            let span = token.span;
            self.push_scope(); // 新しいスコープに入る
            let mut body = Vec::new();
            while self.consume_punct("}").is_none() {
                if let Some(decls) = self.decl()? {
                    for decl in decls {
                        let symbol_id = self.register_var(&decl, self.current_func)?;
                        self.get_current_func_mut()?
                            .locals
                            .push(LocalVar::new(symbol_id));
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
        if let Some(token) = self.consume_keyword("if") {
            let span = token.span;
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
            let els = if self.consume_keyword("else").is_some() {
                self.stmt()?
            } else {
                None
            };
            return Ok(Some(Box::new(Node::new(
                NodeKind::If {
                    cond,
                    then,
                    els,
                    label,
                },
                span,
            ))));
        }
        Ok(None)
    }

    // iteration_stmt ::= "while" "(" expr ")" stmt
    //                  | "do" stmt "while" "(" expr ")" ";"
    //                  | "for" "(" expr? ";" expr? ";" expr? ")" stmt
    //                  | "for" "(" decl expr? ";" expr? ")" stmt // TODO: 未実装
    fn iteration_stmt(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        if let Some(token) = self.consume_keyword("while") {
            let span = token.span;
            let label = self.next_label();
            self.push_loop(label);
            self.expect_punct("(")?;
            let cond = self.expr()?.ok_or_else(|| CompileError::InvalidStmt {
                msg: "while文の条件式がありません".to_string(),
                span,
            })?;
            self.expect_punct(")")?;
            let then = self.stmt()?.ok_or_else(|| CompileError::InvalidStmt {
                msg: "while文のthen文がありません".to_string(),
                span,
            })?;
            self.pop_loop()?;
            return Ok(Some(Box::new(Node::new(
                NodeKind::While { cond, then, label },
                span,
            ))));
        }

        if let Some(token) = self.consume_keyword("do") {
            let span = token.span;
            let label = self.next_label();
            self.push_loop(label);
            let then = self.stmt()?.ok_or_else(|| CompileError::InvalidStmt {
                msg: "do-while文のthen文がありません".to_string(),
                span,
            })?;
            self.expect_keyword("while")?;
            self.expect_punct("(")?;
            let cond = self.expr()?.ok_or_else(|| CompileError::InvalidStmt {
                msg: "do-while文の条件式がありません".to_string(),
                span,
            })?;
            self.expect_punct(")")?;
            self.expect_punct(";")?;
            self.pop_loop()?;
            return Ok(Some(Box::new(Node::new(
                NodeKind::Do { then, cond, label },
                span,
            ))));
        }

        if let Some(token) = self.consume_keyword("for") {
            let span = token.span;
            let label = self.next_label();
            self.push_loop(label);
            self.expect_punct("(")?;
            // 初期化式
            let init = if self.consume_punct(";").is_none() {
                let expr = self.expr()?;
                self.expect_punct(";")?;
                expr
            } else {
                None
            };
            // 条件式
            let cond = if self.consume_punct(";").is_none() {
                let expr = self.expr()?;
                self.expect_punct(";")?;
                expr
            } else {
                None
            };
            // 更新式
            let inc = if self.consume_punct(")").is_none() {
                let expr = self.expr()?;
                self.expect_punct(")")?;
                expr
            } else {
                None
            };
            let then = self.stmt()?.ok_or_else(|| CompileError::InvalidStmt {
                msg: "for文のthen文がありません".to_string(),
                span,
            })?;
            self.pop_loop()?;
            return Ok(Some(Box::new(Node::new(
                NodeKind::For {
                    init,
                    cond,
                    inc,
                    then,
                    label,
                },
                span,
            ))));
        }
        Ok(None)
    }

    // jump_stmt ::= "goto" ident ";"
    //             | "continue" ";"
    //             | "break" ";"
    //             | "return" expr? ";"
    fn jump_stmt(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        if let Some(token) = self.consume_keyword("goto") {
            let span = token.span;
            let (name, _) = self.consume_ident().ok_or(CompileError::InvalidStmt {
                msg: "goto文の後にラベル名が必要です".to_string(),
                span,
            })?;
            self.expect_punct(";")?;
            return Ok(Some(Box::new(Node::new(NodeKind::Goto { name }, span))));
        }

        if let Some(token) = self.consume_keyword("continue") {
            let span = token.span;
            let label = self.current_loop_label().ok_or(CompileError::InvalidStmt {
                msg: "continue文がループの外で使われています".to_string(),
                span,
            })?;
            self.expect_punct(";")?;
            return Ok(Some(Box::new(Node::new(
                NodeKind::Continue { label },
                span,
            ))));
        }

        if let Some(token) = self.consume_keyword("break") {
            let span = token.span;
            let label = self.current_loop_label().ok_or(CompileError::InvalidStmt {
                msg: "break文がループの外で使われています".to_string(),
                span,
            })?;
            self.expect_punct(";")?;
            return Ok(Some(Box::new(Node::new(NodeKind::Break { label }, span))));
        }

        if let Some(token) = self.consume_keyword("return") {
            let span = token.span;
            if self.consume_punct(";").is_some() {
                // TODO: プロトタイプ宣言実装まで保留
                // if TypeKind::Void != self.get_current_func()?.return_ty.kind {
                //     return Err(CompileError::InvalidReturnType {
                //         expected: self.get_current_func()?.return_ty.clone().kind,
                //         found: TypeKind::Void,
                //     });
                // }
                return Ok(Some(Box::new(Node::new(
                    NodeKind::Return { expr: None },
                    span,
                ))));
            }
            let node = self.expr()?;
            // TODO: プロトタイプ宣言実装まで保留
            // if let Some(n) = &mut node {
            //     // let func_ret_ty = &self.get_current_func()?.return_ty;
            //     // if &n.ty != func_ret_ty {
            //     //     return Err(CompileError::InvalidReturnType {
            //     //         expected: func_ret_ty.kind.clone(),
            //     //         found: n.ty.kind.clone(),
            //     //     });
            //     // }
            // }
            self.expect_punct(";")?;
            return Ok(Some(Box::new(Node::new(
                NodeKind::Return { expr: node },
                span,
            ))));
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
        if let Some(token) = self.consume_punct(";") {
            let span = token.span;
            Ok(Some(Box::new(Node::new(NodeKind::Nop, span))))
        } else {
            let expr_node = self.expr()?;
            self.expect_punct(";")?;
            Ok(expr_node)
        }
    }
}
