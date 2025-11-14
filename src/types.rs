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
    Ptr,
    Array,
    Func,
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
            TypeKind::Ptr => write!(f, "ptr"),
            TypeKind::Array => write!(f, "array"),
            TypeKind::Func => write!(f, "func"),
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

#[derive(Clone)]
pub struct Type {
    pub kind: TypeKind,
    pub ptr_to: Option<Box<Type>>,
    pub array_size: usize,            // 配列の要素数（配列型の場合）
    pub params: Option<Vec<Var>>,     // 関数型の場合のパラメータリスト
    pub return_ty: Option<Box<Type>>, // 関数型の場合の戻り値の型
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            TypeKind::Ptr => {
                let mut cur = self;
                let mut depth = 0;
                while let Some(ref to) = cur.ptr_to {
                    depth += 1;
                    cur = to;
                }
                write!(f, "{}{:?}", "*".repeat(depth), cur.kind)
            }
            TypeKind::Array => {
                if let Some(ref to) = self.ptr_to {
                    write!(f, "[{:?}; {}]", to, self.array_size)
                } else {
                    write!(f, "[Unknown; {}]", self.array_size)
                }
            }
            TypeKind::Func => {
                let return_ty = if let Some(ref ret_ty) = self.return_ty {
                    format!("{:?}", ret_ty)
                } else {
                    "Unknown".to_string()
                };
                let params = if let Some(ref params) = self.params {
                    let param_strs: Vec<String> =
                        params.iter().map(|p| format!("{:?}", p.ty)).collect();
                    param_strs.join(", ")
                } else {
                    "Unknown".to_string()
                };
                write!(f, "func({}) -> {}", params, return_ty)
            }
            _ => write!(f, "{:?}", self.kind),
        }
    }
}

impl Type {
    pub fn new(kind: TypeKind) -> Self {
        Type {
            kind,
            ptr_to: None,
            array_size: 0,
            params: None,
            return_ty: None,
        }
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
            kind: TypeKind::Ptr,
            ptr_to: Some(Box::new(to.clone())),
            array_size: 0,
            params: None,
            return_ty: None,
        }
    }

    pub fn new_array(of: &Type, size: usize) -> Self {
        Type {
            kind: TypeKind::Array,
            ptr_to: Some(Box::new(of.clone())),
            array_size: size,
            params: None,
            return_ty: None,
        }
    }

    pub fn new_func(return_ty: &Type, params: Vec<Var>) -> Self {
        Type {
            kind: TypeKind::Func,
            ptr_to: None,
            array_size: params.len(),
            params: Some(params),
            return_ty: Some(Box::new(return_ty.clone())),
        }
    }

    pub fn is_ptr(&self) -> bool {
        matches!(self.kind, TypeKind::Ptr)
    }

    pub fn size_of(&self) -> i64 {
        match self.kind {
            TypeKind::Void => 0,
            TypeKind::Char => 1,
            TypeKind::Short => 2,
            TypeKind::Int => 4,
            TypeKind::Long => 8,
            TypeKind::Float => 4,
            TypeKind::Double => 8,
            TypeKind::Bool => 1,
            TypeKind::Ptr => 8,
            TypeKind::Array => {
                if let Some(ref to) = self.ptr_to {
                    to.size_of() * self.array_size as i64
                } else {
                    0
                }
            }
            TypeKind::Func => 8, // TODO: 一旦8バイト固定
        }
    }
}
