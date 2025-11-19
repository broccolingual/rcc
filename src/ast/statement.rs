use core::panic;
use std::ops::Deref;

use crate::ast::{Ast, AstError};
use crate::node::{Node, NodeKind};
use crate::types::Type;

impl Ast {
    // TODO: case文, default文の実装
    fn labeled_stmt(&mut self) -> Result<Option<Box<Node>>, AstError> {
        if let Some(name) = self.consume_ident() {
            if let Some(_) = self.consume_punctuator(":") {
                return Ok(Some(Box::new(Node::new_unary(
                    NodeKind::Label { name },
                    self.stmt()?,
                ))));
            } else {
                // ラベル名ではなかった場合、トークンを元に戻す
                self.retreat_token();
            }
        }
        Ok(None)
    }

    // compound_stmt ::= "{" declaration* stmt* "}"
    pub(super) fn compound_stmt(&mut self) -> Result<Option<Box<Node>>, AstError> {
        if let Some(_) = self.consume_punctuator("{") {
            let mut body = Vec::new();
            while self.consume_punctuator("}").is_none() {
                if let Some(vars) = self.declaration()? {
                    for var in vars {
                        self.get_current_func()?.gen_lvar(var)?;
                    }
                    continue;
                } else if let Some(stmt) = self.stmt()? {
                    body.push(stmt);
                } else {
                    panic!("ブロック内の文のパースに失敗しました");
                }
            }
            return Ok(Some(Box::new(Node::from(NodeKind::Block { body }))));
        }
        Ok(None)
    }

    // TODO: switch文の実装
    // selection_stmt ::= "if" "(" expr ")" stmt ("else" stmt)?
    fn selection_stmt(&mut self) -> Result<Option<Box<Node>>, AstError> {
        if let Some(_) = self.consume_keyword("if") {
            self.expect_punctuator("(")?;
            let cond = self.expr()?;
            self.expect_punctuator(")")?;
            let then = self.stmt()?;
            let els = if let Some(_) = self.consume_keyword("else") {
                self.stmt()?
            } else {
                None
            };
            return Ok(Some(Box::new(Node::from(NodeKind::If { cond, then, els }))));
        }
        Ok(None)
    }

    // iteration_stmt ::= "while" "(" expr ")" stmt
    //                    | "do" stmt "while" "(" expr ")" ";"
    //                    | "for" "(" expr? ";" expr? ";" expr? ")" stmt
    fn iteration_stmt(&mut self) -> Result<Option<Box<Node>>, AstError> {
        if let Some(_) = self.consume_keyword("while") {
            self.expect_punctuator("(")?;
            let cond = self.expr()?;
            self.expect_punctuator(")")?;
            let then = self.stmt()?;
            return Ok(Some(Box::new(Node::from(NodeKind::While { cond, then }))));
        }

        if let Some(_) = self.consume_keyword("do") {
            let then = self.stmt()?;
            self.expect_reserved("while")?;
            self.expect_punctuator("(")?;
            let cond = self.expr()?;
            self.expect_punctuator(")")?;
            self.expect_punctuator(";")?;
            return Ok(Some(Box::new(Node::from(NodeKind::Do { then, cond }))));
        }

        if let Some(_) = self.consume_keyword("for") {
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
            let then = self.stmt()?;
            return Ok(Some(Box::new(Node::from(NodeKind::For {
                init,
                cond,
                inc,
                then,
            }))));
        }
        Ok(None)
    }

    // jump_stmt ::= "goto" ident ";"
    //               | "continue" ";"
    //               | "break" ";"
    //               | "return" expr? ";"
    fn jump_stmt(&mut self) -> Result<Option<Box<Node>>, AstError> {
        if let Some(_) = self.consume_keyword("goto") {
            let name = self.consume_ident().ok_or(AstError::ParseError(
                "goto文の後に識別子が必要です".to_string(),
            ))?;
            self.expect_punctuator(";")?;
            return Ok(Some(Box::new(Node::from(NodeKind::Goto { name }))));
        }

        if let Some(_) = self.consume_keyword("continue") {
            self.expect_punctuator(";")?;
            return Ok(Some(Box::new(Node::from(NodeKind::Continue))));
        }

        if let Some(_) = self.consume_keyword("break") {
            self.expect_punctuator(";")?;
            return Ok(Some(Box::new(Node::from(NodeKind::Break))));
        }

        if let Some(_) = self.consume_keyword("return") {
            if let Some(_) = self.consume_punctuator(";") {
                if Type::Void != self.get_current_func()?.return_ty {
                    panic!("return文は値を返す必要があります");
                }
                return Ok(Some(Box::new(Node::from(NodeKind::Return))));
            }
            let mut node = self.expr()?;
            if let Some(n) = &mut node {
                n.assign_types()?;
                if let Some(ret_ty) = &n.ty {
                    let func_ret_ty = &self.get_current_func()?.return_ty;
                    if ret_ty.deref() != func_ret_ty {
                        panic!(
                            "関数の戻り値の型とreturn文の型が一致しません: 関数の戻り値の型は{:?}ですが、return文の型は{:?}です",
                            func_ret_ty, ret_ty
                        );
                    }
                }
            }
            self.expect_punctuator(";")?;
            return Ok(Some(Box::new(Node::new_unary(NodeKind::Return, node))));
        }
        Ok(None)
    }

    // stmt ::= labeled_stmt
    //          | expr_stmt
    //          | compound_stmt
    //          | selection_stmt
    //          | iteration_stmt
    //          | jump_stmt
    fn stmt(&mut self) -> Result<Option<Box<Node>>, AstError> {
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
    fn expr_stmt(&mut self) -> Result<Option<Box<Node>>, AstError> {
        if let Some(_) = self.consume_punctuator(";") {
            Ok(Some(Box::new(Node::from(NodeKind::Nop))))
        } else {
            let expr_node = self.expr()?;
            self.expect_punctuator(";")?;
            Ok(expr_node)
        }
    }
}
