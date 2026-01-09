use super::TypeRef;
use crate::decl::{Decl, MemberDecl};
use core::fmt;

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum TypeKind {
    Void,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
    Ptr {
        to: TypeRef,
    }, // to: ポインタの指す型
    Array {
        base: TypeRef,
        size: usize,
    }, // base: 配列の要素型, size: 要素数
    Struct {
        name: String,
        members: Vec<MemberDecl>,
    }, // name: 構造体名, members: メンバーリスト
    Union {
        name: String,
        members: Vec<MemberDecl>,
    }, // name: 共用体名, members: メンバーリスト
    Func {
        return_ty: TypeRef,
        params: Vec<Decl>,
        is_variadic: bool,
    }, // return_ty: 戻り値の型, params: パラメータリスト, is_variadic: 可変長引数かどうか
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
            TypeKind::Ptr { to } => write!(f, "ptr -> {:?}", to),
            TypeKind::Array { base, size } => write!(f, "array[{}] of {:?}", size, base),
            TypeKind::Struct { name, members } => {
                write!(f, "struct {} {{ ", name)?;
                for (i, member) in members.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(
                        f,
                        "TypeRef({}) {} @{:?}",
                        member.ty.0,
                        member.name,
                        member.offset.unwrap_or(0)
                    )?;
                }
                write!(f, " }}")
            }
            TypeKind::Union { name, members } => {
                write!(f, "union {} {{ ", name)?;
                for (i, member) in members.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "TypeRef({}) {}", member.ty.0, member.name,)?;
                }
                write!(f, " }}")
            }
            TypeKind::Func {
                return_ty,
                params,
                is_variadic,
            } => {
                write!(f, "func(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "TypeRef({}) {}", param.ty.0, param.name)?;
                }
                if *is_variadic {
                    write!(f, ", ...")?;
                }
                write!(f, ") -> TypeRef({})", return_ty.0)
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
            _ => write!(f, "{:?}", self),
        }
    }
}

impl TypeKind {
    pub(crate) fn all() -> Vec<TypeKind> {
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

    // 型が未完成型かどうかを判定
    pub(crate) fn is_incomplete(&self) -> bool {
        match self {
            TypeKind::Void => true,                                 // void は常に未完成型
            TypeKind::Array { size, .. } => *size == 0,             // サイズ不明の配列
            TypeKind::Struct { members, .. } => members.is_empty(), // メンバーが定義されていない構造体
            TypeKind::Union { members, .. } => members.is_empty(), // メンバーが定義されていない共用体
            _ => false,
        }
    }
}
