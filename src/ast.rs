use std::fmt;

use crate::parser::{Token, TokenKind};

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum NodeKind {
    Add,    // +
    Sub,    // -
    Mul,    // *
    Div,    // /
    Rem,    // %
    Shl,    // <<
    Shr,    // >>
    Eq,     // ==
    Ne,     // !=
    Lt,     // <
    Le,     // <=
    Assign, // =
    If,     // if
    While,  // while
    For,    // for
    Do,     // do
    Block,  // {}
    LVar,   // ローカル変数
    Return, // return
    Num,    // 整数
}

pub struct Node {
    pub kind: NodeKind,
    pub next: Option<Box<Node>>, // 次のノードへのポインタ
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
    pub val: i64,                // kindがNumのときに使う
    pub offset: i64,             // kindがLVarのときに使う
    pub cond: Option<Box<Node>>, // if, while文の条件式
    pub then: Option<Box<Node>>, // if, while文のthen節
    pub els: Option<Box<Node>>,  // if文のelse節
    pub init: Option<Box<Node>>, // for文の初期化式
    pub inc: Option<Box<Node>>,  // for文の更新式
    pub body: Option<Box<Node>>, // ブロック内の文
}

impl Node {
    pub fn new(kind: NodeKind, lhs: Option<Box<Node>>, rhs: Option<Box<Node>>) -> Self {
        Node {
            kind,
            next: None,
            lhs,
            rhs,
            val: 0,
            offset: 0,
            cond: None,
            then: None,
            els: None,
            init: None,
            inc: None,
            body: None,
        }
    }

    pub fn new_num(val: i64) -> Self {
        Node {
            kind: NodeKind::Num,
            next: None,
            lhs: None,
            rhs: None,
            val,
            offset: 0,
            cond: None,
            then: None,
            els: None,
            init: None,
            inc: None,
            body: None,
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

    // stmt ::= "return" expr ";"
    //          | "if" "(" expr ")" stmt ("else" stmt)?
    //          | "while" "(" expr ")" stmt
    //          | "for" "(" expr? ";" expr? ";" expr? ")" stmt
    //          | "do" stmt "while" "(" expr ")" ";"
    //          | "{" stmt* "}"
    //          | expr_stmt
    fn stmt(&mut self) -> Option<Box<Node>> {
        let mut node: Option<Box<Node>>;

        if self.consume("return") {
            node = Some(Box::new(Node::new(NodeKind::Return, self.expr(), None)));
            self.expect(";").unwrap();
            return node;
        }

        // selection statement
        if self.consume("if") {
            node = Some(Box::new(Node::new(NodeKind::If, None, None)));
            self.expect("(").unwrap();
            node.as_mut().unwrap().cond = self.expr();
            self.expect(")").unwrap();
            node.as_mut().unwrap().then = self.stmt();
            if self.consume("else") {
                node.as_mut().unwrap().els = self.stmt();
            }
            return node;
        }

        if self.consume("while") {
            node = Some(Box::new(Node::new(NodeKind::While, None, None)));
            self.expect("(").unwrap();
            node.as_mut().unwrap().cond = self.expr();
            self.expect(")").unwrap();
            node.as_mut().unwrap().then = self.stmt();
            return node;
        }

        // iteration statement
        if self.consume("for") {
            node = Some(Box::new(Node::new(NodeKind::For, None, None)));
            self.expect("(").unwrap();
            // 初期化式
            if !self.consume(";") {
                node.as_mut().unwrap().init = self.expr();
                self.expect(";").unwrap();
            }
            // 条件式
            if !self.consume(";") {
                node.as_mut().unwrap().cond = self.expr();
                self.expect(";").unwrap();
            }
            // 更新式
            if !self.consume(")") {
                node.as_mut().unwrap().inc = self.expr();
                self.expect(")").unwrap();
            }
            node.as_mut().unwrap().then = self.stmt();
            return node;
        }

        if self.consume("do") {
            node = Some(Box::new(Node::new(NodeKind::Do, None, None)));
            node.as_mut().unwrap().then = self.stmt();
            self.expect("while").unwrap();
            self.expect("(").unwrap();
            node.as_mut().unwrap().cond = self.expr();
            self.expect(")").unwrap();
            self.expect(";").unwrap();
            return node;
        }

        // compound statement
        if self.consume("{") {
            // 一時的に頭のダミーノードを作成
            let mut head: Option<Box<Node>> =
                Some(Box::new(Node::new(NodeKind::Block, None, None)));
            // 現在のノードを指すポインタ
            let mut cur: &mut Option<Box<Node>> = &mut head;
            while !self.consume("}") {
                cur.as_mut().unwrap().next = self.stmt();
                cur = &mut cur.as_mut().unwrap().next;
            }
            // ブロックノードを作成
            let mut node = Some(Box::new(Node::new(NodeKind::Block, None, None)));
            // ブロック内の文を設定
            node.as_mut().unwrap().body = head.as_mut().unwrap().next.take();
            return node;
        }

        self.expr_stmt()
    }

    // expr_stmt ::= expr ";"
    fn expr_stmt(&mut self) -> Option<Box<Node>> {
        let node = self.expr();
        self.expect(";").unwrap();
        node
    }

    // expr ::= assign_expr
    fn expr(&mut self) -> Option<Box<Node>> {
        self.assign_expr()
    }

    // assign_expr ::= conditional_expr ("=" assign_expr)?
    fn assign_expr(&mut self) -> Option<Box<Node>> {
        let mut node = self.conditional_expr();

        if self.consume("=") {
            node = Some(Box::new(Node::new(
                NodeKind::Assign,
                node,
                self.assign_expr(),
            )));
        }
        node
    }

    // conditional_expr ::= logical_or_expr
    fn conditional_expr(&mut self) -> Option<Box<Node>> {
        self.logical_or_expr()
    }

    // logical_or_expr ::= logical_and_expr
    fn logical_or_expr(&mut self) -> Option<Box<Node>> {
        self.logical_and_expr()
    }

    // logical_and_expr ::= inclusive_or_expr
    fn logical_and_expr(&mut self) -> Option<Box<Node>> {
        self.inclusive_or_expr()
    }

    // inclusive_or_expr ::= exclusive_or_expr
    fn inclusive_or_expr(&mut self) -> Option<Box<Node>> {
        self.exclusive_or_expr()
    }

    // exclusive_or_expr ::= and_expr
    fn exclusive_or_expr(&mut self) -> Option<Box<Node>> {
        self.and_expr()
    }

    // and_expr ::= equality_expr
    fn and_expr(&mut self) -> Option<Box<Node>> {
        self.equality_expr()
    }

    // equality_expr ::= relational_expr (("==" | "!=") relational_expr)*
    fn equality_expr(&mut self) -> Option<Box<Node>> {
        let mut node = self.relational_expr();

        loop {
            if self.consume("==") {
                // equal
                node = Some(Box::new(Node::new(
                    NodeKind::Eq,
                    node,
                    self.relational_expr(),
                )));
            } else if self.consume("!=") {
                // not equal
                node = Some(Box::new(Node::new(
                    NodeKind::Ne,
                    node,
                    self.relational_expr(),
                )));
            } else {
                return node;
            }
        }
    }

    // relational_expr ::= shift_expr (("<" | "<=" | ">" | ">=") shift_expr)*
    fn relational_expr(&mut self) -> Option<Box<Node>> {
        let mut node = self.shift_expr();

        loop {
            if self.consume("<") {
                // less than
                node = Some(Box::new(Node::new(NodeKind::Lt, node, self.shift_expr())));
            } else if self.consume("<=") {
                // less than or equal
                node = Some(Box::new(Node::new(NodeKind::Le, node, self.shift_expr())));
            } else if self.consume(">") {
                // greater than
                node = Some(Box::new(Node::new(NodeKind::Lt, self.shift_expr(), node)));
            } else if self.consume(">=") {
                // greater than or equal
                node = Some(Box::new(Node::new(NodeKind::Le, self.shift_expr(), node)));
            } else {
                return node;
            }
        }
    }

    // shift_expr ::= add_expr (("<<" | ">>") add_expr)*
    fn shift_expr(&mut self) -> Option<Box<Node>> {
        let mut node = self.add_expr();

        loop {
            if self.consume("<<") {
                // left shift
                node = Some(Box::new(Node::new(NodeKind::Shl, node, self.add_expr())));
            } else if self.consume(">>") {
                // right shift
                node = Some(Box::new(Node::new(NodeKind::Shr, node, self.add_expr())));
            } else {
                return node;
            }
        }
    }

    // add_expr ::= mul_expr (("+" | "-") mul_expr)*
    fn add_expr(&mut self) -> Option<Box<Node>> {
        let mut node = self.mul_expr();

        loop {
            if self.consume("+") {
                // addition
                node = Some(Box::new(Node::new(NodeKind::Add, node, self.mul_expr())));
            } else if self.consume("-") {
                // subtraction
                node = Some(Box::new(Node::new(NodeKind::Sub, node, self.mul_expr())));
            } else {
                return node;
            }
        }
    }

    // mul_expr ::= cast_expr (("*" | "/" | "%") cast_expr)*
    fn mul_expr(&mut self) -> Option<Box<Node>> {
        let mut node = self.cast_expr();

        loop {
            if self.consume("*") {
                // multiplication
                node = Some(Box::new(Node::new(NodeKind::Mul, node, self.cast_expr())));
            } else if self.consume("/") {
                // division
                node = Some(Box::new(Node::new(NodeKind::Div, node, self.cast_expr())));
            } else if self.consume("%") {
                // remainder
                node = Some(Box::new(Node::new(NodeKind::Rem, node, self.cast_expr())));
            } else {
                return node;
            }
        }
    }

    // cast_expr ::= unary_expr
    fn cast_expr(&mut self) -> Option<Box<Node>> {
        self.unary_expr()
    }

    // unary_expr ::= ("+" | "-") cast_expr
    //                | postfix_expr
    fn unary_expr(&mut self) -> Option<Box<Node>> {
        if self.consume("+") {
            // unary plus
            return self.cast_expr();
        }
        if self.consume("-") {
            // unary minus
            return Some(Box::new(Node::new(
                NodeKind::Sub,
                Some(Box::new(Node::new_num(0))),
                self.cast_expr(),
            )));
        }
        self.postfix_expr()
    }

    // postfix_expr ::= primary_expr
    fn postfix_expr(&mut self) -> Option<Box<Node>> {
        self.primary_expr()
    }

    // primary_expr ::= "(" expr ")"
    //                  | ident
    //                  | num
    fn primary_expr(&mut self) -> Option<Box<Node>> {
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
}
