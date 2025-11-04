use std::fmt;

use crate::parser::{Token, TokenKind};

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum NodeKind {
    Add,    // +
    Sub,    // -
    Mul,    // *
    Div,    // /
    Rem,    // %
    Eq,     // ==
    Ne,     // !=
    Lt,     // <
    Le,     // <=
    Assign, // =
    LVar,   // ローカル変数
    Return, // return
    Num,    // 整数
}

pub struct Node {
    pub kind: NodeKind,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
    pub val: i64,    // kindがNumのときに使う
    pub offset: i64, // kindがLVarのときに使う
}

impl Node {
    pub fn new(kind: NodeKind, lhs: Option<Box<Node>>, rhs: Option<Box<Node>>) -> Self {
        Node {
            kind,
            lhs,
            rhs,
            val: 0,
            offset: 0,
        }
    }

    pub fn new_num(val: i64) -> Self {
        Node {
            kind: NodeKind::Num,
            lhs: None,
            rhs: None,
            val,
            offset: 0,
        }
    }
}

#[derive(Clone)]
pub struct LVar {
    next: Option<Box<LVar>>,
    name: String,
    offset: i64,
}

impl fmt::Debug for LVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut current = Some(self);
        while let Some(var) = current {
            writeln!(f, "LVar {{ name: {}, offset: {} }}", var.name, var.offset)?;
            current = var.next.as_ref().map(|b| b.as_ref());
        }
        Ok(())
    }
}

pub struct Ast {
    pub tokens: Vec<Token>,
    pub code: Vec<Option<Box<Node>>>,
    pub locals: Option<Box<LVar>>,
}

impl Ast {
    pub fn new(tokens: Vec<Token>) -> Self {
        Ast {
            tokens,
            code: Vec::new(),
            locals: None,
        }
    }

    fn find_lvar(&mut self, name: &str) -> Option<&mut Box<LVar>> {
        let mut lvar = self.locals.as_mut();
        while let Some(var) = lvar {
            if var.name == name {
                return Some(var);
            }
            lvar = var.next.as_mut();
        }
        None
    }

    fn consume(&mut self, op: &str) -> bool {
        let current_token = self.tokens.first();
        if current_token.is_none()
            || current_token.unwrap().kind != TokenKind::Reserved
            || current_token.unwrap().input != op
        {
            return false;
        }
        self.tokens.remove(0);
        true
    }

    fn consume_ident(&mut self) -> Option<String> {
        let current_token = self.tokens.first();
        if current_token.is_none() || current_token.unwrap().kind != TokenKind::Ident {
            return None;
        }
        let name = current_token.unwrap().input.clone();
        self.tokens.remove(0);
        Some(name)
    }

    fn expect(&mut self, op: &str) -> Result<(), &str> {
        let current_token = self.tokens.first();
        if current_token.is_none()
            || current_token.unwrap().kind != TokenKind::Reserved
            || current_token.unwrap().input != op
        {
            return Err("予期せぬトークンです");
        }
        self.tokens.remove(0);
        Ok(())
    }

    fn expect_number(&mut self) -> Result<i64, &str> {
        let current_token = self.tokens.first();
        if current_token.is_none() || current_token.unwrap().kind != TokenKind::Num {
            return Err("数値ではありません");
        }
        let val = current_token.unwrap().val;
        self.tokens.remove(0);
        Ok(val)
    }

    fn at_eof(&self) -> bool {
        self.tokens.is_empty() || self.tokens.first().unwrap().kind == TokenKind::EOF
    }

    pub fn program(&mut self) {
        while !self.at_eof() {
            let node = self.stmt();
            self.code.push(node);
        }
    }

    fn expr(&mut self) -> Option<Box<Node>> {
        self.assign()
    }

    fn stmt(&mut self) -> Option<Box<Node>> {
        let node: Option<Box<Node>>;

        if self.consume("return") {
            node = Some(Box::new(Node::new(NodeKind::Return, self.expr(), None)));
        } else {
            node = self.expr();
        }
        self.expect(";").unwrap();
        node
    }

    fn assign(&mut self) -> Option<Box<Node>> {
        let mut node = self.equality();

        if self.consume("=") {
            node = Some(Box::new(Node::new(NodeKind::Assign, node, self.assign())));
        }
        node
    }

    fn equality(&mut self) -> Option<Box<Node>> {
        let mut node = self.relational();

        loop {
            if self.consume("==") {
                node = Some(Box::new(Node::new(NodeKind::Eq, node, self.relational())));
            } else if self.consume("!=") {
                node = Some(Box::new(Node::new(NodeKind::Ne, node, self.relational())));
            } else {
                return node;
            }
        }
    }

    fn relational(&mut self) -> Option<Box<Node>> {
        let mut node = self.add();

        loop {
            if self.consume("<") {
                node = Some(Box::new(Node::new(NodeKind::Lt, node, self.add())));
            } else if self.consume("<=") {
                node = Some(Box::new(Node::new(NodeKind::Le, node, self.add())));
            } else if self.consume(">") {
                node = Some(Box::new(Node::new(NodeKind::Lt, self.add(), node)));
            } else if self.consume(">=") {
                node = Some(Box::new(Node::new(NodeKind::Le, self.add(), node)));
            } else {
                return node;
            }
        }
    }

    fn add(&mut self) -> Option<Box<Node>> {
        let mut node = self.mul();

        loop {
            if self.consume("+") {
                node = Some(Box::new(Node::new(NodeKind::Add, node, self.mul())));
            } else if self.consume("-") {
                node = Some(Box::new(Node::new(NodeKind::Sub, node, self.mul())));
            } else {
                return node;
            }
        }
    }

    fn mul(&mut self) -> Option<Box<Node>> {
        let mut node = self.unary();

        loop {
            if self.consume("*") {
                node = Some(Box::new(Node::new(NodeKind::Mul, node, self.unary())));
            } else if self.consume("/") {
                node = Some(Box::new(Node::new(NodeKind::Div, node, self.unary())));
            } else if self.consume("%") {
                node = Some(Box::new(Node::new(NodeKind::Rem, node, self.unary())));
            } else {
                return node;
            }
        }
    }

    fn primary(&mut self) -> Option<Box<Node>> {
        if self.consume("(") {
            let node = self.expr();
            self.expect(")").unwrap();
            return node;
        }
        let token = self.consume_ident();
        if let Some(name) = token {
            // ローカル変数ノードを作成
            let mut node = Node::new(NodeKind::LVar, None, None);
            let lvar = self.find_lvar(&name);
            if let Some(lvar) = lvar {
                node.offset = lvar.offset; // 既存のローカル変数のオフセットを設定
            } else {
                let offset = if let Some(ref locals) = self.locals {
                    locals.offset + 8
                } else {
                    8
                };
                // 新しいローカル変数を追加
                let new_lvar = LVar {
                    next: self.locals.take(),
                    name: name.clone(),
                    offset,
                };
                node.offset = new_lvar.offset;
                self.locals = Some(Box::new(new_lvar));
            }
            return Some(Box::new(node));
        }
        Some(Box::new(Node::new_num(self.expect_number().unwrap())))
    }

    fn unary(&mut self) -> Option<Box<Node>> {
        if self.consume("+") {
            return self.unary();
        }
        if self.consume("-") {
            return Some(Box::new(Node::new(
                NodeKind::Sub,
                Some(Box::new(Node::new_num(0))),
                self.unary(),
            )));
        }
        self.primary()
    }
}
