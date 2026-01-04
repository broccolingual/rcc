mod table;

pub(crate) use table::*;

use crate::function::FuncId;
use crate::node::Node;
use crate::types::TypeRef;
use core::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct SymbolId(pub usize);

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum SymbolKind {
    Var,
    Func,
    // Tag, // struct, union, enum
}

#[derive(PartialEq, Eq)]
pub(crate) struct Symbol {
    pub(crate) name: String,
    kind: SymbolKind,
    pub(crate) ty: TypeRef,
    owner: Option<FuncId>,
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
        ty: TypeRef,
        owner: Option<FuncId>,
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

    pub(crate) fn new_func(name: &str, ty: TypeRef, is_defined: bool) -> Self {
        Self {
            name: name.to_string(),
            kind: SymbolKind::Func,
            ty,
            owner: None,
            init: Vec::new(),
            is_defined,
        }
    }

    pub(crate) fn get_owner(&self) -> Option<FuncId> {
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
