mod declaration;
mod expression;
mod statement;

use crate::errors::CompileError;
use crate::function::Function;
use crate::node::NodeKind;
use crate::symbol::{FlatTable, Symbol, Variable};
use crate::token::{Token, TokenKind};
use crate::types::{Declaration, Type, TypeKind};

pub(crate) struct Ast<'a> {
    tokens: &'a [Token],
    token_pos: usize,
    pub(crate) globals: Vec<Variable>,
    global_symbol_table: FlatTable<Symbol>,
    global_tag_table: FlatTable<Type>,
    pub(crate) funcs: Vec<Function>,
    current_func: Option<Function>,
    pub(crate) string_literals: Vec<String>,
}

impl<'a> Ast<'a> {
    pub(crate) fn new(tokens: &'a [Token]) -> Self {
        Ast {
            tokens,
            token_pos: 0,
            global_symbol_table: FlatTable::new(),
            global_tag_table: FlatTable::new(),
            globals: Vec::new(),
            funcs: Vec::new(),
            current_func: None,
            string_literals: Vec::new(),
        }
    }

    pub(crate) fn get_global_symbol_by_id(&self, symbol_id: usize) -> Option<&Symbol> {
        self.global_symbol_table.items.get(symbol_id)
    }

    fn get_current_func(&mut self) -> Result<&mut Function, CompileError> {
        self.current_func
            .as_mut()
            .ok_or_else(|| CompileError::InternalError {
                msg: "現在の関数が設定されていません".to_string(),
            })
    }

    fn get_prev_token_span(&self) -> Option<(usize, usize)> {
        if self.token_pos == 0 {
            None
        } else {
            self.tokens.get(self.token_pos - 1).map(|t| t.span)
        }
    }

    fn register_string_literal(&mut self, s: &str) -> usize {
        let index = self.string_literals.len();
        self.string_literals.push(s.to_string());
        index
    }

    fn register_global_var(&mut self, decl: Declaration) -> Result<(), CompileError> {
        if self.global_symbol_table.find(&decl.name).is_some() {
            return Err(CompileError::Redeclaration {
                name: decl.name,
                span: decl.span,
            });
        }
        let symbol = Symbol {
            name: decl.name.clone(),
            ty: decl.ty,
            offset: 0,
        };
        let symbol_id = self.global_symbol_table.insert(decl.name.clone(), symbol);
        self.globals.push(Variable {
            symbol_id,
            init: decl.init,
        });
        Ok(())
    }

    fn find_global_var(&self, name: &str) -> Option<&Symbol> {
        self.global_symbol_table.find(name)
    }

    fn register_struct_tag(
        &mut self,
        name: String,
        ty: Type,
        span: (usize, usize),
    ) -> Result<(), CompileError> {
        if self.global_tag_table.find(&name).is_some() {
            return Err(CompileError::Redeclaration { name, span });
        }
        self.global_tag_table.insert(name, ty);
        Ok(())
    }

    fn find_struct_tag(&self, name: &str) -> Option<&Type> {
        self.global_tag_table.find(name)
    }

    // 関数名から関数を検索し、戻り値の型を取得
    fn get_function_return_type(&self, name: &str) -> Option<&Type> {
        for func in &self.funcs {
            if func.name == name {
                return Some(&func.return_ty);
            }
        }
        None
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

    fn consume_ident(&mut self) -> Option<(String, &Token)> {
        match self.get_token() {
            Some(Token {
                kind: TokenKind::Identifier(name),
                ..
            }) => {
                let name_clone = name.clone();
                self.advance_token();
                Some((
                    name_clone,
                    self.tokens.get(self.token_pos.saturating_sub(1)).unwrap(),
                ))
            }
            _ => None,
        }
    }

    fn consume_string(&mut self) -> Option<(String, &Token)> {
        match self.get_token() {
            Some(Token {
                kind: TokenKind::String(s),
                ..
            }) => {
                let s_clone = s.clone();
                self.advance_token();
                Some((
                    s_clone,
                    self.tokens.get(self.token_pos.saturating_sub(1)).unwrap(),
                ))
            }
            _ => None,
        }
    }

    fn consume_number(&mut self) -> Option<(i64, &Token)> {
        match self.get_token() {
            Some(Token {
                kind: TokenKind::Number(val),
                ..
            }) => {
                let val_clone = *val;
                self.advance_token();
                Some((
                    val_clone,
                    self.tokens.get(self.token_pos.saturating_sub(1)).unwrap(),
                ))
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

    fn peek_punctuator(&mut self, sym: &str) -> bool {
        match self.get_token() {
            Some(token) => matches!(&token.kind, TokenKind::Punctuator(s) if s == sym),
            _ => false,
        }
    }

    fn at_eof(&mut self) -> bool {
        self.tokens.is_empty()
            || matches!(
                self.get_token(),
                Some(Token {
                    kind: TokenKind::Eof,
                    ..
                })
            )
    }

    // translation_unit ::= external_declaration*
    pub(crate) fn translation_unit(&mut self) -> Result<(), CompileError> {
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
        self.token_pos = token_pos; // 関数定義でなかった場合、トークン位置を元に戻す
        // グローバル変数宣言
        if let Some(declarations) = self.declaration()? {
            for declaration in declarations {
                self.register_global_var(declaration)?;
            }
            return Ok(());
        }
        let span = self.get_prev_token_span().unwrap_or((0, 0));
        Err(CompileError::InvalidDeclaration {
            msg: "外部宣言のパースに失敗しました。関数定義またはグローバル変数宣言が必要です"
                .to_string(),
            span,
        })
    }

    // func_def ::= declaration_specifiers declarator compound_stmt
    fn func_def(&mut self) -> Result<Option<Function>, CompileError> {
        let specifiers = self.declaration_specifiers()?;
        if specifiers.is_empty() {
            return Ok(None);
        }
        let base_ty = Type::from_ds(specifiers).ok_or_else(|| {
            let span = self.get_prev_token_span().unwrap_or((0, 0));
            CompileError::InvalidDeclaration {
                msg: "関数の基本型の解決に失敗しました。無効な型指定子の組み合わせです".to_string(),
                span,
            }
        })?;
        let func_decl = self.declarator(&base_ty)?;
        let mut func = Function::new(&func_decl.name);
        if let TypeKind::Func { params, return_ty } = func_decl.ty.kind {
            for param_decl in params {
                func.register_param(param_decl)?;
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
            let span = func_body.span;
            return Err(CompileError::InvalidDeclaration {
                msg: "関数本体がブロックではありません。'{' と '}' で囲まれた複合文が必要です"
                    .to_string(),
                span,
            });
        }
        self.current_func = None; // 関数の登録が終わったら現在の関数をクリア
        Ok(Some(func))
    }
}
