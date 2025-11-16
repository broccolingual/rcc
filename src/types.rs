use core::fmt;

use crate::ast::Var;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeclarationSpecifier {
    StorageClassSpecifier(StorageClassKind),
    TypeSpecifierQualifier(TypeSpecifierQualifier),
    FunctionSpecifier(FunctionKind),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeSpecifierQualifier {
    TypeSpecifier(TypeKind),
    TypeQualifier(TypeQualifierKind),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FunctionKind {
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
    pub fn all() -> Vec<FunctionKind> {
        vec![FunctionKind::Inline]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StorageClassKind {
    Auto,
    Constexpr,
    Extern,
    Register,
    Static,
    ThreadLocal,
    Typedef,
}

impl fmt::Display for StorageClassKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageClassKind::Auto => write!(f, "auto"),
            StorageClassKind::Constexpr => write!(f, "constexpr"),
            StorageClassKind::Extern => write!(f, "extern"),
            StorageClassKind::Register => write!(f, "register"),
            StorageClassKind::Static => write!(f, "static"),
            StorageClassKind::ThreadLocal => write!(f, "thread_local"),
            StorageClassKind::Typedef => write!(f, "typedef"),
        }
    }
}

impl StorageClassKind {
    pub fn all() -> Vec<StorageClassKind> {
        vec![
            StorageClassKind::Auto,
            StorageClassKind::Constexpr,
            StorageClassKind::Extern,
            StorageClassKind::Register,
            StorageClassKind::Static,
            StorageClassKind::ThreadLocal,
            StorageClassKind::Typedef,
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeQualifierKind {
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
    pub fn all() -> Vec<TypeQualifierKind> {
        vec![
            TypeQualifierKind::Const,
            TypeQualifierKind::Volatile,
            TypeQualifierKind::Restrict,
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeKind {
    Void,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
    Bool,
    Ptr {
        to: Box<Type>,
    }, // to: ポインタの指す型
    Array {
        base: Box<Type>,
        size: usize,
    }, // base: 配列の要素型, size: 要素数
    Func {
        return_ty: Box<Type>,
        params: Vec<Var>,
    }, // return_ty: 戻り値の型, params: パラメータリスト
}

impl fmt::Display for TypeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeKind::Void => write!(f, "void"),
            TypeKind::Char => write!(f, "char"),
            TypeKind::Short => write!(f, "short"),
            TypeKind::Int => write!(f, "int"),
            TypeKind::Long => write!(f, "long"),
            TypeKind::Float => write!(f, "float"),
            TypeKind::Double => write!(f, "double"),
            TypeKind::Bool => write!(f, "bool"),
            TypeKind::Ptr { to } => write!(f, "ptr to {:?}", to),
            TypeKind::Array { base, size } => write!(f, "array[{}] of {:?}", size, base),
            TypeKind::Func { return_ty, params } => {
                write!(f, "func({:?}) -> {:?}", params, return_ty)
            }
        }
    }
}

impl TypeKind {
    pub fn all() -> Vec<TypeKind> {
        vec![
            TypeKind::Void,
            TypeKind::Char,
            TypeKind::Short,
            TypeKind::Int,
            TypeKind::Long,
            TypeKind::Float,
            TypeKind::Double,
            TypeKind::Bool,
        ]
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Type {
    pub kind: TypeKind,
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            TypeKind::Ptr { to } => {
                write!(f, "*{:?}", to)
            }
            TypeKind::Array { base, size } => {
                write!(f, "[{:?}; {}]", base, size)
            }
            TypeKind::Func { return_ty, params } => {
                write!(f, "fn({:?}) -> {:?}", params, return_ty)
            }
            _ => write!(f, "{:?}", self.kind),
        }
    }
}

impl Type {
    pub fn new(kind: TypeKind) -> Self {
        Type { kind }
    }

    // TODO: constやvolatileの情報も扱う
    pub fn from(declaration_specifiers: &Vec<DeclarationSpecifier>) -> Option<Self> {
        for specifier in declaration_specifiers {
            if let DeclarationSpecifier::TypeSpecifierQualifier(tsq) = specifier
                && let TypeSpecifierQualifier::TypeSpecifier(type_kind) = tsq
            {
                return Some(Type::new(type_kind.clone()));
            }
        }
        None
    }

    pub fn new_ptr(to: &Type) -> Self {
        Type {
            kind: TypeKind::Ptr {
                to: Box::new(to.clone()),
            },
        }
    }

    pub fn new_array(of: &Type, size: usize) -> Self {
        Type {
            kind: TypeKind::Array {
                base: Box::new(of.clone()),
                size,
            },
        }
    }

    pub fn new_func(return_ty: &Type, params: Vec<Var>) -> Self {
        Type {
            kind: TypeKind::Func {
                return_ty: Box::new(return_ty.clone()),
                params,
            },
        }
    }

    pub fn is_ptr(&self) -> bool {
        matches!(self.kind, TypeKind::Ptr { .. })
    }

    // 実際のサイズ（配列の場合は要素数を考慮）
    pub fn actual_size_of(&self) -> i64 {
        match &self.kind {
            TypeKind::Array { base, size } => base.actual_size_of() * *size as i64,
            _ => self.size_of(),
        }
    }

    // 型のサイズ（配列の場合は要素のサイズ）
    pub fn size_of(&self) -> i64 {
        match &self.kind {
            TypeKind::Void => 0,
            TypeKind::Char => 1,
            TypeKind::Short => 2,
            TypeKind::Int => 4,
            TypeKind::Long => 8,
            TypeKind::Float => 4,
            TypeKind::Double => 8,
            TypeKind::Bool => 1,
            TypeKind::Ptr { .. } => 8,
            TypeKind::Array { base, .. } => base.size_of(),
            TypeKind::Func { .. } => 8, // TODO: 一旦8バイト固定
        }
    }
}
