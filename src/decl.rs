use crate::node::Node;
use crate::types::TypeRef;
use crate::utils::Span;
use core::fmt;

#[derive(Clone)]
pub(crate) struct Decl {
    pub(crate) name: String,
    pub(crate) ty: TypeRef,
    pub(crate) init: Vec<Node>,
    pub(crate) span: Span,
}

impl Decl {
    pub(crate) fn new(name: String, ty: TypeRef, span: Span) -> Self {
        Decl { name, ty, init: Vec::new(), span }
    }

    pub(crate) fn new_abst(ty: TypeRef, span: Span) -> Self {
        Decl { name: String::new(), ty, init: Vec::new(), span }
    }
}

impl fmt::Debug for Decl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} {}", self.ty.get(), self.name)
    }
}

// init と span を比較から除外
impl PartialEq for Decl {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.ty == other.ty
        // init と span は型の同一性に影響しない
    }
}

impl Eq for Decl {}

#[derive(Clone)]
pub(crate) struct MemberDecl {
    pub(crate) name: String,
    pub(crate) ty: TypeRef,
    pub(crate) offset: Option<usize>,
}

impl From<Decl> for MemberDecl {
    fn from(decl: Decl) -> Self {
        MemberDecl { name: decl.name, ty: decl.ty, offset: None }
    }
}

impl fmt::Debug for MemberDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} {} @{:?}", self.ty.get(), self.name, self.offset)
    }
}

// offset を比較から除外
impl PartialEq for MemberDecl {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.ty == other.ty
        // offset は無視
    }
}

impl Eq for MemberDecl {}
