use crate::ast::Ast;
use crate::errors::CompileError;
use crate::node::{Node, NodeKind};

impl<'a> Ast<'a> {
    // TODO: case文, default文の実装
    fn labeled_stmt(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        if let Some((name, token)) = self.consume_ident() {
            let span = token.span;
            if self.consume_punctuator(":").is_some() {
                let expr = self.stmt()?.ok_or_else(|| CompileError::InvalidStatement {
                    msg: "ラベルの後に文がありません".to_string(),
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

    // compound_stmt ::= "{" declaration* stmt* "}"
    pub(super) fn compound_stmt(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        if let Some(token) = self.consume_punctuator("{") {
            let span = token.span;
            self.get_current_func()?.enter_scope();
            let mut body = Vec::new();
            while self.consume_punctuator("}").is_none() {
                if let Some(declarations) = self.declaration()? {
                    for declaration in declarations {
                        self.get_current_func()?.register_local_var(
                            declaration.name,
                            declaration.ty,
                            declaration.init,
                        )?;
                    }
                    continue;
                } else if let Some(stmt) = self.stmt()? {
                    body.push(*stmt);
                } else {
                    return Err(CompileError::InvalidStatement {
                        msg: "ブロック内で無効な文が見つかりました".to_string(),
                    });
                }
            }
            self.get_current_func()?.leave_scope();
            return Ok(Some(Box::new(Node::new(NodeKind::Block { body }, span))));
        }
        Ok(None)
    }

    // TODO: switch文の実装
    // selection_stmt ::= "if" "(" expr ")" stmt ("else" stmt)?
    fn selection_stmt(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        if let Some(token) = self.consume_keyword("if") {
            let span = token.span;
            self.expect_punctuator("(")?;
            let cond = self.expr()?.ok_or_else(|| CompileError::InvalidStatement {
                msg: "if文の条件式がありません".to_string(),
            })?;
            self.expect_punctuator(")")?;
            let then = self.stmt()?.ok_or_else(|| CompileError::InvalidStatement {
                msg: "if文のthen文がありません".to_string(),
            })?;
            let els = if self.consume_keyword("else").is_some() {
                self.stmt()?
            } else {
                None
            };
            return Ok(Some(Box::new(Node::new(
                NodeKind::If { cond, then, els },
                span,
            ))));
        }
        Ok(None)
    }

    // iteration_stmt ::= "while" "(" expr ")" stmt
    //                    | "do" stmt "while" "(" expr ")" ";"
    //                    | "for" "(" expr? ";" expr? ";" expr? ")" stmt
    fn iteration_stmt(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        if let Some(token) = self.consume_keyword("while") {
            let span = token.span;
            self.expect_punctuator("(")?;
            let cond = self.expr()?.ok_or_else(|| CompileError::InvalidStatement {
                msg: "while文の条件式がありません".to_string(),
            })?;
            self.expect_punctuator(")")?;
            let then = self.stmt()?.ok_or_else(|| CompileError::InvalidStatement {
                msg: "while文のthen文がありません".to_string(),
            })?;
            return Ok(Some(Box::new(Node::new(
                NodeKind::While { cond, then },
                span,
            ))));
        }

        if let Some(token) = self.consume_keyword("do") {
            let span = token.span;
            let then = self.stmt()?.ok_or_else(|| CompileError::InvalidStatement {
                msg: "do-while文のthen文がありません".to_string(),
            })?;
            self.expect_keyword("while")?;
            self.expect_punctuator("(")?;
            let cond = self.expr()?.ok_or_else(|| CompileError::InvalidStatement {
                msg: "do-while文の条件式がありません".to_string(),
            })?;
            self.expect_punctuator(")")?;
            self.expect_punctuator(";")?;
            return Ok(Some(Box::new(Node::new(NodeKind::Do { then, cond }, span))));
        }

        if let Some(token) = self.consume_keyword("for") {
            let span = token.span;
            self.expect_punctuator("(")?;
            // 初期化式
            let init = if self.consume_punctuator(";").is_none() {
                let expr = self.expr()?;
                self.expect_punctuator(";")?;
                expr
            } else {
                None
            };
            // 条件式
            let cond = if self.consume_punctuator(";").is_none() {
                let expr = self.expr()?;
                self.expect_punctuator(";")?;
                expr
            } else {
                None
            };
            // 更新式
            let inc = if self.consume_punctuator(")").is_none() {
                let expr = self.expr()?;
                self.expect_punctuator(")")?;
                expr
            } else {
                None
            };
            let then = self.stmt()?.ok_or_else(|| CompileError::InvalidStatement {
                msg: "for文のthen文がありません".to_string(),
            })?;
            return Ok(Some(Box::new(Node::new(
                NodeKind::For {
                    init,
                    cond,
                    inc,
                    then,
                },
                span,
            ))));
        }
        Ok(None)
    }

    // jump_stmt ::= "goto" ident ";"
    //               | "continue" ";"
    //               | "break" ";"
    //               | "return" expr? ";"
    fn jump_stmt(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        if let Some(token) = self.consume_keyword("goto") {
            let span = token.span;
            let (name, _) = self.consume_ident().ok_or(CompileError::InvalidStatement {
                msg: "goto文の後にラベル名が必要です".to_string(),
            })?;
            self.expect_punctuator(";")?;
            return Ok(Some(Box::new(Node::new(NodeKind::Goto { name }, span))));
        }

        if let Some(token) = self.consume_keyword("continue") {
            let span = token.span;
            self.expect_punctuator(";")?;
            return Ok(Some(Box::new(Node::new(NodeKind::Continue, span))));
        }

        if let Some(token) = self.consume_keyword("break") {
            let span = token.span;
            self.expect_punctuator(";")?;
            return Ok(Some(Box::new(Node::new(NodeKind::Break, span))));
        }

        if let Some(token) = self.consume_keyword("return") {
            let span = token.span;
            if self.consume_punctuator(";").is_some() {
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
            self.expect_punctuator(";")?;
            return Ok(Some(Box::new(Node::new(
                NodeKind::Return { expr: node },
                span,
            ))));
        }
        Ok(None)
    }

    // stmt ::= labeled_stmt
    //          | expr_stmt
    //          | compound_stmt
    //          | selection_stmt
    //          | iteration_stmt
    //          | jump_stmt
    fn stmt(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        // labeled statement
        if let Some(node) = self.labeled_stmt()? {
            return Ok(Some(node));
        }
        // selection statement
        if let Some(node) = self.selection_stmt()? {
            return Ok(Some(node));
        }
        // iteration statement
        if let Some(node) = self.iteration_stmt()? {
            return Ok(Some(node));
        }
        // compound statement
        if let Some(node) = self.compound_stmt()? {
            return Ok(Some(node));
        }
        // jump statement
        if let Some(node) = self.jump_stmt()? {
            return Ok(Some(node));
        }
        self.expr_stmt()
    }

    // expr_stmt ::= expr? ";"
    fn expr_stmt(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        if let Some(token) = self.consume_punctuator(";") {
            let span = token.span;
            Ok(Some(Box::new(Node::new(NodeKind::Nop, span))))
        } else {
            let expr_node = self.expr()?;
            self.expect_punctuator(";")?;
            Ok(expr_node)
        }
    }
}
