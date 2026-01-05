mod table;

pub(crate) use table::*;

use crate::function::FuncId;
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
        Self {
            name: name.to_string(),
            ty,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct SymbolId(pub usize);

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum SymbolKind {
    Var,
    Func,
    EnumConst,
}

#[derive(PartialEq, Eq)]
pub(crate) struct Symbol {
    pub(crate) name: String,
    kind: SymbolKind,
    pub(crate) ty: TypeRef,
    owner: Option<FuncId>,
    pub(crate) init: Vec<Node>,
    value: Option<i64>,          // 列挙定数の値
    pub(crate) is_defined: bool, // 定義されているかどうか
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_enum_const() {
            write!(f, "{} = {}", self.name, self.value.unwrap())
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
        Self {
            name: name.to_string(),
            kind: SymbolKind::Var,
            ty,
            owner,
            init,
            value: None,
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
            value: None,
            is_defined,
        }
    }

    pub(crate) fn new_enum_const(name: &str, value: i64) -> Self {
        // 列挙定数の型は int とする
        let ty = TypeRef::register(TypeKind::Int, TypeAttr::default(), None);
        Self {
            name: name.to_string(),
            kind: SymbolKind::EnumConst,
            ty,
            owner: None,
            init: Vec::new(),
            value: Some(value),
            is_defined: true,
        }
    }

    pub(crate) fn get_owner(&self) -> Option<FuncId> {
        self.owner
    }

    pub(crate) fn get_value(&self) -> Option<i64> {
        self.value
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

    pub(crate) fn is_enum_const(&self) -> bool {
        self.kind == SymbolKind::EnumConst
    }
}
