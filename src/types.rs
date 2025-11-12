use core::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeKind {
    Int,
    Ptr,
    Array,
}

#[derive(Clone)]
pub struct Type {
    pub kind: TypeKind,
    pub ptr_to: Option<Box<Type>>,
    pub array_size: usize, // 配列の要素数（配列型の場合）
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
            array_size: 0,
        }
    }

    pub fn new_ptr(to: &Type) -> Self {
        Type {
            kind: TypeKind::Ptr,
            ptr_to: Some(Box::new(to.clone())),
            array_size: 0,
        }
    }

    pub fn new_array(of: &Type, size: usize) -> Self {
        Type {
            kind: TypeKind::Array,
            ptr_to: Some(Box::new(of.clone())),
            array_size: size,
        }
    }

    pub fn is_ptr(&self) -> bool {
        matches!(self.kind, TypeKind::Ptr)
    }

    // TODO: 配列型のサイズ計算は仮実装
    pub fn size_of(&self) -> i64 {
        match self.kind {
            TypeKind::Int => 8, // サイズは8バイトで仮置き
            TypeKind::Ptr => 8,
            TypeKind::Array => {
                if let Some(ref to) = self.ptr_to {
                    to.size_of() * self.array_size as i64
                } else {
                    0
                }
            }
        }
    }
}
