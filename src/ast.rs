use core::fmt;

mod declaration;
mod expression;
mod statement;

use crate::errors::CompileError;
use crate::node::{Node, NodeKind};
use crate::token::{Token, TokenKind};
use crate::types::{Type, TypeKind};

#[derive(Clone, PartialEq, Eq)]
pub struct Var {
    pub name: String,
    pub offset: usize,
    pub ty: Box<Type>,
    pub init: Option<Box<Node>>,
}

impl Var {
    pub fn new(name: &str, ty: Type) -> Self {
        Var {
            name: name.to_string(),
            offset: 0,
            ty: Box::new(ty),
            init: None,
        }
    }
}

impl fmt::Debug for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {:?} (offset: {})", self.name, self.ty, self.offset)?;
        if self.init.is_some() {
            write!(f, " = {:?}", self.init)?;
        }
        Ok(())
    }
}

pub struct Function {
    pub name: String,
    pub body: Vec<Box<Node>>,
    pub locals: Vec<Var>,
    pub return_ty: Type,
}

impl Function {
    pub fn new(name: &str) -> Self {
        Function {
            name: name.to_string(),
            body: Vec::new(),
            locals: Vec::new(),
            return_ty: Type::new(&TypeKind::Void),
        }
    }
}

impl Function {
    fn gen_lvar(&mut self, mut var: Var) -> Result<(), CompileError> {
        if self.find_lvar(&var.name).is_some() {
            return Err(CompileError::Redeclaration {
                name: var.name.clone(),
            });
        }
        // TODO: 構造体の場合のオフセット計算
        var.offset = if let Some(first_var) = self.locals.first() {
            first_var.offset + var.ty.size_of()
        } else {
            var.ty.size_of()
        };
        self.locals.insert(0, var); // オフセット計算のために先頭に追加
        Ok(())
    }

    fn find_lvar(&mut self, name: &str) -> Option<&mut Var> {
        self.locals
            .iter_mut()
            .find(|var| var.name == name)
            .map(|v| v as _)
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
    token_pos: usize,
    pub globals: Vec<Var>,
    pub funcs: Vec<Box<Function>>,
    current_func: Option<Box<Function>>,
    pub string_literals: Vec<String>,
}

impl Ast {
    pub fn new(tokens: &[Token]) -> Self {
        Ast {
            tokens: tokens.to_vec(),
            token_pos: 0,
            globals: Vec::new(),
            funcs: Vec::new(),
            current_func: None,
            string_literals: Vec::new(),
        }
    }

    fn get_current_func(&mut self) -> Result<&mut Box<Function>, CompileError> {
        self.current_func
            .as_mut()
            .ok_or_else(|| CompileError::InternalError {
                msg: "現在の関数が設定されていません".to_string(),
            })
    }

    fn gen_gvar(&mut self, var: Var) -> Result<(), CompileError> {
        if self.find_gvar(&var.name).is_some() {
            return Err(CompileError::Redeclaration {
                name: var.name.clone(),
            });
        }
        self.globals.push(var);
        Ok(())
    }

    fn find_gvar(&mut self, name: &str) -> Option<&mut Var> {
        self.globals
            .iter_mut()
            .find(|var| var.name == name)
            .map(|v| v as _)
    }

    // 現在のトークンを取得
    fn get_token(&self) -> Option<&Token> {
        self.tokens.get(self.token_pos)
    }

    // トークンを1つ進める
    fn advance_token(&mut self) {
        if self.token_pos < self.tokens.len() - 1 {
            self.token_pos += 1;
        }
    }

    // トークンを1つ戻す
    fn retreat_token(&mut self) {
        if self.token_pos > 0 {
            self.token_pos -= 1;
        }
    }

    fn consume(&mut self, kind: &TokenKind) -> Option<&Token> {
        if let Some(t) = self.tokens.get(self.token_pos)
            && &t.kind == kind
        {
            self.advance_token();
            return self.tokens.get(self.token_pos.saturating_sub(1));
        }
        None
    }

    fn consume_punctuator(&mut self, sym: &str) -> Option<&Token> {
        self.consume(&TokenKind::Punctuator(sym.to_string()))
    }

    fn consume_keyword(&mut self, word: &str) -> Option<&Token> {
        self.consume(&TokenKind::Keyword(word.to_string()))
    }

    fn consume_ident(&mut self) -> Option<String> {
        match self.get_token() {
            Some(Token {
                kind: TokenKind::Identifier(name),
                ..
            }) => {
                let name_clone = name.clone();
                self.advance_token();
                Some(name_clone)
            }
            _ => None,
        }
    }

    fn consume_string(&mut self) -> Option<String> {
        match self.get_token() {
            Some(Token {
                kind: TokenKind::String(s),
                ..
            }) => {
                let s_clone = s.clone();
                self.advance_token();
                Some(s_clone)
            }
            _ => None,
        }
    }

    fn consume_number(&mut self) -> Option<i64> {
        match self.get_token() {
            Some(Token {
                kind: TokenKind::Number(val),
                ..
            }) => {
                let val_clone = *val;
                self.advance_token();
                Some(val_clone)
            }
            _ => None,
        }
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<(), CompileError> {
        match self.get_token() {
            Some(t) => {
                if &t.kind == kind {
                    self.advance_token();
                    return Ok(());
                }
                Err(CompileError::UnexpectedToken {
                    expected: kind.clone(),
                    found: t.kind.clone(),
                    span: t.span,
                })
            }
            _ => Err(CompileError::UnexpectedEof),
        }
    }

    fn expect_punctuator(&mut self, sym: &str) -> Result<(), CompileError> {
        self.expect(&TokenKind::Punctuator(sym.to_string()))
    }

    fn expect_keyword(&mut self, word: &str) -> Result<(), CompileError> {
        self.expect(&TokenKind::Keyword(word.to_string()))
    }

    fn expect_number(&mut self) -> Result<i64, CompileError> {
        match self.get_token() {
            Some(token) => {
                if let TokenKind::Number(val) = &token.kind {
                    let val_clone = *val;
                    self.advance_token();
                    return Ok(val_clone);
                }
                Err(CompileError::UnexpectedToken {
                    expected: TokenKind::Number(0),
                    found: token.kind.clone(),
                    span: token.span,
                })
            }
            _ => Err(CompileError::UnexpectedEof),
        }
    }

    fn at_eof(&mut self) -> bool {
        self.tokens.is_empty()
            || matches!(
                self.get_token(),
                Some(Token {
                    kind: TokenKind::EOF,
                    ..
                })
            )
    }

    // translation_unit ::= external_declaration*
    pub fn translation_unit(&mut self) -> Result<(), CompileError> {
        while !self.at_eof() {
            self.external_declaration()?;
        }
        Ok(())
    }

    // external_declaration ::= func_def
    //                          | declaration
    fn external_declaration(&mut self) -> Result<(), CompileError> {
        // 関数定義
        let token_pos = self.token_pos;
        if let Some(func) = self.func_def()? {
            self.funcs.push(func);
            return Ok(());
        }

        self.token_pos = token_pos;
        // グローバル変数宣言
        if let Some(vars) = self.declaration()? {
            for var in vars {
                self.gen_gvar(var)?;
            }
            return Ok(());
        }
        Err(CompileError::InvalidDeclaration {
            msg: "外部宣言のパースに失敗しました".to_string(),
        })
    }

    // func_def ::= declaration_specifier declarator compound_stmt
    fn func_def(&mut self) -> Result<Option<Box<Function>>, CompileError> {
        let specifier = self.declaration_specifier()?;
        let base_kind = if let Some(specifier) = specifier {
            Type::from_ds(&vec![specifier]).unwrap()
        } else {
            return Err(CompileError::InvalidTypeSpecifier {
                msg: "関数定義の型指定子が無効です".to_string(),
            });
        };
        let func_decl = if let Ok(var) = self.declarator(base_kind) {
            var
        } else {
            return Err(CompileError::InvalidDeclaration {
                msg: "関数定義のパースに失敗しました".to_string(),
            });
        };
        let mut func = Box::new(Function::new(&func_decl.name));
        if let TypeKind::Func { params, return_ty } = func_decl.ty.kind {
            for param in params {
                func.gen_lvar(param.clone())?;
            }
            func.return_ty = *return_ty;
        } else {
            return Ok(None);
        }
        self.current_func = Some(func);
        let func_body = if let Some(func_body) = self.compound_stmt()? {
            func_body
        } else {
            return Ok(None);
        };
        func = self
            .current_func
            .take()
            .ok_or_else(|| CompileError::InternalError {
                msg: "現在の関数が設定されていません".to_string(),
            })?;
        if let NodeKind::Block { body } = func_body.kind {
            func.body = body;
        } else {
            return Err(CompileError::InvalidDeclaration {
                msg: "関数本体がブロックではありません".to_string(),
            });
        }
        Ok(Some(func))
    }
}
