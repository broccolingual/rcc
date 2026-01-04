mod declaration;
mod expression;
mod statement;

use crate::errors::CompileError;
use crate::function::{Func, FuncId, LocalVar};
use crate::symbol::{ScopedTable, Symbol, SymbolId, Tag};
use crate::token::{Token, TokenKind};
use crate::types::{AlignUp, Decl, TypeRef};
use std::collections::HashMap;

pub(crate) struct Ast<'a> {
    tokens: &'a [Token],
    token_pos: usize,
    pub(crate) funcs: Vec<Func>,
    current_func: Option<FuncId>,
    symbol_table: ScopedTable,
    pub(crate) string_literals: HashMap<String, usize>,
    label_seq: usize,
    loop_stack: Vec<usize>,
}

impl<'a> Ast<'a> {
    pub(crate) fn new(tokens: &'a [Token]) -> Self {
        Ast {
            tokens,
            token_pos: 0,
            funcs: Vec::new(),
            current_func: None,
            symbol_table: ScopedTable::new(),
            string_literals: HashMap::new(),
            label_seq: 0,
            loop_stack: Vec::new(),
        }
    }

    pub(crate) fn get_symbols(&self) -> &Vec<Symbol> {
        self.symbol_table.get_symbols()
    }

    pub(crate) fn get_tags(&self) -> &Vec<Tag> {
        self.symbol_table.get_tags()
    }

    pub(crate) fn get_symbol(&self, symbol_id: SymbolId) -> &Symbol {
        self.symbol_table.get_symbol(symbol_id)
    }

    pub(crate) fn get_func(&self, func_id: FuncId) -> &Func {
        &self.funcs[func_id.0]
    }

    // 関数シンボルを登録
    fn register_func_symbol(&mut self, name: &str, ty: TypeRef, is_defined: bool) -> SymbolId {
        let symbol = Symbol::new_func(name, ty, is_defined);
        self.symbol_table.insert_symbol(name, symbol)
    }

    // 関数定義を登録
    fn register_func_def(&mut self, func: Func) -> FuncId {
        self.funcs.push(func);
        FuncId(self.funcs.len() - 1)
    }

    fn get_current_func_mut(&mut self) -> Result<&mut Func, CompileError> {
        self.current_func
            .and_then(|func_id| self.funcs.get_mut(func_id.0))
            .ok_or_else(|| CompileError::InternalError {
                msg: "現在の関数が設定されていません".to_string(),
            })
    }

    // 現在の関数のオフセットを計算
    fn calc_current_func_offset(&mut self) -> Result<(), CompileError> {
        if let Some(func_id) = self.current_func
            && let Some(func) = self.funcs.get_mut(func_id.0)
        {
            let calculate_offsets =
                |vars: &mut [LocalVar], symbol_table: &ScopedTable, mut offset: usize| {
                    for var in vars {
                        let symbol = symbol_table.get_symbol(var.symbol_id);
                        offset = offset.align_up(symbol.ty.align_of());
                        offset += symbol.ty.size_of();
                        var.offset = offset;
                    }
                    offset
                };

            let mut offset = 0;
            offset = calculate_offsets(&mut func.params, &self.symbol_table, offset); // パラメータのオフセット計算
            offset = calculate_offsets(&mut func.locals, &self.symbol_table, offset); // ローカル変数のオフセット計算
            func.stack_size = offset.align_up(16);
            Ok(())
        } else {
            Err(CompileError::InternalError {
                msg: "現在の関数が設定されていません".to_string(),
            })
        }
    }

    fn push_scope(&mut self) {
        self.symbol_table.push_scope();
    }

    fn pop_scope(&mut self) {
        self.symbol_table.pop_scope();
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

    fn register_var(
        &mut self,
        decl: &Decl,
        owner: Option<FuncId>,
    ) -> Result<SymbolId, CompileError> {
        // 同じスコープに同名の変数が存在する場合はエラー
        if self
            .symbol_table
            .find_symbol_in_current_scope(&decl.name)
            .is_some()
        {
            return Err(CompileError::Redecl {
                name: decl.name.to_string(),
                span: decl.span,
            });
        }
        let is_defined = !decl.ty.is_extern();
        let symbol = Symbol::new_var(&decl.name, decl.ty, owner, decl.init.clone(), is_defined);
        Ok(self.symbol_table.insert_symbol(&decl.name, symbol))
    }

    fn find_symbol_id(&self, name: &str) -> Option<SymbolId> {
        self.symbol_table.find_symbol_id(name)
    }

    fn find_symbol(&self, name: &str) -> Option<&Symbol> {
        self.symbol_table.find_symbol(name)
    }

    fn find_symbol_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        self.symbol_table.find_symbol_mut(name)
    }

    fn register_tag(&mut self, name: &str, ty: TypeRef) {
        // 同じスコープに同名のタグが存在しない場合のみ登録
        if self.symbol_table.find_tag_in_current_scope(name).is_none() {
            self.symbol_table.insert_tag(name, ty);
        }
    }

    fn find_tag(&self, name: &str) -> Option<&Tag> {
        self.symbol_table.find_tag(name)
    }

    fn find_tag_in_current_scope(&self, name: &str) -> Option<&Tag> {
        self.symbol_table.find_tag_in_current_scope(name)
    }

    fn next_label(&mut self) -> usize {
        let seq = self.label_seq;
        self.label_seq += 1;
        seq
    }

    fn push_loop(&mut self, label_seq: usize) {
        self.loop_stack.push(label_seq);
    }

    fn pop_loop(&mut self) -> Result<(), CompileError> {
        self.loop_stack
            .pop()
            .ok_or_else(|| CompileError::InternalError {
                msg: "ループスタックが空です".to_string(),
            })
            .map(|_| ())
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
        self.get_token()
            .map(|token| matches!(&token.kind, TokenKind::Punct(s) if s == sym))
            .unwrap_or(false)
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
