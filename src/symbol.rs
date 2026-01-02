mod table;
mod variable;

use crate::types::Type;
use core::fmt;
pub(crate) use table::*;
pub(crate) use variable::*;

pub(crate) struct Symbol {
    pub(crate) name: String,
    pub(crate) ty: Type,
    pub(crate) offset: usize,
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {:?} @{}", self.name, self.ty, self.offset)
    }
}

impl Symbol {
    pub(crate) fn new(name: &str, ty: Type, offset: usize) -> Self {
        Self {
            name: name.to_string(),
            ty,
            offset,
        }
    }
}
