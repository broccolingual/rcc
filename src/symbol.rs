mod table;

use crate::node::Node;
use crate::types::Type;
use core::fmt;
pub(crate) use table::*;

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum SymbolKind {
    Var,
    Func,
    // Tag, // struct, union, enum
}

pub(crate) struct Symbol {
    pub(crate) name: String,
    kind: SymbolKind,
    pub(crate) ty: Type,
    owner: Option<usize>, // Funcのインデックス
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
        owner: Option<usize>,
        init: Vec<Node>,
    ) -> Self {
        Self {
            name: name.to_string(),
            kind,
            ty,
            owner,
            init,
            is_defined: true,
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

    pub(crate) fn get_owner_func_id(&self) -> Option<usize> {
        self.owner
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
