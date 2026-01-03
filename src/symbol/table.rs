use crate::symbol::Symbol;
use crate::types::Type;
use std::collections::HashMap;

#[derive(Debug)]
struct Scope {
    names: HashMap<String, usize>, // name -> symbol_id
    tags: HashMap<String, Type>,   // name -> type
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

    pub(crate) fn get_symbol(&self, symbol_id: usize) -> &Symbol {
        &self.symbols[symbol_id]
    }

    pub(crate) fn get_symbol_mut(&mut self, symbol_id: usize) -> &mut Symbol {
        &mut self.symbols[symbol_id]
    }

    pub(crate) fn find_symbol_id(&mut self, name: &str) -> Option<usize> {
        for scope in self.scopes.iter().rev() {
            if let Some(&symbol_id) = scope.names.get(name) {
                return Some(symbol_id);
            }
        }
        None
    }

    pub(crate) fn find_tag(&self, name: &str) -> Option<&Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.tags.get(name) {
                return Some(ty);
            }
        }
        None
    }

    pub(crate) fn find_symbol_in_current_scope(&self, name: &str) -> Option<&Symbol> {
        if let Some(scope) = self.scopes.last()
            && let Some(&symbol_id) = scope.names.get(name)
        {
            return self.symbols.get(symbol_id);
        }

        None
    }

    pub(crate) fn find_tag_in_current_scope(&self, name: &str) -> Option<&Type> {
        if let Some(scope) = self.scopes.last()
            && let Some(ty) = scope.tags.get(name)
        {
            return Some(ty);
        }

        None
    }

    pub(crate) fn insert_symbol(&mut self, name: &str, symbol: Symbol) -> usize {
        self.symbols.push(symbol);
        let symbol_id = self.symbols.len() - 1;
        if let Some(scope) = self.scopes.last_mut() {
            scope.names.insert(name.to_string(), symbol_id);
        }
        symbol_id
    }

    pub(crate) fn insert_tag(&mut self, name: &str, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.tags.insert(name.to_string(), ty);
        }
    }
}
