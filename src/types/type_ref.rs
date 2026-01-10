use super::{
    DeclSpec, StorageClassKind, TypeAttr, TypeData, TypeKind, TypeQualKind, TypeSpecQual,
    get_type_table,
};
use crate::decl::MemberDecl;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) struct TypeRef(pub usize);

impl TypeRef {
    pub(crate) fn register(
        kind: TypeKind,
        attr: TypeAttr,
        storage_class: Option<StorageClassKind>,
    ) -> Self {
        let data = TypeData::from_kind(kind, attr, storage_class);
        let table = get_type_table();
        let mut table = table.write().unwrap();
        table.register(data)
    }

    pub(crate) fn get(&self) -> TypeData {
        let table = get_type_table();
        let table = table.read().unwrap();
        table.get(*self).clone()
    }

    fn set(&self, data: TypeData) {
        let table = get_type_table();
        let mut table = table.write().unwrap();
        table.set(*self, data);
    }

    pub(crate) fn kind(&self) -> TypeKind {
        let table = get_type_table().read().unwrap();
        table.get(*self).kind.clone()
    }

    pub(crate) fn attr(&self) -> TypeAttr {
        let table = get_type_table().read().unwrap();
        table.get(*self).attr
    }

    pub(crate) fn storage_class(&self) -> Option<StorageClassKind> {
        let table = get_type_table().read().unwrap();
        table.get(*self).storage_class
    }

    pub(crate) fn size_of(&self) -> usize {
        let table = get_type_table().read().unwrap();
        table.get(*self).size
    }

    pub(crate) fn align_of(&self) -> usize {
        let table = get_type_table().read().unwrap();
        table.get(*self).align
    }

    pub(crate) fn is_incomplete(&self) -> bool {
        self.kind().is_incomplete()
    }

    pub(crate) fn is_struct_or_union(&self) -> bool {
        matches!(&self.kind(), TypeKind::Struct { .. } | TypeKind::Union { .. })
    }

    pub(crate) fn is_extern(&self) -> bool {
        matches!(self.storage_class(), Some(StorageClassKind::Extern))
    }

    pub(crate) fn is_typedef(&self) -> bool {
        matches!(self.storage_class(), Some(StorageClassKind::Typedef))
    }

    pub(crate) fn is_ptr(&self) -> bool {
        matches!(&self.kind(), TypeKind::Ptr { .. })
    }

    pub(crate) fn is_array(&self) -> bool {
        matches!(&self.kind(), TypeKind::Array { .. })
    }

    pub(crate) fn is_integer(&self) -> bool {
        matches!(&self.kind(), TypeKind::Char | TypeKind::Short | TypeKind::Int | TypeKind::Long)
    }

    pub(crate) fn is_floating_point(&self) -> bool {
        matches!(&self.kind(), TypeKind::Float | TypeKind::Double)
    }

    pub(crate) fn is_scalar(&self) -> bool {
        self.is_integer() || self.is_floating_point() || self.is_ptr()
    }

    pub(crate) fn is_void(&self) -> bool {
        matches!(&self.kind(), TypeKind::Void)
    }

    pub(crate) fn base_type(&self) -> TypeRef {
        match &self.kind() {
            TypeKind::Ptr { to } => *to,
            TypeKind::Array { base, .. } => *base,
            _ => *self,
        }
    }

    pub(crate) fn find_struct_or_union_member(&self, name: &str) -> Option<MemberDecl> {
        if let TypeKind::Struct { members, .. } | TypeKind::Union { members, .. } = &self.kind() {
            for member in members {
                if member.name == name {
                    return Some(member.clone());
                }
            }
        }
        None
    }

    pub(crate) fn from_ds(decl_specs: Vec<DeclSpec>) -> Option<Self> {
        let mut attr = TypeAttr::default();
        let mut ty_ref: Option<TypeRef> = None;
        let mut storage_class = None;
        for spec in decl_specs {
            match spec {
                DeclSpec::TypeQual(tq_kind) => match tq_kind {
                    TypeQualKind::Const => attr.is_const = true,
                    TypeQualKind::Volatile => attr.is_volatile = true,
                    TypeQualKind::Restrict => attr.is_restrict = true,
                },
                DeclSpec::TypeSpec(ty) => {
                    if ty_ref.is_some() {
                        return None; // すでに型指定子があった場合は無効
                    }
                    ty_ref = Some(ty);
                }
                DeclSpec::StorageClassSpec(sc_kind) => {
                    if storage_class.is_some() {
                        return None; // すでに記憶クラス指定子があった場合は無効
                    }
                    storage_class = Some(sc_kind);
                }
                DeclSpec::FuncSpec(_) => {}
            }
        }
        if let Some(ty) = ty_ref {
            // 型修飾子を既存の型に追加
            let merged_attr = TypeAttr {
                is_const: attr.is_const || ty.attr().is_const,
                is_volatile: attr.is_volatile || ty.attr().is_volatile,
                is_restrict: attr.is_restrict || ty.attr().is_restrict,
            };
            Some(TypeRef::register(ty.kind(), merged_attr, storage_class))
        } else {
            None
        }
    }

    pub(crate) fn from_tsq(type_spec_quals: Vec<TypeSpecQual>) -> Option<Self> {
        let mut attr = TypeAttr::default();
        let mut ty_ref: Option<TypeRef> = None;
        for spec in type_spec_quals {
            match spec {
                TypeSpecQual::TypeQual(tq_kind) => match tq_kind {
                    TypeQualKind::Const => attr.is_const = true,
                    TypeQualKind::Volatile => attr.is_volatile = true,
                    TypeQualKind::Restrict => attr.is_restrict = true,
                },
                TypeSpecQual::TypeSpec(ty) => {
                    if ty_ref.is_some() {
                        return None; // すでに型指定子があった場合は無効
                    }
                    ty_ref = Some(ty);
                }
            }
        }
        if let Some(ty) = ty_ref {
            // 型修飾子を既存の型に追加
            let merged_attr = TypeAttr {
                is_const: attr.is_const || ty.attr().is_const,
                is_volatile: attr.is_volatile || ty.attr().is_volatile,
                is_restrict: attr.is_restrict || ty.attr().is_restrict,
            };
            Some(TypeRef::register(ty.kind(), merged_attr, None))
        } else {
            None
        }
    }

    pub(crate) fn complete_array(&self, size: usize) -> TypeRef {
        if let TypeKind::Array { base, .. } = self.kind() {
            let data = TypeData::from_kind(
                TypeKind::Array { base, size },
                self.attr(),
                self.storage_class(),
            );
            self.set(data);
            *self
        } else {
            *self
        }
    }

    pub(crate) fn complete_struct_or_union(&self, members: Vec<MemberDecl>) -> TypeRef {
        let data = match self.kind() {
            TypeKind::Struct { name, .. } => TypeData::from_kind(
                TypeKind::Struct { name, members },
                self.attr(),
                self.storage_class(),
            ),
            TypeKind::Union { name, .. } => TypeData::from_kind(
                TypeKind::Union { name, members },
                self.attr(),
                self.storage_class(),
            ),
            _ => return *self,
        };
        self.set(data);
        *self
    }
}
