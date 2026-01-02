use crate::node::Node;
use crate::symbol::Symbol;
use crate::types::Type;
use core::fmt;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(PartialEq, Eq)]
pub(crate) struct LocalVar {
    pub(crate) symbol: Rc<RefCell<Symbol>>, // symbolのインデックス
    pub(crate) offset: usize,               // スタック上のオフセット
}

impl fmt::Debug for LocalVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} (@{})", self.symbol.borrow().ty, self.offset)
    }
}

impl LocalVar {
    pub(crate) fn new(symbol: Rc<RefCell<Symbol>>) -> Self {
        Self { symbol, offset: 0 }
    }
}

#[derive(PartialEq, Eq)]
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

    pub(crate) fn find_local_var(&self, symbol: Rc<RefCell<Symbol>>) -> Option<&LocalVar> {
        for param in &self.params {
            if Rc::ptr_eq(&param.symbol, &symbol) {
                return Some(param);
            }
        }
        self.locals
            .iter()
            .find(|&local| Rc::ptr_eq(&local.symbol, &symbol))
            .map(|v| v as _)
    }
}
