mod table;

pub(crate) use table::*;

use crate::func::FuncId;
use crate::node::Node;
use crate::types::{TypeAttr, TypeKind, TypeRef};
use core::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct TagId(pub usize);

#[derive(PartialEq, Eq)]
pub(crate) struct Tag {
    pub(crate) name: String,
    pub(crate) ty: TypeRef,
}

impl fmt::Debug for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {:?}", self.name, self.ty)
    }
}

impl Tag {
    pub(crate) fn new(name: &str, ty: TypeRef) -> Self {
        Self { name: name.to_string(), ty }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct SymbolId(pub usize);

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum SymbolKind {
    Var { owner: Option<FuncId>, init: Vec<Node>, is_defined: bool },
    Func { is_defined: bool },
    Typedef,
    EnumConst { value: i64 },
}

#[derive(PartialEq, Eq)]
pub(crate) struct Symbol {
    pub(crate) name: String,
    kind: SymbolKind,
    pub(crate) ty: TypeRef,
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(value) = self.get_value() {
            write!(f, "{} = {}", self.name, value) // 列挙定数の場合
        } else {
            write!(f, "{}: {:?}", self.name, self.ty)
        }
    }
}

impl Symbol {
    pub(crate) fn new_var(
        name: &str,
        ty: TypeRef,
        owner: Option<FuncId>,
        init: Vec<Node>,
        is_defined: bool,
    ) -> Self {
        Self { name: name.to_string(), kind: SymbolKind::Var { owner, init, is_defined }, ty }
    }

    pub(crate) fn new_func(name: &str, ty: TypeRef, is_defined: bool) -> Self {
        Self { name: name.to_string(), kind: SymbolKind::Func { is_defined }, ty }
    }

    pub(crate) fn new_enum_const(name: &str, value: i64) -> Self {
        let int_ty = TypeRef::register(TypeKind::Int, TypeAttr::default(), None);
        Self { name: name.to_string(), kind: SymbolKind::EnumConst { value }, ty: int_ty }
    }

    pub(crate) fn new_typedef(name: &str, ty: TypeRef) -> Self {
        Self { name: name.to_string(), kind: SymbolKind::Typedef, ty }
    }

    pub(crate) fn get_owner(&self) -> Option<FuncId> {
        match &self.kind {
            SymbolKind::Var { owner, .. } => *owner,
            _ => None,
        }
    }

    pub(crate) fn get_value(&self) -> Option<i64> {
        match &self.kind {
            SymbolKind::EnumConst { value, .. } => Some(*value),
            _ => None,
        }
    }

    pub(crate) fn get_init(&self) -> &[Node] {
        match &self.kind {
            SymbolKind::Var { init, .. } => init,
            _ => &[],
        }
    }

    pub(crate) fn set_defined(&mut self, defined: bool) {
        match &mut self.kind {
            SymbolKind::Var { is_defined, .. } => *is_defined = defined,
            SymbolKind::Func { is_defined } => *is_defined = defined,
            _ => {}
        }
    }

    pub(crate) fn is_var(&self) -> bool {
        matches!(self.kind, SymbolKind::Var { .. })
    }

    pub(crate) fn is_global_var(&self) -> bool {
        self.is_var() && self.get_owner().is_none()
    }

    pub(crate) fn is_func(&self) -> bool {
        matches!(self.kind, SymbolKind::Func { .. })
    }

    pub(crate) fn is_typedef(&self) -> bool {
        matches!(self.kind, SymbolKind::Typedef { .. })
    }

    pub(crate) fn is_defined(&self) -> bool {
        match &self.kind {
            SymbolKind::Var { is_defined, .. } => *is_defined,
            SymbolKind::Func { is_defined } => *is_defined,
            SymbolKind::Typedef { .. } => true,
            SymbolKind::EnumConst { .. } => true,
        }
    }

    pub(crate) fn is_enum_const(&self) -> bool {
        matches!(self.kind, SymbolKind::EnumConst { .. })
    }
}
