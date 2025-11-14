use core::{fmt, panic};

mod declaration;
mod expression;
mod statement;

use crate::node::Node;
use crate::token::Token;
use crate::types::Type;

#[derive(Clone)]
pub struct Var {
    pub name: String,
    pub offset: i64,
    pub ty: Box<Type>,
}

impl Var {
    pub fn new(name: &str, ty: Type) -> Self {
        Var {
            name: name.to_string(),
            offset: 0,
            ty: Box::new(ty),
        }
    }
}

impl fmt::Debug for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Var {{ name: '{}', type: {:?}, offset: {} }}",
            self.name, self.ty, self.offset
        )
    }
}

pub struct Function {
    pub name: String,
    pub body: Vec<Box<Node>>,
    pub locals: Vec<Var>,
}

impl Function {
    pub fn new(name: &str) -> Self {
        Function {
            name: name.to_string(),
            body: Vec::new(),
            locals: Vec::new(),
        }
    }
}

impl Function {
    fn gen_lvar(&mut self, mut var: Var) -> Result<(), &str> {
        if self.find_lvar(&var.name).is_some() {
            return Err("同じ名前のローカル変数が既に存在します");
        }
        var.offset = if let Some(first_var) = self.locals.first() {
            first_var.offset + var.ty.size_of() as i64
        } else {
            var.ty.size_of() as i64
        };
        self.locals.insert(0, var);
        Ok(())
    }

    fn find_lvar(&mut self, name: &str) -> Option<&mut Var> {
        for var in &mut self.locals {
            if var.name == name {
                return Some(var);
            }
        }
        None
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Function {{ name: '{}', locals: {:?} }}",
            self.name, self.locals
        )
    }
}

pub struct Ast {
    tokens: Vec<Token>,
    pub globals: Vec<Var>,
    pub funcs: Vec<Box<Function>>,
    current_func: Option<Box<Function>>,
}

impl Ast {
    pub fn new(tokens: &[Token]) -> Self {
        Ast {
            tokens: tokens.to_vec(),
            globals: Vec::new(),
            funcs: Vec::new(),
            current_func: None,
        }
    }

    fn gen_gvar(&mut self, var: Var) -> Result<(), &str> {
        if self.find_gvar(&var.name).is_some() {
            return Err("同じ名前のグローバル変数が既に存在します");
        }
        self.globals.push(var);
        Ok(())
    }

    fn find_gvar(&mut self, name: &str) -> Option<&mut Var> {
        for var in &mut self.globals {
            if var.name == name {
                return Some(var);
            }
        }
        None
    }

    fn consume(&mut self, token: &Token) -> bool {
        match self.tokens.first() {
            Some(t) if t == token => {
                self.tokens.remove(0);
                true
            }
            _ => false,
        }
    }

    fn consume_punctuator(&mut self, sym: &str) -> bool {
        self.consume(&Token::Punctuator(sym.to_string()))
    }

    fn consume_keyword(&mut self, word: &str) -> bool {
        self.consume(&Token::Keyword(word.to_string()))
    }

    fn consume_ident(&mut self) -> Option<String> {
        match self.tokens.first() {
            Some(Token::Identifier(name)) => {
                let name_clone = name.clone();
                self.tokens.remove(0);
                Some(name_clone)
            }
            _ => None,
        }
    }

    fn expect(&mut self, token: &Token) -> Result<(), &str> {
        match self.tokens.first() {
            Some(t) if t == token => {
                self.tokens.remove(0);
                Ok(())
            }
            _ => Err("期待されたトークンではありません"),
        }
    }

    fn expect_punctuator(&mut self, sym: &str) -> Result<(), &str> {
        self.expect(&Token::Punctuator(sym.to_string()))
    }

    fn expect_reserved(&mut self, word: &str) -> Result<(), &str> {
        self.expect(&Token::Keyword(word.to_string()))
    }

    fn expect_number(&mut self) -> Result<i64, &str> {
        match self.tokens.first() {
            Some(Token::Num(val)) => {
                let val_clone = *val;
                self.tokens.remove(0);
                Ok(val_clone)
            }
            _ => Err("数値トークンではありません"),
        }
    }

    fn at_eof(&self) -> bool {
        self.tokens.is_empty() || matches!(self.tokens.first(), Some(Token::EOF))
    }

    // translation_unit ::= external_declaration*
    pub fn translation_unit(&mut self) {
        while !self.at_eof() {
            if let Some(func) = self.external_declaration() {
                self.funcs.push(func);
            }
        }
    }

    // external_declaration ::= func_def
    //                          | declaration
    fn external_declaration(&mut self) -> Option<Box<Function>> {
        if let Some(result) = self.declaration() {
            match result {
                Ok(var) => {
                    // グローバル変数宣言
                    self.gen_gvar(var).unwrap();
                    return None;
                }
                Err(var) => {
                    // 関数定義
                    if let Ok(func) = self.func_def(var) {
                        return Some(func);
                    }
                    panic!("関数定義のパースに失敗しました");
                }
            }
        }
        panic!("external_declarationのパースに失敗しました");
    }

    // func_body ::= compound_stmt
    fn func_body(&mut self) -> Option<Box<Node>> {
        self.compound_stmt()
    }

    // func_def ::= "(" (declarator ("," declarator)*)? ")" func_body
    fn func_def(&mut self, global_info: Var) -> Result<Box<Function>, &str> {
        if self.consume_punctuator("(") {
            // 関数の引数のパース（型情報もパース）
            let mut func = Function::new(&global_info.name);
            loop {
                if let Ok(param) = self.declarator() {
                    func.gen_lvar(param).unwrap();
                }
                if !self.consume_punctuator(",") {
                    break;
                }
            }
            self.expect_punctuator(")").unwrap();
            if self.consume_punctuator(";") {
                // 関数プロトタイプ宣言
                return Err("関数プロトタイプ宣言はサポートされていません");
            }

            self.current_func = Some(Box::new(func)); // 現在の関数を設定

            // 関数本体のパース
            if let Some(node) = self.func_body() {
                self.current_func.as_mut().unwrap().body = node.body;
                return Ok(self.current_func.take().unwrap());
            }
            return Err("関数本体のパースに失敗しました");
        }
        Err("関数の引数リストのパースに失敗しました")
    }
}
