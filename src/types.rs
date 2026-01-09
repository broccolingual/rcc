mod kind;
mod spec;
mod table;
mod type_ref;

pub(crate) use kind::*;
pub(crate) use spec::*;
pub(crate) use table::*;
pub(crate) use type_ref::*;

use crate::utils::AlignUp;
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
pub(crate) struct TypeData {
    kind: TypeKind,
    size: usize,
    align: usize,
    attr: TypeAttr,
    storage_class: Option<StorageClassKind>,
}

impl fmt::Debug for TypeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(sc) = &self.storage_class {
            write!(f, "{} ", sc)?;
        }
        write!(f, "{:?}{:?}", self.attr, self.kind)
    }
}

impl Default for TypeData {
    fn default() -> Self {
        TypeData::from_kind(TypeKind::Void, TypeAttr::default(), None)
    }
}

impl TypeData {
    fn from_kind(
        mut kind: TypeKind,
        attr: TypeAttr,
        storage_class: Option<StorageClassKind>,
    ) -> Self {
        let (size, align) = match &mut kind {
            TypeKind::Void => (0, 0),
            TypeKind::Char => (1, 1),
            TypeKind::Short => (2, 2),
            TypeKind::Int => (4, 4),
            TypeKind::Long => (8, 8),
            TypeKind::Float => (4, 4),
            TypeKind::Double => (8, 8),
            TypeKind::Ptr { .. } => (8, 8),
            TypeKind::Array {
                base,
                size: array_size,
            } => {
                let base_data = base.get();
                (base_data.size * *array_size, base_data.align)
            }
            TypeKind::Struct { members, .. } => {
                let mut offset = 0;
                let mut max_align = 1;
                for member in members {
                    let member_align = member.ty.align_of();
                    offset = offset.align_up(member_align);
                    member.offset = Some(offset);
                    offset += member.ty.size_of();
                    if member_align > max_align {
                        max_align = member_align;
                    }
                }
                (offset.align_up(max_align), max_align)
            }
            TypeKind::Union { members, .. } => {
                let mut max_size = 0;
                let mut max_align = 1;
                for member in members {
                    let member_size = member.ty.size_of();
                    let member_align = member.ty.align_of();
                    if member_size > max_size {
                        max_size = member_size;
                    }
                    if member_align > max_align {
                        max_align = member_align;
                    }
                    member.offset = Some(0); // 共用体のメンバーのオフセットは常に0
                }
                (max_size.align_up(max_align), max_align)
            }
            TypeKind::Func { .. } => (8, 8),
        };

        TypeData {
            kind,
            size,
            align,
            attr,
            storage_class,
        }
    }
}
