use crate::node::Node;
use crate::symbol::SymbolId;
use crate::types::TypeRef;
use core::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct FuncId(pub usize);

#[derive(PartialEq, Eq)]
pub(crate) struct LocalVar {
    pub(crate) symbol_id: SymbolId, // symbolのインデックス
    pub(crate) offset: usize,       // スタック上のオフセット
}

impl fmt::Debug for LocalVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} (@{})", self.symbol_id, self.offset)
    }
}

impl LocalVar {
    pub(crate) fn new(symbol_id: SymbolId) -> Self {
        Self {
            symbol_id,
            offset: 0,
        }
    }
}

pub(crate) struct Func {
    pub(crate) name: String,
    pub(crate) body: Vec<Node>,
    pub(crate) params: Vec<LocalVar>, // symbolのインデックス
    pub(crate) locals: Vec<LocalVar>, // symbolのインデックス
    pub(crate) stack_size: usize,
    pub(crate) return_ty: TypeRef,
}

impl fmt::Debug for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:?} {} {{", self.return_ty, self.name)?;
        writeln!(f, "  stack size: {}", self.stack_size)?;
        write!(f, "  params: [")?;
        for param in &self.params {
            write!(f, "{:?}, ", param)?;
        }
        writeln!(f, "]")?;
        write!(f, "  locals: [")?;
        for local in &self.locals {
            write!(f, "{:?}, ", local)?;
        }
        writeln!(f, "]")?;
        writeln!(f, "}}")
    }
}

impl Func {
    pub(crate) fn new(name: &str) -> Self {
        Func {
            name: name.to_string(),
            body: Vec::new(),
            params: Vec::new(),
            locals: Vec::new(),
            stack_size: 0,
            return_ty: TypeRef::default(),
        }
    }

    pub(crate) fn find_local_var(&self, symbol_id: SymbolId) -> Option<&LocalVar> {
        self.params
            .iter()
            .chain(self.locals.iter())
            .find(|v| v.symbol_id == symbol_id)
    }
}
