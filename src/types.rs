#[derive(Clone)]
pub enum TypeKind {
    Int,
    Ptr,
}

#[derive(Clone)]
pub struct Type {
    pub kind: TypeKind,
    pub ptr_to: Option<Box<Type>>,
}

impl Type {
    pub fn new_int() -> Self {
        Type {
            kind: TypeKind::Int,
            ptr_to: None,
        }
    }

    pub fn new_ptr(to: &Type) -> Self {
        Type {
            kind: TypeKind::Ptr,
            ptr_to: Some(Box::new(to.clone())),
        }
    }
}
