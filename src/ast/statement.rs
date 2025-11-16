use crate::ast::Ast;
use crate::node::{Node, NodeKind};

impl Ast {
    // TODO: case文, default文の実装
    fn labeled_stmt(&mut self) -> Option<Box<Node>> {
        if let Some(name) = self.consume_ident() {
            if self.consume_punctuator(":") {
                return Some(Box::new(Node::new_unary(
                    NodeKind::Label { name },
                    self.stmt(),
                )));
            } else {
                // ラベル名ではなかった場合、トークンを元に戻す
                self.retreat_token();
            }
        }
        None
    }

    // compound_stmt ::= "{" declaration* stmt* "}"
    pub(super) fn compound_stmt(&mut self) -> Option<Box<Node>> {
        if self.consume_punctuator("{") {
            let mut body = Vec::new();
            while !self.consume_punctuator("}") {
                if let Some(vars) = self.declaration() {
                    for var in vars {
                        self.current_func.as_mut().unwrap().gen_lvar(var).unwrap();
                    }
                    continue;
                } else if let Some(stmt) = self.stmt() {
                    body.push(stmt);
                } else {
                    panic!("ブロック内の文のパースに失敗しました");
                }
            }
            return Some(Box::new(Node::from(NodeKind::Block { body })));
        }
        None
    }

    // TODO: switch文の実装
    // selection_stmt ::= "if" "(" expr ")" stmt ("else" stmt)?
    fn selection_stmt(&mut self) -> Option<Box<Node>> {
        if self.consume_keyword("if") {
            self.expect_punctuator("(").unwrap();
            let cond = self.expr();
            self.expect_punctuator(")").unwrap();
            let then = self.stmt();
            let els = if self.consume_keyword("else") {
                self.stmt()
            } else {
                None
            };
            return Some(Box::new(Node::from(NodeKind::If { cond, then, els })));
        }
        None
    }

    // iteration_stmt ::= "while" "(" expr ")" stmt
    //                    | "do" stmt "while" "(" expr ")" ";"
    //                    | "for" "(" expr? ";" expr? ";" expr? ")" stmt
    fn iteration_stmt(&mut self) -> Option<Box<Node>> {
        if self.consume_keyword("while") {
            self.expect_punctuator("(").unwrap();
            let cond = self.expr();
            self.expect_punctuator(")").unwrap();
            let then = self.stmt();
            return Some(Box::new(Node::from(NodeKind::While { cond, then })));
        }

        if self.consume_keyword("do") {
            let then = self.stmt();
            self.expect_reserved("while").unwrap();
            self.expect_punctuator("(").unwrap();
            let cond = self.expr();
            self.expect_punctuator(")").unwrap();
            self.expect_punctuator(";").unwrap();
            return Some(Box::new(Node::from(NodeKind::Do { then, cond })));
        }

        if self.consume_keyword("for") {
            self.expect_punctuator("(").unwrap();
            // 初期化式
            let init = if !self.consume_punctuator(";") {
                let expr = self.expr();
                self.expect_punctuator(";").unwrap();
                expr
            } else {
                None
            };
            // 条件式
            let cond = if !self.consume_punctuator(";") {
                let expr = self.expr();
                self.expect_punctuator(";").unwrap();
                expr
            } else {
                None
            };
            // 更新式
            let inc = if !self.consume_punctuator(")") {
                let expr = self.expr();
                self.expect_punctuator(")").unwrap();
                expr
            } else {
                None
            };
            let then = self.stmt();
            return Some(Box::new(Node::from(NodeKind::For {
                init,
                cond,
                inc,
                then,
            })));
        }
        None
    }

    // jump_stmt ::= "goto" ident ";"
    //               | "continue" ";"
    //               | "break" ";"
    //               | "return" expr? ";"
    fn jump_stmt(&mut self) -> Option<Box<Node>> {
        if self.consume_keyword("goto") {
            let name = self.consume_ident().unwrap();
            self.expect_punctuator(";").unwrap();
            return Some(Box::new(Node::from(NodeKind::Goto { name })));
        }

        if self.consume_keyword("continue") {
            self.expect_punctuator(";").unwrap();
            return Some(Box::new(Node::from(NodeKind::Continue)));
        }

        if self.consume_keyword("break") {
            self.expect_punctuator(";").unwrap();
            return Some(Box::new(Node::from(NodeKind::Break)));
        }

        if self.consume_keyword("return") {
            if self.consume_punctuator(";") {
                return Some(Box::new(Node::from(NodeKind::Return)));
            }

            let node = Node::new_unary(NodeKind::Return, self.expr());
            self.expect_punctuator(";").unwrap();
            return Some(Box::new(node));
        }
        None
    }

    // stmt ::= labeled_stmt
    //          | expr_stmt
    //          | compound_stmt
    //          | selection_stmt
    //          | iteration_stmt
    //          | jump_stmt
    fn stmt(&mut self) -> Option<Box<Node>> {
        // labeled statement
        if let Some(node) = self.labeled_stmt() {
            return Some(node);
        }

        // selection statement
        if let Some(node) = self.selection_stmt() {
            return Some(node);
        }

        // iteration statement
        if let Some(node) = self.iteration_stmt() {
            return Some(node);
        }

        // compound statement
        if let Some(node) = self.compound_stmt() {
            return Some(node);
        }

        // jump statement
        if let Some(node) = self.jump_stmt() {
            return Some(node);
        }

        self.expr_stmt()
    }

    // expr_stmt ::= expr? ";"
    fn expr_stmt(&mut self) -> Option<Box<Node>> {
        if self.consume_punctuator(";") {
            Some(Box::new(Node::from(NodeKind::Nop)))
        } else {
            let expr_node = self.expr();
            self.expect_punctuator(";").unwrap();
            expr_node
        }
    }
}
