mod decl;
mod expr;
mod stmt;

use crate::decl::Decl;
use crate::errors::CompileError;
use crate::func::{Func, FuncId, LocalVar};
use crate::symbol::{ScopedTable, Symbol, SymbolId, Tag};
use crate::token::{Token, TokenKind};
use crate::types::TypeRef;
use crate::utils::{AlignUp, Span};
use std::collections::HashMap;

// switch文のコンテキスト情報
struct SwitchCtx {
    label: usize,
    cases: Vec<(i64, usize)>,
    default_label: Option<usize>,
}

// ループとswitchを統合したコンテキスト
enum BreakableCtx {
    Loop(usize),            // ループのラベル
    Switch(Box<SwitchCtx>), // switchのコンテキスト
}

pub(crate) struct Ast<'a> {
    tokens: &'a [Token],
    token_pos: usize,
    pub(crate) funcs: Vec<Func>,
    current_func: Option<FuncId>,
    symbol_table: ScopedTable,
    pub(crate) string_literals: HashMap<String, usize>,
    label_seq: usize,
    loop_stack: Vec<usize>,
    breakable_stack: Vec<BreakableCtx>, // ループとswitchを統合
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
            breakable_stack: Vec::new(),
        }
    }

    // translation_unit ::= external_decl*
    pub(crate) fn translation_unit(&mut self) -> Result<(), CompileError> {
        while !self.at_eof() {
            self.external_decl()?;
        }
        Ok(())
    }

    // パーサーの試行を行い、成功した場合は結果を返し、失敗した場合はトークンを元の位置に戻す
    fn attempt<F, T>(&mut self, mut parser: F) -> Option<T>
    where
        F: FnMut(&mut Self) -> Result<T, CompileError>,
    {
        let saved_pos = self.token_pos;
        match parser(self) {
            Ok(res) => Some(res),
            Err(_) => {
                self.token_pos = saved_pos;
                None
            }
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

    // 列挙定数シンボルを登録
    fn register_enum_const_symbol(
        &mut self,
        name: &str,
        value: i64,
        span: Span,
    ) -> Result<SymbolId, CompileError> {
        // 同じスコープに同名の列挙定数が存在する場合はエラー
        if self.symbol_table.find_symbol_in_current_scope(name).is_some() {
            return Err(CompileError::Redecl { name: name.to_string(), span });
        }
        let symbol = Symbol::new_enum_const(name, value);
        Ok(self.symbol_table.insert_symbol(name, symbol))
    }

    // 関数定義を登録
    fn register_func_def(&mut self, func: Func) -> FuncId {
        self.funcs.push(func);
        FuncId(self.funcs.len() - 1)
    }

    fn get_current_func_mut(&mut self) -> Result<&mut Func, CompileError> {
        self.current_func.and_then(|func_id| self.funcs.get_mut(func_id.0)).ok_or_else(|| {
            CompileError::InternalError {
                msg: "現在の関数が設定されていません".to_string()
            }
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
                msg: "現在の関数が設定されていません".to_string()
            })
        }
    }

    fn push_scope(&mut self) {
        self.symbol_table.push_scope();
    }

    fn pop_scope(&mut self) {
        self.symbol_table.pop_scope();
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
        // 同じスコープに同名が存在する場合はエラー
        if self.symbol_table.find_symbol_in_current_scope(&decl.name).is_some() {
            return Err(CompileError::Redecl { name: decl.name.to_string(), span: decl.span });
        }
        let is_defined = !decl.ty.is_extern();
        let symbol = Symbol::new_var(&decl.name, decl.ty, owner, decl.init.clone(), is_defined);
        Ok(self.symbol_table.insert_symbol(&decl.name, symbol))
    }

    fn register_typedef(
        &mut self,
        name: &str,
        ty: TypeRef,
        span: Span,
    ) -> Result<SymbolId, CompileError> {
        // 同じスコープに同名が存在する場合はエラー
        if self.symbol_table.find_symbol_in_current_scope(name).is_some() {
            return Err(CompileError::Redecl { name: name.to_string(), span });
        }
        let aliased_ty = TypeRef::register(ty.kind(), ty.attr(), None); // typedefを剥がす
        let symbol = Symbol::new_typedef(name, aliased_ty);
        Ok(self.symbol_table.insert_symbol(name, symbol))
    }

    fn find_typedef(&self, name: &str) -> Option<&Symbol> {
        self.symbol_table.find_symbol(name).filter(|sym| sym.is_typedef())
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

    // タグの重複チェックと型の登録
    fn validate_and_register_tag(
        &mut self,
        tag_name: &str,
        ty: TypeRef,
        span: Span,
    ) -> Result<(), CompileError> {
        if let Some(existing_tag) = self.find_tag_in_current_scope(tag_name) {
            if !existing_tag.ty.is_incomplete() {
                return Err(CompileError::InvalidDecl {
                    msg: format!("タグ '{}' はすでに定義されています", tag_name),
                    span,
                });
            }
            // 不完全型の場合は既存のTypeRefを使用
            self.register_tag(tag_name, existing_tag.ty);
        } else {
            // タグを未完成型で登録
            self.register_tag(tag_name, ty);
        }
        Ok(())
    }

    fn next_label(&mut self) -> usize {
        let seq = self.label_seq;
        self.label_seq += 1;
        seq
    }

    fn push_loop(&mut self, label_seq: usize) {
        self.loop_stack.push(label_seq);
        self.breakable_stack.push(BreakableCtx::Loop(label_seq));
    }

    fn pop_loop(&mut self) -> Result<(), CompileError> {
        self.loop_stack.pop().ok_or_else(|| CompileError::InternalError {
            msg: "ループスタックが空です".to_string(),
        })?;
        // breakable_stackからもpop
        match self.breakable_stack.pop() {
            Some(BreakableCtx::Loop(_)) => Ok(()),
            _ => Err(CompileError::InternalError {
                msg: "breakable_stackの整合性が取れていません。Loopコンテキストを期待しました。"
                    .to_string(),
            }),
        }?;
        Ok(())
    }

    fn current_loop_label(&self) -> Option<usize> {
        self.loop_stack.last().copied()
    }

    // ループスコープを管理するRAIIヘルパー
    // スコープを抜ける際に自動的にpop_loopを呼び出す
    fn with_loop_scope<F, T>(&mut self, label: usize, f: F) -> Result<T, CompileError>
    where
        F: FnOnce(&mut Self) -> Result<T, CompileError>,
    {
        self.push_loop(label);
        let result = f(self);
        let pop_result = self.pop_loop();
        match result {
            Ok(value) => pop_result.map(|_| value),
            Err(e) => Err(e),
        }
    }

    fn push_switch(&mut self, label: usize) {
        let ctx = SwitchCtx { label, cases: Vec::new(), default_label: None };
        self.breakable_stack.push(BreakableCtx::Switch(Box::new(ctx)));
    }

    fn pop_switch(&mut self) -> Result<SwitchCtx, CompileError> {
        // breakable_stackからpop
        if let Some(BreakableCtx::Switch(ctx)) = self.breakable_stack.pop() {
            Ok(*ctx)
        } else {
            Err(CompileError::InternalError { msg: "switchスタックが空です".to_string() })
        }
    }

    fn current_breakable_label(&self) -> Option<usize> {
        self.breakable_stack.last().map(|ctx| match ctx {
            BreakableCtx::Loop(label) => *label,
            BreakableCtx::Switch(switch_ctx) => switch_ctx.label,
        })
    }

    fn add_case(&mut self, val: i64, case_label: usize, span: Span) -> Result<(), CompileError> {
        // breakable_stackから最後のswitchを見つける
        let switch_ctx = self
            .breakable_stack
            .iter_mut()
            .rev()
            .find_map(|ctx| match ctx {
                BreakableCtx::Switch(switch_ctx) => Some(switch_ctx.as_mut()),
                _ => None,
            })
            .ok_or_else(|| CompileError::InvalidStmt {
                msg: "case文がswitch文の外にあります".to_string(),
                span,
            })?;
        // 重複チェック
        if switch_ctx.cases.iter().any(|(v, _)| *v == val) {
            return Err(CompileError::InvalidStmt {
                msg: format!("case値 {} が重複しています", val),
                span,
            });
        }
        switch_ctx.cases.push((val, case_label));
        Ok(())
    }

    fn set_default(&mut self, default_label: usize, span: Span) -> Result<(), CompileError> {
        // breakable_stackから最後のswitchを見つける
        let switch_ctx = self
            .breakable_stack
            .iter_mut()
            .rev()
            .find_map(|ctx| match ctx {
                BreakableCtx::Switch(switch_ctx) => Some(switch_ctx.as_mut()),
                _ => None,
            })
            .ok_or_else(|| CompileError::InvalidStmt {
                msg: "default文がswitch文の外にあります".to_string(),
                span,
            })?;
        if switch_ctx.default_label.is_some() {
            return Err(CompileError::InvalidStmt {
                msg: "defaultラベルが重複しています".to_string(),
                span,
            });
        }
        switch_ctx.default_label = Some(default_label);
        Ok(())
    }

    // 現在のトークンを取得
    fn get_token(&self) -> &Token {
        &self.tokens[self.token_pos]
    }

    fn current_span(&self) -> Span {
        self.get_token().span
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

    fn consume(&mut self, kind: &TokenKind) -> Option<Span> {
        let token = self.tokens.get(self.token_pos)?;
        if &token.kind == kind {
            let span = token.span;
            self.advance_token();
            Some(span)
        } else {
            None
        }
    }

    fn consume_punct(&mut self, sym: &str) -> Option<Span> {
        self.consume(&TokenKind::Punct(sym.to_string()))
    }

    fn consume_keyword(&mut self, word: &str) -> Option<Span> {
        self.consume(&TokenKind::Keyword(word.to_string()))
    }

    fn consume_ident(&mut self) -> Option<(String, Span)> {
        match self.get_token() {
            Token { kind: TokenKind::Ident(name), span } => {
                let result = (name.clone(), *span);
                self.advance_token();
                Some(result)
            }
            _ => None,
        }
    }

    fn consume_string(&mut self) -> Option<(String, Span)> {
        match self.get_token() {
            Token { kind: TokenKind::StrLiteral(s), span } => {
                let result = (s.clone(), *span);
                self.advance_token();
                Some(result)
            }
            _ => None,
        }
    }

    fn consume_const(&mut self) -> Option<(i64, Span)> {
        match self.get_token() {
            Token { kind: TokenKind::IntConst(val) | TokenKind::CharConst(val), span } => {
                let result = (*val, *span);
                self.advance_token();
                Some(result)
            }
            _ => None,
        }
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<Span, CompileError> {
        let token = self.get_token();
        if &token.kind == kind {
            let span = token.span;
            self.advance_token();
            Ok(span)
        } else {
            Err(CompileError::UnexpectedToken {
                expected: kind.clone(),
                found: token.kind.clone(),
                span: token.span,
            })
        }
    }

    fn expect_punct(&mut self, sym: &str) -> Result<Span, CompileError> {
        self.expect(&TokenKind::Punct(sym.to_string()))
    }

    fn expect_keyword(&mut self, word: &str) -> Result<Span, CompileError> {
        self.expect(&TokenKind::Keyword(word.to_string()))
    }

    fn peek_punct(&self, sym: &str) -> bool {
        matches!(&self.get_token().kind, TokenKind::Punct(s) if s == sym)
    }

    fn peek_ident(&self) -> Option<String> {
        match self.get_token() {
            Token { kind: TokenKind::Ident(name), .. } => Some(name.clone()),
            _ => None,
        }
    }

    fn at_eof(&self) -> bool {
        self.tokens.is_empty() || matches!(self.get_token(), Token { kind: TokenKind::Eof, .. })
    }
}
