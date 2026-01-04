use super::{TypeAttr, TypeData, TypeKind, TypeRef};
use std::sync::{OnceLock, RwLock};

static TYPE_TABLE: OnceLock<RwLock<TypeTable>> = OnceLock::new();

pub(crate) fn get_type_table() -> &'static RwLock<TypeTable> {
    TYPE_TABLE.get_or_init(|| RwLock::new(TypeTable::new()))
}

pub(crate) struct TypeTable {
    types: Vec<TypeData>,
}

impl TypeTable {
    fn new() -> Self {
        let mut table = TypeTable { types: Vec::new() };

        // 基本型を事前登録
        table.register_primitive(TypeKind::Void, 0, 0);
        table.register_primitive(TypeKind::Char, 1, 1);
        table.register_primitive(TypeKind::Short, 2, 2);
        table.register_primitive(TypeKind::Int, 4, 4);
        table.register_primitive(TypeKind::Long, 8, 8);
        table.register_primitive(TypeKind::Float, 4, 4);
        table.register_primitive(TypeKind::Double, 8, 8);

        table
    }

    fn register_primitive(&mut self, kind: TypeKind, size: usize, align: usize) -> TypeRef {
        let data = TypeData {
            kind,
            size,
            align,
            attr: TypeAttr::default(),
            storage_class: None,
        };
        let id = TypeRef(self.types.len());
        self.types.push(data.clone());
        id
    }

    pub(crate) fn register(&mut self, data: TypeData) -> TypeRef {
        // 既に登録されている型があればそれを返す
        if let Some(pos) = self.types.iter().position(|existing| existing == &data) {
            return TypeRef(pos);
        }

        let id = TypeRef(self.types.len());
        self.types.push(data);
        id
    }

    pub(crate) fn get(&self, id: TypeRef) -> &TypeData {
        &self.types[id.0]
    }

    pub(crate) fn get_all_types(&self) -> &Vec<TypeData> {
        &self.types
    }

    pub(crate) fn set(&mut self, id: TypeRef, data: TypeData) {
        self.types[id.0] = data;
    }
}
