mod table;

use crate::function::Func;
use crate::node::Node;
use crate::types::Type;
use core::fmt;
use std::cell::RefCell;
use std::rc::Rc;
pub(crate) use table::*;

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum SymbolKind {
    Var,
    Func,
    // Tag, // struct, union, enum
}

#[derive(PartialEq, Eq)]
pub(crate) struct Symbol {
    pub(crate) name: String,
    kind: SymbolKind,
    pub(crate) ty: Type,
    owner: Option<Rc<RefCell<Func>>>, // 所有する関数のインデックス
    pub(crate) init: Vec<Node>,
    pub(crate) is_defined: bool, // 定義されているかどうか
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {:?}", self.name, self.ty)
    }
}

impl Symbol {
    pub(crate) fn new(
        name: &str,
        kind: SymbolKind,
        ty: Type,
        owner: Option<Rc<RefCell<Func>>>,
        init: Vec<Node>,
        is_defined: bool,
    ) -> Self {
        Self {
            name: name.to_string(),
            kind,
            ty,
            owner,
            init,
            is_defined,
        }
    }

    pub(crate) fn new_func(name: &str, ty: Type, is_defined: bool) -> Self {
        Self {
            name: name.to_string(),
            kind: SymbolKind::Func,
            ty,
            owner: None,
            init: Vec::new(),
            is_defined,
        }
    }

    pub(crate) fn get_owner(&self) -> Option<Rc<RefCell<Func>>> {
        self.owner.clone()
    }

    pub(crate) fn is_var(&self) -> bool {
        self.kind == SymbolKind::Var
    }

    pub(crate) fn is_global_var(&self) -> bool {
        self.is_var() && self.owner.is_none()
    }

    pub(crate) fn is_func(&self) -> bool {
        self.kind == SymbolKind::Func
    }
}
