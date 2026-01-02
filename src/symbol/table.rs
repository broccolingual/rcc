use crate::symbol::Symbol;
use crate::types::Type;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
struct Scope {
    names: HashMap<String, Rc<RefCell<Symbol>>>, // name -> symbol_id
    tags: HashMap<String, Type>,                 // name -> type
}

#[derive(Debug)]
pub(crate) struct ScopedTable {
    symbols: Vec<Rc<RefCell<Symbol>>>,
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

    pub(crate) fn get_symbols(&self) -> &Vec<Rc<RefCell<Symbol>>> {
        &self.symbols
    }

    pub(crate) fn find_symbol(&self, name: &str) -> Option<Rc<RefCell<Symbol>>> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.names.get(name) {
                return Some(Rc::clone(symbol));
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

    pub(crate) fn find_symbol_in_current_scope(&self, name: &str) -> Option<Rc<RefCell<Symbol>>> {
        if let Some(scope) = self.scopes.last()
            && let Some(symbol) = scope.names.get(name)
        {
            return Some(Rc::clone(symbol));
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

    pub(crate) fn insert_symbol(&mut self, name: &str, symbol: Symbol) -> Rc<RefCell<Symbol>> {
        let symbol = Rc::new(RefCell::new(symbol));
        self.symbols.push(Rc::clone(&symbol));
        if let Some(scope) = self.scopes.last_mut() {
            scope.names.insert(name.to_string(), Rc::clone(&symbol));
        }
        symbol
    }

    pub(crate) fn insert_tag(&mut self, name: &str, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.tags.insert(name.to_string(), ty);
        }
    }
}
