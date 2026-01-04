use crate::types::*;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

static TYPE_TABLE: OnceLock<Mutex<TypeTable>> = OnceLock::new();

pub(crate) fn get_type_table() -> &'static Mutex<TypeTable> {
    TYPE_TABLE.get_or_init(|| Mutex::new(TypeTable::new()))
}

pub(crate) struct TypeTable {
    types: Vec<TypeData>,
    type_map: HashMap<TypeData, TypeRef>,
}

impl TypeTable {
    fn new() -> Self {
        let mut table = TypeTable {
            types: Vec::new(),
            type_map: HashMap::new(),
        };

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
        self.type_map.insert(data, id);
        id
    }

    pub(crate) fn register(&mut self, data: TypeData) -> TypeRef {
        // 既に登録されている型か確認
        if let Some(&id) = self.type_map.get(&data) {
            return id;
        }

        let id = TypeRef(self.types.len());
        self.types.push(data.clone());
        self.type_map.insert(data, id);
        id
    }

    pub(crate) fn get(&self, id: TypeRef) -> &TypeData {
        &self.types[id.0]
    }

    pub(crate) fn get_all_types(&self) -> &Vec<TypeData> {
        &self.types
    }
}
