use core::fmt;
use std::collections::HashMap;

pub(crate) struct Scope {
    table: HashMap<String, usize>, // シンボル名からシンボルのインデックスへのマッピング
}

pub(crate) struct ScopedTable<T: fmt::Debug> {
    pub(crate) items: Vec<T>, // 全てのシンボルのリスト（永続）
    scopes: Vec<Scope>,       // スコープのスタック（AST構成時のみ）
}

impl<T: fmt::Debug> Default for ScopedTable<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: fmt::Debug> fmt::Debug for ScopedTable<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:?}", item)?;
        }
        Ok(())
    }
}

impl<T: fmt::Debug> ScopedTable<T> {
    pub(crate) fn new() -> Self {
        Self {
            items: Vec::new(),
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

    pub(crate) fn insert(&mut self, name: String, item: T) -> usize {
        let index = self.items.len();
        self.items.push(item);
        if let Some(scope) = self.scopes.last_mut() {
            scope.table.insert(name, index);
        }
        index
    }

    pub(crate) fn find_in_current_scope(&self, name: &str) -> Option<&T> {
        if let Some(scope) = self.scopes.last()
            && let Some(&index) = scope.table.get(name)
        {
            return self.items.get(index);
        }
        None
    }

    pub(crate) fn find(&self, name: &str) -> Option<&T> {
        for scope in self.scopes.iter().rev() {
            if let Some(index) = scope.table.get(name) {
                return self.items.get(*index);
            }
        }
        None
    }
}

pub(crate) struct FlatTable<T: fmt::Debug> {
    pub(crate) items: Vec<T>,      // 全てのシンボルのリスト
    table: HashMap<String, usize>, // パラメータ名からインデックスへのマッピング
}

impl<T: fmt::Debug> Default for FlatTable<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: fmt::Debug> fmt::Debug for FlatTable<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:?}", item)?;
        }
        Ok(())
    }
}

impl<T: fmt::Debug> FlatTable<T> {
    pub(crate) fn new() -> Self {
        Self {
            items: Vec::new(),
            table: HashMap::new(),
        }
    }

    pub(crate) fn insert(&mut self, name: String, item: T) -> usize {
        let index = self.items.len();
        self.items.push(item);
        self.table.insert(name, index);
        index
    }

    pub(crate) fn find(&self, name: &str) -> Option<&T> {
        if let Some(&index) = self.table.get(name) {
            return self.items.get(index);
        }
        None
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }
}
