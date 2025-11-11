use core::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeKind {
    Int,
    Ptr,
}

#[derive(Clone)]
pub struct Type {
    pub kind: TypeKind,
    pub ptr_to: Option<Box<Type>>,
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            TypeKind::Ptr => {
                let mut cur = self;
                let mut depth = 0;
                while let Some(ref to) = cur.ptr_to {
                    depth += 1;
                    cur = to;
                }
                write!(f, "{}{:?}", "*".repeat(depth), cur.kind)
            }
            _ => write!(f, "{:?}", self.kind),
        }
    }
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

    pub fn is_ptr(&self) -> bool {
        matches!(self.kind, TypeKind::Ptr)
    }

    pub fn size_of(&self) -> i64 {
        match self.kind {
            TypeKind::Int => 4,
            TypeKind::Ptr => 8,
        }
    }
}
