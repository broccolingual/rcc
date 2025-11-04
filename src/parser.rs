use std::fmt;

use crate::ast::{Node, NodeKind};

#[derive(PartialEq, Eq, Clone, Debug)]
enum TokenKind {
    Reserved, // 記号
    Num,      // 整数トークン
    EOF,      // 入力の終わりを表すトークン
}

#[derive(Clone, Debug)]
struct Token {
    kind: TokenKind,
    next: Option<Box<Token>>, // 次の入力トークン
    val: i64,                 // kindがNumの場合、その数値
    input: String,            // トークン文字列
    length: usize,            // トークンの長さ
}

impl Token {
    fn new(kind: TokenKind, input: &str) -> Self {
        Token {
            kind,
            next: None,
            val: 0,
            input: input.to_string(),
            length: input.len(),
        }
    }
}

pub struct Tokenizer {
    head: Option<Box<Token>>,
    current_token: Option<Box<Token>>,
}

impl fmt::Debug for Tokenizer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tokens = Vec::new();
        let mut current = self.head.as_ref();
        while let Some(token) = current {
            tokens.push(token.as_ref().clone());
            current = token.next.as_ref();
        }
        for token in tokens {
            writeln!(
                f,
                "<{:?}: '{}' (len: {})>",
                token.kind, token.input, token.length
            )?;
        }
        Ok(())
    }
}

impl Tokenizer {
    pub fn new() -> Self {
        // トークナイズの実装は省略
        Tokenizer {
            head: None,
            current_token: None,
        }
    }

    fn consume(&mut self, op: &str) -> bool {
        if self.current_token.is_none()
            || self.current_token.as_ref().unwrap().kind != TokenKind::Reserved
            || self.current_token.as_ref().unwrap().length != op.len()
            || self.current_token.as_ref().unwrap().input != op
        {
            return false;
        }
        self.current_token = self.current_token.as_mut().unwrap().next.take();
        true
    }

    fn expect(&mut self, op: &str) -> Result<(), &str> {
        if self.current_token.is_none()
            || self.current_token.as_ref().unwrap().kind != TokenKind::Reserved
            || self.current_token.as_ref().unwrap().length != op.len()
            || self.current_token.as_ref().unwrap().input != op
        {
            return Err("予期せぬトークンです");
        }
        self.current_token = self.current_token.as_mut().unwrap().next.take();
        Ok(())
    }

    fn expect_number(&mut self) -> Result<i64, &str> {
        if self.current_token.is_none()
            || self.current_token.as_ref().unwrap().kind != TokenKind::Num
        {
            return Err("数値ではありません");
        }
        let val = self.current_token.as_ref().unwrap().val;
        self.current_token = self.current_token.as_mut().unwrap().next.take();
        Ok(val)
    }

    #[allow(dead_code)]
    fn at_eof(&self) -> bool {
        self.current_token.is_none() || self.current_token.as_ref().unwrap().kind == TokenKind::EOF
    }

    fn append_token(&mut self, token: Token) {
        let boxed_token = Box::new(token);
        if self.head.is_none() {
            self.head = Some(boxed_token);
        } else {
            let mut tail = self.head.as_mut().unwrap();
            while tail.next.is_some() {
                tail = tail.next.as_mut().unwrap();
            }
            tail.next = Some(boxed_token);
        }
    }

    pub fn expr(&mut self) -> Option<Box<Node>> {
        self.equality()
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

    pub fn tokenize(&mut self, input: &str) {
        let mut c_iter = input.chars().peekable();

        while let Some(c) = c_iter.next() {
            // 空白文字をスキップ
            if c.is_whitespace() {
                continue;
            }

            // 2文字の記号トークン
            if let Some(&next_c) = c_iter.peek() {
                let two_char_op = format!("{}{}", c, next_c);
                if ["==", "!=", "<=", ">="].contains(&two_char_op.as_str()) {
                    let token = Token::new(TokenKind::Reserved, &two_char_op);
                    self.append_token(token);
                    c_iter.next(); // 次の文字を消費
                    continue;
                }
            }

            // 単一文字の記号トークン
            if "+-*/()<>".contains(c) {
                let token = Token::new(TokenKind::Reserved, &c.to_string());
                self.append_token(token);
                continue;
            }

            if c.is_digit(10) {
                let mut num_str = String::new();
                num_str.push(c);
                while let Some(&next_c) = c_iter.peek() {
                    if next_c.is_digit(10) {
                        num_str.push(next_c);
                        c_iter.next();
                    } else {
                        break;
                    }
                }
                let val = num_str.parse::<i64>().unwrap();
                let length = num_str.len();
                let token = Token {
                    kind: TokenKind::Num,
                    next: None,
                    val,
                    input: num_str,
                    length,
                };
                self.append_token(token);
                continue;
            }

            panic!("不明な文字です: {}", c);
        }
        let eof_token = Token::new(TokenKind::EOF, "");
        self.append_token(eof_token);
        self.current_token = self.head.clone();
    }
}
