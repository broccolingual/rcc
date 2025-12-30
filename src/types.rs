mod kind;
mod specifier;

pub(crate) use kind::*;
pub(crate) use specifier::*;

use crate::node::Node;
use core::fmt;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Type {
    pub(crate) kind: TypeKind,
    size: usize,
    align: usize,
    pub(crate) is_const: bool,
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

impl Default for Type {
    fn default() -> Self {
        Type::from(TypeKind::Void, false)
    }
}

impl Type {
    pub(crate) fn from(kind: TypeKind, is_const: bool) -> Self {
        match kind {
            TypeKind::Void => Type {
                kind: TypeKind::Void,
                size: 0,
                align: 0,
                is_const,
            },
            TypeKind::Char => Type {
                kind: TypeKind::Char,
                size: 1,
                align: 1,
                is_const,
            },
            TypeKind::Short => Type {
                kind: TypeKind::Short,
                size: 2,
                align: 2,
                is_const,
            },
            TypeKind::Int => Type {
                kind: TypeKind::Int,
                size: 4,
                align: 4,
                is_const,
            },
            TypeKind::Long => Type {
                kind: TypeKind::Long,
                size: 8,
                align: 8,
                is_const,
            },
            TypeKind::Float => Type {
                kind: TypeKind::Float,
                size: 4,
                align: 4,
                is_const,
            },
            TypeKind::Double => Type {
                kind: TypeKind::Double,
                size: 8,
                align: 8,
                is_const,
            },
            TypeKind::Ptr { ref to } => Type {
                kind: TypeKind::Ptr { to: to.clone() },
                size: 8,
                align: 8,
                is_const,
            },
            TypeKind::Array { ref base, size } => Type {
                kind: TypeKind::Array {
                    base: base.clone(),
                    size,
                },
                size: base.size * size,
                align: base.align,
                is_const,
            },
            TypeKind::Struct {
                ref name,
                ref members,
            } => {
                let mut offset = 0;
                let mut max_align = 1;
                let mut members = members.clone();
                for member in members.iter_mut() {
                    let a = member.ty.align_of();
                    offset = offset.align_up(a); // メンバーのアラインメントに合わせてオフセットを調整
                    member.offset = Some(offset); // メンバーの相対オフセットを設定
                    offset += member.ty.size_of(); // メンバーのサイズ分オフセットを進める
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
            TypeKind::Func {
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
    pub(crate) fn from_ds(declaration_specifiers: Vec<DeclarationSpecifier>) -> Option<Self> {
        let mut ty = Type::default();
        let mut has_type_specifier = false;
        for specifier in declaration_specifiers {
            match specifier {
                DeclarationSpecifier::TypeSpecifierQualifier(tsq) => match tsq {
                    TypeSpecifierQualifier::TypeQualifier(tq) => {
                        if tq == TypeQualifierKind::Const {
                            ty.is_const = true;
                        }
                    }
                    TypeSpecifierQualifier::TypeSpecifier(ty_kind) => {
                        ty = Type::from(ty_kind, ty.is_const);
                        has_type_specifier = true;
                    }
                },
                DeclarationSpecifier::StorageClassSpecifier(_) => {}
                DeclarationSpecifier::FunctionSpecifier(_) => {}
            }
        }
        if has_type_specifier { Some(ty) } else { None }
    }

    pub(crate) fn from_tsq(type_specifier_qualifiers: Vec<TypeSpecifierQualifier>) -> Option<Self> {
        let mut ty = Type::default();
        let mut has_type_specifier = false;
        for specifier in type_specifier_qualifiers {
            match specifier {
                TypeSpecifierQualifier::TypeQualifier(tq) => {
                    if tq == TypeQualifierKind::Const {
                        ty.is_const = true;
                    }
                }
                TypeSpecifierQualifier::TypeSpecifier(ty_kind) => {
                    ty = Type::from(ty_kind, ty.is_const);
                    has_type_specifier = true;
                }
            }
        }
        if has_type_specifier { Some(ty) } else { None }
    }

    // ポインタもしくは配列の指している型を取得
    pub(crate) fn base_type(&self) -> &Type {
        match &self.kind {
            TypeKind::Ptr { to } => to,
            TypeKind::Array { base, .. } => base,
            _ => self,
        }
    }

    // 型がポインタかどうか
    pub(crate) fn is_ptr(&self) -> bool {
        matches!(&self.kind, TypeKind::Ptr { .. })
    }

    // 型が配列かどうか
    pub(crate) fn is_array(&self) -> bool {
        matches!(&self.kind, TypeKind::Array { .. })
    }

    // 型が整数型かどうか
    pub(crate) fn is_integer(&self) -> bool {
        matches!(
            &self.kind,
            TypeKind::Char | TypeKind::Short | TypeKind::Int | TypeKind::Long
        )
    }

    // 型が浮動小数点型かどうか
    pub(crate) fn is_floating_point(&self) -> bool {
        matches!(&self.kind, TypeKind::Float | TypeKind::Double)
    }

    // 型がスカラー型かどうか（整数型または浮動小数点型）
    pub(crate) fn is_scalar(&self) -> bool {
        self.is_integer() || self.is_floating_point()
    }

    // 型が構造体かどうか
    pub(crate) fn is_struct(&self) -> bool {
        matches!(&self.kind, TypeKind::Struct { .. })
    }

    // 構造体メンバーの検索
    pub(crate) fn find_struct_member(&self, name: &str) -> Option<&MemberDeclaration> {
        if let TypeKind::Struct { members, .. } = &self.kind {
            for member in members {
                if member.name == name {
                    return Some(member);
                }
            }
        }
        None
    }

    // 型の実際のサイズ
    pub(crate) fn size_of(&self) -> usize {
        self.size
    }

    // 型のアラインメント
    pub(crate) fn align_of(&self) -> usize {
        self.align
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Declaration {
    pub(crate) name: String,
    pub(crate) ty: Type,
    pub(crate) init: Vec<Node>,
    pub(crate) span: (usize, usize),
}

impl fmt::Debug for Declaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} {}", self.ty, self.name)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct MemberDeclaration {
    pub(crate) name: String,
    pub(crate) ty: Type,
    pub(crate) offset: Option<usize>,
    pub(crate) span: (usize, usize),
}

impl From<Declaration> for MemberDeclaration {
    fn from(decl: Declaration) -> Self {
        MemberDeclaration {
            name: decl.name,
            ty: decl.ty,
            offset: None,
            span: decl.span,
        }
    }
}

impl fmt::Debug for MemberDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} {}", self.ty, self.name)
    }
}

pub(crate) trait AlignUp {
    fn align_up(&self, align: usize) -> usize;
}

impl AlignUp for usize {
    // alignの倍数に切り上げる
    fn align_up(&self, align: usize) -> usize {
        (*self + align - 1) & !(align - 1)
    }
}
