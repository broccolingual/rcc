use crate::types::TypeKind;
use core::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DeclarationSpecifier {
    StorageClassSpecifier(StorageClassKind),
    TypeSpecifierQualifier(TypeSpecifierQualifier),
    FunctionSpecifier(FunctionKind),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TypeSpecifierQualifier {
    TypeSpecifier(TypeKind),
    TypeQualifier(TypeQualifierKind),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FunctionKind {
    Inline,
}

impl fmt::Display for FunctionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FunctionKind::Inline => write!(f, "inline"),
        }
    }
}

impl FunctionKind {
    pub(crate) fn all() -> Vec<FunctionKind> {
        vec![FunctionKind::Inline]
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
pub(crate) enum TypeQualifierKind {
    Const,
    Volatile,
    Restrict,
}

impl fmt::Display for TypeQualifierKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeQualifierKind::Const => write!(f, "const"),
            TypeQualifierKind::Volatile => write!(f, "volatile"),
            TypeQualifierKind::Restrict => write!(f, "restrict"),
        }
    }
}

impl TypeQualifierKind {
    pub(crate) fn all() -> Vec<TypeQualifierKind> {
        vec![
            TypeQualifierKind::Const,
            TypeQualifierKind::Volatile,
            TypeQualifierKind::Restrict,
        ]
    }
}
