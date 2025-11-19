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
    TypeSpecifier(Type),
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
pub enum Type {
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

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Void => write!(f, "void"),
            Type::Char => write!(f, "char"),
            Type::Short => write!(f, "short"),
            Type::Int => write!(f, "int"),
            Type::Long => write!(f, "long"),
            Type::Float => write!(f, "float"),
            Type::Double => write!(f, "double"),
            Type::Bool => write!(f, "bool"),
            Type::Ptr { to } => write!(f, "ptr to {:?}", to),
            Type::Array { base, size } => write!(f, "array[{}] of {:?}", size, base),
            Type::Func { return_ty, params } => {
                write!(f, "func({:?}) -> {:?}", params, return_ty)
            }
        }
    }
}

impl Type {
    pub fn all() -> Vec<Type> {
        vec![
            Type::Void,
            Type::Char,
            Type::Short,
            Type::Int,
            Type::Long,
            Type::Float,
            Type::Double,
            Type::Bool,
        ]
    }

    // TODO: constやvolatileの情報も扱う
    pub fn from(declaration_specifiers: &Vec<DeclarationSpecifier>) -> Option<Self> {
        for specifier in declaration_specifiers {
            if let DeclarationSpecifier::TypeSpecifierQualifier(tsq) = specifier
                && let TypeSpecifierQualifier::TypeSpecifier(ty) = tsq
            {
                return Some(ty.clone());
            }
        }
        None
    }

    // ポインタもしくは配列の指している型を取得
    pub fn base_type(&self) -> &Type {
        match &self {
            Type::Ptr { to } => to,
            Type::Array { base, .. } => base,
            _ => self,
        }
    }

    // 型がポインタもしくは配列かどうか
    pub fn is_ptr_or_array(&self) -> bool {
        matches!(self, Type::Ptr { .. } | Type::Array { .. })
    }

    // 型が整数型かどうか
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            Type::Char | Type::Short | Type::Int | Type::Long | Type::Bool
        )
    }

    // 型が浮動小数点型かどうか
    pub fn is_floating_point(&self) -> bool {
        matches!(self, Type::Float | Type::Double)
    }

    // 型がスカラー型かどうか（整数型または浮動小数点型）
    pub fn is_scalar(&self) -> bool {
        self.is_integer() || self.is_floating_point()
    }

    // 型の実際のサイズ（配列の場合は要素数を考慮）
    pub fn size_of(&self) -> i64 {
        match &self {
            Type::Array { base, size } => base.size_of() * *size as i64,
            _ => self.align_of(),
        }
    }

    // 型のアラインメント
    pub fn align_of(&self) -> i64 {
        match &self {
            Type::Void => 0,
            Type::Char => 1,
            Type::Short => 2,
            Type::Int => 4,
            Type::Long => 8,
            Type::Float => 4,
            Type::Double => 8,
            Type::Bool => 1,
            Type::Ptr { .. } => 8,
            Type::Array { .. } => 8,
            Type::Func { .. } => 8, // TODO: 一旦8バイト固定
        }
    }
}
