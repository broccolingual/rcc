use crate::types::TypeKind;
use core::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DeclSpec {
    StorageClassSpec(StorageClassKind),
    TypeSpecQual(TypeSpecQual),
    FuncSpec(FuncKind),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TypeSpecQual {
    TypeSpec(TypeKind),
    TypeQual(TypeQualKind),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FuncKind {
    Inline,
}

impl fmt::Display for FuncKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FuncKind::Inline => write!(f, "inline"),
        }
    }
}

impl FuncKind {
    pub(crate) fn all() -> Vec<FuncKind> {
        vec![FuncKind::Inline]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum StorageClassKind {
    Auto,
    Extern,
    Register,
    Static,
    Typedef,
}

impl fmt::Display for StorageClassKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageClassKind::Auto => write!(f, "auto"),
            StorageClassKind::Extern => write!(f, "extern"),
            StorageClassKind::Register => write!(f, "register"),
            StorageClassKind::Static => write!(f, "static"),
            StorageClassKind::Typedef => write!(f, "typedef"),
        }
    }
}

impl StorageClassKind {
    pub(crate) fn all() -> Vec<StorageClassKind> {
        vec![
            StorageClassKind::Auto,
            StorageClassKind::Extern,
            StorageClassKind::Register,
            StorageClassKind::Static,
            StorageClassKind::Typedef,
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TypeQualKind {
    Const,
    Volatile,
    Restrict,
}

impl fmt::Display for TypeQualKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeQualKind::Const => write!(f, "const"),
            TypeQualKind::Volatile => write!(f, "volatile"),
            TypeQualKind::Restrict => write!(f, "restrict"),
        }
    }
}

impl TypeQualKind {
    pub(crate) fn all() -> Vec<TypeQualKind> {
        vec![
            TypeQualKind::Const,
            TypeQualKind::Volatile,
            TypeQualKind::Restrict,
        ]
    }
}
