mod table;
mod variable;

use crate::types::Type;
pub(crate) use table::*;
pub(crate) use variable::*;

#[derive(Debug)]
pub(crate) struct Symbol {
    pub(crate) name: String,
    pub(crate) ty: Type,
    pub(crate) offset: usize,
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
