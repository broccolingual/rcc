mod declaration;
mod expression;
mod statement;

use crate::errors::CompileError;
use crate::function::Func;
use crate::symbol::{FlatTable, Symbol, Variable};
use crate::token::{Token, TokenKind};
use crate::types::{Decl, Type};
use std::collections::HashMap;

pub(crate) struct Ast<'a> {
    tokens: &'a [Token],
    token_pos: usize,
    pub(crate) globals: Vec<Variable>,
    global_symbol_table: FlatTable<Symbol>,
    global_tag_table: FlatTable<Type>,
    pub(crate) funcs: Vec<Func>,
    current_func: Option<Func>,
    pub(crate) string_literals: HashMap<String, usize>,
    label_seq: usize,
    loop_stack: Vec<usize>,
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
            string_literals: HashMap::new(),
            label_seq: 0,
            loop_stack: Vec::new(),
        }
    }

    pub(crate) fn get_global_symbol_by_id(&self, symbol_id: usize) -> Option<&Symbol> {
        self.global_symbol_table.items.get(symbol_id)
    }

    fn get_current_func(&mut self) -> Result<&mut Func, CompileError> {
        self.current_func
            .as_mut()
            .ok_or_else(|| CompileError::InternalError {
                msg: "現在の関数が設定されていません".to_string(),
            })
    }

    fn get_prev_token_span(&self) -> Option<(usize, usize)> {
        self.token_pos
            .checked_sub(1)
            .and_then(|pos| self.tokens.get(pos))
            .map(|token| token.span)
    }

    fn register_string_literal(&mut self, string: &str) -> usize {
        // 既に登録されている場合はそのインデックスを返す
        if let Some(&index) = self.string_literals.get(string) {
            return index;
        }
        // 新規登録
        let index = self.string_literals.len();
        self.string_literals.insert(string.to_string(), index);
        index
    }

    fn register_global_var(&mut self, decl: Decl) -> Result<(), CompileError> {
        if self.global_symbol_table.find(&decl.name).is_some() {
            return Err(CompileError::Redecl {
                name: decl.name,
                span: decl.span,
            });
        }
        let symbol = Symbol::new(&decl.name, decl.ty, 0);
        let symbol_id = self.global_symbol_table.insert(decl.name, symbol);
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
            return Err(CompileError::Redecl { name, span });
        }
        self.global_tag_table.insert(name, ty);
        Ok(())
    }

    fn find_struct_tag(&self, name: &str) -> Option<&Type> {
        self.global_tag_table.find(name)
    }

    // 関数名から関数を検索し、戻り値の型を取得
    fn get_func_return_type(&self, name: &str) -> Option<&Type> {
        for func in &self.funcs {
            if func.name == name {
                return Some(&func.return_ty);
            }
        }
        None
    }

    fn next_label(&mut self) -> usize {
        let seq = self.label_seq;
        self.label_seq += 1;
        seq
    }

    fn push_loop(&mut self, label_seq: usize) {
        self.loop_stack.push(label_seq);
    }

    fn pop_loop(&mut self) {
        self.loop_stack.pop();
    }

    fn current_loop_label(&self) -> Option<usize> {
        self.loop_stack.last().copied()
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

    fn consume_punct(&mut self, sym: &str) -> Option<&Token> {
        self.consume(&TokenKind::Punct(sym.to_string()))
    }

    fn consume_keyword(&mut self, word: &str) -> Option<&Token> {
        self.consume(&TokenKind::Keyword(word.to_string()))
    }

    fn consume_ident(&mut self) -> Option<(String, &Token)> {
        let token_pos = self.token_pos;
        match self.get_token() {
            Some(Token {
                kind: TokenKind::Ident(name),
                ..
            }) => {
                let name_clone = name.clone();
                self.advance_token();
                Some((name_clone, &self.tokens[token_pos]))
            }
            _ => None,
        }
    }

    fn consume_string(&mut self) -> Option<(String, &Token)> {
        let token_pos = self.token_pos;
        match self.get_token() {
            Some(Token {
                kind: TokenKind::String(s),
                ..
            }) => {
                let s_clone = s.clone();
                self.advance_token();
                Some((s_clone, &self.tokens[token_pos]))
            }
            _ => None,
        }
    }

    fn consume_number(&mut self) -> Option<(i64, &Token)> {
        let token_pos = self.token_pos;
        match self.get_token() {
            Some(Token {
                kind: TokenKind::Number(val),
                ..
            }) => {
                let val = *val;
                self.advance_token();
                Some((val, &self.tokens[token_pos]))
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

    fn expect_punct(&mut self, sym: &str) -> Result<(), CompileError> {
        self.expect(&TokenKind::Punct(sym.to_string()))
    }

    fn expect_keyword(&mut self, word: &str) -> Result<(), CompileError> {
        self.expect(&TokenKind::Keyword(word.to_string()))
    }

    fn peek_punct(&mut self, sym: &str) -> bool {
        match self.get_token() {
            Some(token) => matches!(&token.kind, TokenKind::Punct(s) if s == sym),
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

    // translation_unit ::= external_decl*
    pub(crate) fn translation_unit(&mut self) -> Result<(), CompileError> {
        while !self.at_eof() {
            self.external_decl()?;
        }
        Ok(())
    }
}
