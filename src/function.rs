use crate::node::Node;
use crate::types::Type;
use core::fmt;

pub(crate) struct LocalVar {
    pub(crate) symbol_idx: usize, // symbolのインデックス
    pub(crate) offset: usize,     // スタック上のオフセット
}

impl fmt::Debug for LocalVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "id: {} (@{})", self.symbol_idx, self.offset)
    }
}

impl LocalVar {
    pub(crate) fn new(symbol_idx: usize) -> Self {
        Self {
            symbol_idx,
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
    pub(crate) return_ty: Type,
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
            return_ty: Type::default(),
        }
    }

    pub(crate) fn find_local_var(&self, symbol_idx: usize) -> Option<&LocalVar> {
        for param in &self.params {
            if param.symbol_idx == symbol_idx {
                return Some(param);
            }
        }
        self.locals
            .iter()
            .find(|&local| local.symbol_idx == symbol_idx)
            .map(|v| v as _)
    }
}
