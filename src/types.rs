mod kind;
mod specifier;

pub(crate) use kind::*;
pub(crate) use specifier::*;

use crate::node::Node;
use core::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct TypeAttr {
    pub(crate) is_const: bool,
    pub(crate) is_volatile: bool,
    pub(crate) is_restrict: bool,
}

impl fmt::Debug for TypeAttr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut attrs = Vec::new();
        if self.is_const {
            attrs.push("const ");
        }
        if self.is_volatile {
            attrs.push("volatile ");
        }
        if self.is_restrict {
            attrs.push("restrict ");
        }
        write!(f, "{}", attrs.concat())
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Type {
    pub(crate) kind: TypeKind,
    size: usize,
    align: usize,
    pub(crate) attr: TypeAttr,
    pub(crate) storage_class: Option<StorageClassKind>,
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(sc) = &self.storage_class {
            write!(f, "{} ", sc)?;
        }
        write!(f, "{:?}{:?}", self.attr, self.kind)
    }
}

impl Default for Type {
    fn default() -> Self {
        Type::from(TypeKind::Void, TypeAttr::default(), None)
    }
}

impl Type {
    pub(crate) fn from(
        kind: TypeKind,
        attr: TypeAttr,
        storage_class: Option<StorageClassKind>,
    ) -> Self {
        match kind {
            TypeKind::Void => Type {
                kind: TypeKind::Void,
                size: 0,
                align: 0,
                attr,
                storage_class,
            },
            TypeKind::Char => Type {
                kind: TypeKind::Char,
                size: 1,
                align: 1,
                attr,
                storage_class,
            },
            TypeKind::Short => Type {
                kind: TypeKind::Short,
                size: 2,
                align: 2,
                attr,
                storage_class,
            },
            TypeKind::Int => Type {
                kind: TypeKind::Int,
                size: 4,
                align: 4,
                attr,
                storage_class,
            },
            TypeKind::Long => Type {
                kind: TypeKind::Long,
                size: 8,
                align: 8,
                attr,
                storage_class,
            },
            TypeKind::Float => Type {
                kind: TypeKind::Float,
                size: 4,
                align: 4,
                attr,
                storage_class,
            },
            TypeKind::Double => Type {
                kind: TypeKind::Double,
                size: 8,
                align: 8,
                attr,
                storage_class,
            },
            TypeKind::Ptr { to } => Type {
                kind: TypeKind::Ptr { to: to.clone() },
                size: 8,
                align: 8,
                attr,
                storage_class,
            },
            TypeKind::Array { base, size } => Type {
                kind: TypeKind::Array {
                    base: base.clone(),
                    size,
                },
                size: base.size * size,
                align: base.align,
                attr,
                storage_class,
            },
            TypeKind::Struct { name, members } => {
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
                    attr,
                    storage_class,
                }
            }
            TypeKind::Func { return_ty, params } => Type {
                kind: TypeKind::Func {
                    return_ty: return_ty.clone(),
                    params: params.clone(),
                },
                size: 8,
                align: 8,
                attr,
                storage_class,
            },
        }
    }

    pub(crate) fn from_ds(decl_specs: Vec<DeclSpec>) -> Option<Self> {
        let mut ty = Type::default();
        let mut has_type_spec = false;
        let mut storage_class = None;
        for spec in decl_specs {
            match spec {
                DeclSpec::TypeSpecQual(tsq) => match tsq {
                    TypeSpecQual::TypeQual(tq) => match tq {
                        TypeQualKind::Const => ty.attr.is_const = true,
                        TypeQualKind::Volatile => ty.attr.is_volatile = true,
                        TypeQualKind::Restrict => ty.attr.is_restrict = true,
                    },
                    TypeSpecQual::TypeSpec(ty_kind) => {
                        if has_type_spec {
                            return None; // すでに型指定子があった場合は無効
                        }
                        ty = Type::from(ty_kind, ty.attr, None);
                        has_type_spec = true;
                    }
                },
                DeclSpec::StorageClassSpec(scs) => {
                    storage_class = Some(scs);
                }
                DeclSpec::FuncSpec(_) => {}
            }
        }
        if has_type_spec {
            ty.storage_class = storage_class;
            Some(ty)
        } else {
            None
        }
    }

    pub(crate) fn from_tsq(type_spec_quals: Vec<TypeSpecQual>) -> Option<Self> {
        let mut ty = Type::default();
        let mut has_type_spec = false;
        for spec in type_spec_quals {
            match spec {
                TypeSpecQual::TypeQual(tq) => match tq {
                    TypeQualKind::Const => ty.attr.is_const = true,
                    TypeQualKind::Volatile => ty.attr.is_volatile = true,
                    TypeQualKind::Restrict => ty.attr.is_restrict = true,
                },
                TypeSpecQual::TypeSpec(ty_kind) => {
                    if has_type_spec {
                        return None; // すでに型指定子があった場合は無効
                    }
                    ty = Type::from(ty_kind, ty.attr, None);
                    has_type_spec = true;
                }
            }
        }
        if has_type_spec { Some(ty) } else { None }
    }

    // ポインタもしくは配列の指している型を取得
    pub(crate) fn base_type(&self) -> &Type {
        match &self.kind {
            TypeKind::Ptr { to } => to,
            TypeKind::Array { base, .. } => base,
            _ => self,
        }
    }

    // 型がexternかどうか
    pub(crate) fn is_extern(&self) -> bool {
        matches!(self.storage_class, Some(StorageClassKind::Extern))
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
    pub(crate) fn find_struct_member(&self, name: &str) -> Option<&MemberDecl> {
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
pub(crate) struct Decl {
    pub(crate) name: String,
    pub(crate) ty: Type,
    pub(crate) init: Vec<Node>,
    pub(crate) span: (usize, usize),
}

impl fmt::Debug for Decl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} {}", self.ty, self.name)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct MemberDecl {
    pub(crate) name: String,
    pub(crate) ty: Type,
    pub(crate) offset: Option<usize>,
    pub(crate) span: (usize, usize),
}

impl From<Decl> for MemberDecl {
    fn from(decl: Decl) -> Self {
        MemberDecl {
            name: decl.name,
            ty: decl.ty,
            offset: None,
            span: decl.span,
        }
    }
}

impl fmt::Debug for MemberDecl {
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
