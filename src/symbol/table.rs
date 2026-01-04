use super::{Symbol, SymbolId};
use crate::types::TypeRef;
use std::collections::HashMap;

#[derive(Debug)]
struct Scope {
    names: HashMap<String, SymbolId>, // name -> symbol_id
    tags: HashMap<String, TypeRef>,   // name -> type
}

#[derive(Debug)]
pub(crate) struct ScopedTable {
    symbols: Vec<Symbol>,
    scopes: Vec<Scope>,
}

impl ScopedTable {
    pub(crate) fn new() -> Self {
        Self {
            symbols: Vec::new(),
            scopes: vec![Scope {
                names: HashMap::new(),
                tags: HashMap::new(),
            }],
        }
    }

    pub(crate) fn push_scope(&mut self) {
        self.scopes.push(Scope {
            names: HashMap::new(),
            tags: HashMap::new(),
        });
    }

    pub(crate) fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub(crate) fn get_symbols(&self) -> &Vec<Symbol> {
        &self.symbols
    }

    pub(crate) fn get_symbol(&self, symbol_id: SymbolId) -> &Symbol {
        &self.symbols[symbol_id.0]
    }

    pub(crate) fn get_symbol_mut(&mut self, symbol_id: SymbolId) -> &mut Symbol {
        &mut self.symbols[symbol_id.0]
    }

    pub(crate) fn find_symbol_id(&self, name: &str) -> Option<SymbolId> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.names.get(name).copied())
    }

    pub(crate) fn find_tag(&self, name: &str) -> Option<&TypeRef> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.tags.get(name))
    }

    pub(crate) fn find_symbol_in_current_scope(&self, name: &str) -> Option<&Symbol> {
        self.scopes.last().and_then(|scope| {
            scope
                .names
                .get(name)
                .map(|&symbol_id| self.get_symbol(symbol_id))
        })
    }

    pub(crate) fn find_tag_in_current_scope(&self, name: &str) -> Option<&TypeRef> {
        self.scopes.last().and_then(|scope| scope.tags.get(name))
    }

    pub(crate) fn insert_symbol(&mut self, name: &str, symbol: Symbol) -> SymbolId {
        self.symbols.push(symbol);
        let symbol_id = SymbolId(self.symbols.len() - 1);
        if let Some(scope) = self.scopes.last_mut() {
            scope.names.insert(name.to_string(), symbol_id);
        }
        symbol_id
    }

    pub(crate) fn insert_tag(&mut self, name: &str, ty: TypeRef) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.tags.insert(name.to_string(), ty);
        }
    }
}
