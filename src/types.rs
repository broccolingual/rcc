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
    pub fn all() -> Vec<StorageClassKind> {
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

#[derive(Clone, PartialEq, Eq)]
pub enum TypeKind {
    Void,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
    Ptr {
        to: Box<Type>,
    }, // to: ポインタの指す型
    Array {
        base: Box<Type>,
        size: usize,
    }, // base: 配列の要素型, size: 要素数
    Struct {
        name: String,
        members: Vec<Var>,
    }, // name: 構造体名, members: メンバーリスト
    Func {
        return_ty: Box<Type>,
        params: Vec<Var>,
    }, // return_ty: 戻り値の型, params: パラメータリスト
}

impl fmt::Debug for TypeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeKind::Void => write!(f, "void"),
            TypeKind::Char => write!(f, "char"),
            TypeKind::Short => write!(f, "short"),
            TypeKind::Int => write!(f, "int"),
            TypeKind::Long => write!(f, "long"),
            TypeKind::Float => write!(f, "float"),
            TypeKind::Double => write!(f, "double"),
            // ポインタや配列は再帰的に*をつけて表示
            TypeKind::Ptr { to } => write!(f, "{:?}*", to),
            TypeKind::Array { base, .. } => write!(f, "{:?}*", base),
            TypeKind::Struct { name, members } => write!(f, "struct {} {{ {:?} }}", name, members),
            TypeKind::Func { return_ty, params } => {
                write!(f, "func(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{:?}", param)?;
                }
                write!(f, ") -> {:?}", return_ty)
            }
        }
    }
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
            TypeKind::Ptr { to } => write!(f, "ptr to {:?}", to),
            TypeKind::Array { base, size } => write!(f, "array[{}] of {:?}", size, base),
            TypeKind::Struct { name, members } => {
                write!(f, "struct {} {{ {:?} }}", name, members)
            }
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
        ]
    }
}

pub trait AlignUp {
    fn align_up(&self, align: usize) -> usize;
}

impl AlignUp for usize {
    // alignの倍数に切り上げる
    fn align_up(&self, align: usize) -> usize {
        (*self + align - 1) & !(align - 1)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Type {
    pub kind: TypeKind,
    size: usize,
    align: usize,
    pub is_const: bool,
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_const {
            write!(f, "const {:?}", self.kind)
        } else {
            write!(f, "{:?}", self.kind)
        }
    }
}

impl Type {
    pub fn from(kind: &TypeKind, is_const: bool) -> Self {
        match kind {
            &TypeKind::Void => Type {
                kind: TypeKind::Void,
                size: 0,
                align: 0,
                is_const,
            },
            &TypeKind::Char => Type {
                kind: TypeKind::Char,
                size: 1,
                align: 1,
                is_const,
            },
            &TypeKind::Short => Type {
                kind: TypeKind::Short,
                size: 2,
                align: 2,
                is_const,
            },
            &TypeKind::Int => Type {
                kind: TypeKind::Int,
                size: 4,
                align: 4,
                is_const,
            },
            &TypeKind::Long => Type {
                kind: TypeKind::Long,
                size: 8,
                align: 8,
                is_const,
            },
            &TypeKind::Float => Type {
                kind: TypeKind::Float,
                size: 4,
                align: 4,
                is_const,
            },
            &TypeKind::Double => Type {
                kind: TypeKind::Double,
                size: 8,
                align: 8,
                is_const,
            },
            &TypeKind::Ptr { ref to } => Type {
                kind: TypeKind::Ptr { to: to.clone() },
                size: 8,
                align: 8,
                is_const,
            },
            &TypeKind::Array { ref base, size } => Type {
                kind: TypeKind::Array {
                    base: base.clone(),
                    size,
                },
                size: base.size * size,
                align: base.align,
                is_const,
            },
            &TypeKind::Struct {
                ref name,
                ref members,
            } => {
                let mut offset = 0;
                let mut max_align = 1;
                let mut members = members.clone();
                for member in members.iter_mut() {
                    let a = member.ty.align_of();
                    offset = offset.align_up(a); // メンバーのアラインメントに合わせてオフセットを調整
                    offset += member.ty.size_of(); // メンバーのサイズ分オフセットを進める
                    member.offset = offset; // メンバーのオフセットを設定
                    // 構造体全体のアラインメントを更新
                    if a > max_align {
                        max_align = a;
                    }
                }
                Type {
                    kind: TypeKind::Struct {
                        name: name.to_string(),
                        members,
                    },
                    size: offset.align_up(max_align), // 構造体全体のサイズをアラインメントに合わせて調整
                    align: max_align, // メンバーの最大アラインメントを構造体のアラインメントとする
                    is_const,
                }
            }
            &TypeKind::Func {
                ref return_ty,
                ref params,
            } => Type {
                kind: TypeKind::Func {
                    return_ty: return_ty.clone(),
                    params: params.clone(),
                },
                size: 8,
                align: 8,
                is_const,
            },
        }
    }

    // TODO: constやvolatileの情報も扱う
    pub fn from_ds(declaration_specifiers: &Vec<DeclarationSpecifier>) -> Option<Self> {
        for specifier in declaration_specifiers {
            if let DeclarationSpecifier::TypeSpecifierQualifier(tsq) = specifier
                && let TypeSpecifierQualifier::TypeSpecifier(ty) = tsq
            {
                return Some(Type::from(ty, false));
            }
        }
        None
    }

    pub fn from_tsq(type_specifier_qualifiers: &Vec<TypeSpecifierQualifier>) -> Option<Self> {
        for specifier in type_specifier_qualifiers {
            if let TypeSpecifierQualifier::TypeSpecifier(ty) = specifier {
                return Some(Type::from(ty, false));
            }
        }
        None
    }

    // ポインタもしくは配列の指している型を取得
    pub fn base_type(&self) -> &Type {
        match &self.kind {
            TypeKind::Ptr { to } => to,
            TypeKind::Array { base, .. } => base,
            _ => self,
        }
    }

    // 型が配列かどうか
    pub fn is_array(&self) -> bool {
        matches!(&self.kind, TypeKind::Array { .. })
    }

    // 型がポインタもしくは配列かどうか
    pub fn is_ptr_or_array(&self) -> bool {
        matches!(&self.kind, TypeKind::Ptr { .. } | TypeKind::Array { .. })
    }

    // 型が整数型かどうか
    pub fn is_integer(&self) -> bool {
        matches!(
            &self.kind,
            TypeKind::Char | TypeKind::Short | TypeKind::Int | TypeKind::Long
        )
    }

    // 型が浮動小数点型かどうか
    pub fn is_floating_point(&self) -> bool {
        matches!(&self.kind, TypeKind::Float | TypeKind::Double)
    }

    // 型がスカラー型かどうか（整数型または浮動小数点型）
    pub fn is_scalar(&self) -> bool {
        self.is_integer() || self.is_floating_point()
    }

    // 型の実際のサイズ
    pub fn size_of(&self) -> usize {
        self.size
    }

    // 型のアラインメント
    pub fn align_of(&self) -> usize {
        self.align
    }
}
