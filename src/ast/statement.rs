use crate::ast::Ast;
use crate::node::{Node, NodeKind};
use crate::token::Token;

impl Ast {
    // TODO: case文, default文の実装
    fn labeled_stmt(&mut self) -> Option<Box<Node>> {
        if let Some(label_name) = self.consume_ident() {
            if self.consume_punctuator(":") {
                let mut node = Node::new_unary(NodeKind::Label, self.stmt());
                node.label_name = label_name;
                return Some(Box::new(node));
            } else {
                // ラベル名ではなかった場合、トークンを元に戻す
                self.tokens.insert(0, Token::Identifier(label_name));
            }
        }
        None
    }

    // compound_stmt ::= "{" declaration* stmt* "}"
    pub(super) fn compound_stmt(&mut self) -> Option<Box<Node>> {
        if self.consume_punctuator("{") {
            let mut node = Node::from(NodeKind::Block);
            while !self.consume_punctuator("}") {
                if let Some(Ok(var)) = self.declaration() {
                    self.current_func.as_mut().unwrap().gen_lvar(var).unwrap();
                    continue;
                } else if let Some(stmt) = self.stmt() {
                    node.body.push(stmt);
                } else {
                    panic!("ブロック内の文のパースに失敗しました");
                }
            }
            return Some(Box::new(node));
        }
        None
    }

    // TODO: switch文の実装
    // selection_stmt ::= "if" "(" expr ")" stmt ("else" stmt)?
    fn selection_stmt(&mut self) -> Option<Box<Node>> {
        if self.consume_keyword("if") {
            let mut node = Node::from(NodeKind::If);
            self.expect_punctuator("(").unwrap();
            node.cond = self.expr();
            self.expect_punctuator(")").unwrap();
            node.then = self.stmt();
            if self.consume_keyword("else") {
                node.els = self.stmt();
            }
            return Some(Box::new(node));
        }
        None
    }

    // iteration_stmt ::= "while" "(" expr ")" stmt
    //                    | "do" stmt "while" "(" expr ")" ";"
    //                    | "for" "(" expr? ";" expr? ";" expr? ")" stmt
    fn iteration_stmt(&mut self) -> Option<Box<Node>> {
        if self.consume_keyword("while") {
            let mut node = Node::from(NodeKind::While);
            self.expect_punctuator("(").unwrap();
            node.cond = self.expr();
            self.expect_punctuator(")").unwrap();
            node.then = self.stmt();
            return Some(Box::new(node));
        }

        if self.consume_keyword("do") {
            let mut node = Node::from(NodeKind::Do);
            node.then = self.stmt();
            self.expect_reserved("while").unwrap();
            self.expect_punctuator("(").unwrap();
            node.cond = self.expr();
            self.expect_punctuator(")").unwrap();
            self.expect_punctuator(";").unwrap();
            return Some(Box::new(node));
        }

        if self.consume_keyword("for") {
            let mut node = Node::from(NodeKind::For);
            self.expect_punctuator("(").unwrap();
            // 初期化式
            if !self.consume_punctuator(";") {
                node.init = self.expr();
                self.expect_punctuator(";").unwrap();
            }
            // 条件式
            if !self.consume_punctuator(";") {
                node.cond = self.expr();
                self.expect_punctuator(";").unwrap();
            }
            // 更新式
            if !self.consume_punctuator(")") {
                node.inc = self.expr();
                self.expect_punctuator(")").unwrap();
            }
            node.then = self.stmt();
            return Some(Box::new(node));
        }
        None
    }

    // jump_stmt ::= "goto" ident ";"
    //               | "continue" ";"
    //               | "break" ";"
    //               | "return" expr? ";"
    fn jump_stmt(&mut self) -> Option<Box<Node>> {
        if self.consume_keyword("goto") {
            let mut node = Node::from(NodeKind::Goto);
            node.label_name = self.consume_ident().unwrap();
            self.expect_punctuator(";").unwrap();
            return Some(Box::new(node));
        }

        if self.consume_keyword("continue") {
            let node = Node::from(NodeKind::Continue);
            self.expect_punctuator(";").unwrap();
            return Some(Box::new(node));
        }

        if self.consume_keyword("break") {
            let node = Node::from(NodeKind::Break);
            self.expect_punctuator(";").unwrap();
            return Some(Box::new(node));
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
            return Some(Box::new(Node::from(NodeKind::Nop)));
        } else {
            let expr_node = self.expr();
            self.expect_punctuator(";").unwrap();
            return expr_node;
        };
    }
}
