use std::collections::HashMap;

use crate::node::Node;
use crate::types::Type;

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct Declaration {
    pub(crate) name: String,
    pub(crate) ty: Type,
    pub(crate) init: Vec<Node>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct Variable {
    pub(crate) symbol_id: usize,
    pub(crate) init: Vec<Node>,
}

#[derive(Debug)]
pub(crate) struct Symbol {
    pub(crate) name: String,
    pub(crate) ty: Type,
    pub(crate) offset: usize,
}

impl Symbol {
    pub(crate) fn new(name: &str, ty: Type, offset: usize) -> Self {
        Self {
            name: name.to_string(),
            ty,
            offset,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Scope {
    table: HashMap<String, usize>, // シンボル名からシンボルのインデックスへのマッピング
}

#[derive(Debug)]
pub(crate) struct LocalSymbolTable {
    pub(crate) symbols: Vec<Symbol>, // 全てのシンボルのリスト（永続）
    scopes: Vec<Scope>,              // スコープのスタック（AST構成時のみ）
}

impl Default for LocalSymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalSymbolTable {
    pub(crate) fn new() -> Self {
        Self {
            symbols: Vec::new(),
            scopes: Vec::new(),
        }
    }

    pub(crate) fn enter_scope(&mut self) {
        self.scopes.push(Scope {
            table: HashMap::new(),
        });
    }

    pub(crate) fn leave_scope(&mut self) {
        self.scopes.pop();
    }

    pub(crate) fn insert(&mut self, name: String, symbol: Symbol) -> usize {
        let index = self.symbols.len();
        self.symbols.push(symbol);
        if let Some(scope) = self.scopes.last_mut() {
            scope.table.insert(name, index);
        }
        index
    }

    pub(crate) fn find_in_current_scope(&self, name: &str) -> Option<&Symbol> {
        if let Some(scope) = self.scopes.last()
            && let Some(&symbol_index) = scope.table.get(name)
        {
            return self.symbols.get(symbol_index);
        }

        None
    }

    pub(crate) fn find(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol_index) = scope.table.get(name) {
                return self.symbols.get(*symbol_index);
            }
        }
        None
    }
}

#[derive(Debug)]
pub(crate) struct ParamSymbolTable {
    pub(crate) symbols: Vec<Symbol>, // 全てのシンボルのリスト
    table: HashMap<String, usize>,   // パラメータ名からインデックスへのマッピング
}

impl Default for ParamSymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl ParamSymbolTable {
    pub(crate) fn new() -> Self {
        Self {
            symbols: Vec::new(),
            table: HashMap::new(),
        }
    }

    pub(crate) fn insert(&mut self, name: String, symbol: Symbol) -> usize {
        let index = self.symbols.len();
        self.symbols.push(symbol);
        self.table.insert(name, index);
        index
    }

    pub(crate) fn find(&self, name: &str) -> Option<&Symbol> {
        if let Some(&index) = self.table.get(name) {
            return self.symbols.get(index);
        }
        None
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Symbol> {
        self.symbols.iter()
    }
}

#[derive(Debug)]
pub(crate) struct GlobalSymbolTable {
    pub(crate) symbols: Vec<Symbol>, // 全てのシンボルのリスト
    table: HashMap<String, usize>,   // シンボル名からインデックスへのマッピング
}

impl Default for GlobalSymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalSymbolTable {
    pub(crate) fn new() -> Self {
        Self {
            symbols: Vec::new(),
            table: HashMap::new(),
        }
    }

    pub(crate) fn insert(&mut self, name: String, symbol: Symbol) -> usize {
        let index = self.symbols.len();
        self.symbols.push(symbol);
        self.table.insert(name, index);
        index
    }

    pub(crate) fn find(&self, name: &str) -> Option<&Symbol> {
        if let Some(&index) = self.table.get(name) {
            return self.symbols.get(index);
        }
        None
    }
}
