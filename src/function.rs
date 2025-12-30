use core::fmt;

use crate::errors::CompileError;
use crate::node::Node;
use crate::symbol::{FlatTable, ScopedTable, Symbol, Variable};
use crate::types::{AlignUp, Decl, Type, TypeKind};

struct StackFrame {
    next_offset: usize,
}

impl Default for StackFrame {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for StackFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (16 byte aligned)", self.next_offset.align_up(16))
    }
}

impl StackFrame {
    fn new() -> Self {
        Self { next_offset: 0 }
    }

    // 型に合わせてスタック上に領域を確保し、そのオフセットを返す
    fn alloc(&mut self, ty: &Type) -> usize {
        self.next_offset = self.next_offset.align_up(ty.align_of());
        self.next_offset += ty.size_of();
        self.next_offset
    }
}

#[derive(Debug)]
pub(crate) struct Func {
    pub(crate) name: String,
    pub(crate) body: Vec<Node>,
    pub(crate) locals: Vec<Variable>,
    local_symbol_table: ScopedTable<Symbol>,
    param_symbol_table: FlatTable<Symbol>,
    local_tag_table: ScopedTable<Type>,
    stack_frame: StackFrame,
    pub(crate) return_ty: Type,
}

impl Func {
    pub(crate) fn new(name: &str) -> Self {
        Func {
            name: name.to_string(),
            body: Vec::new(),
            locals: Vec::new(),
            local_symbol_table: ScopedTable::new(),
            param_symbol_table: FlatTable::new(),
            local_tag_table: ScopedTable::new(),
            stack_frame: StackFrame::new(),
            return_ty: Type::from(TypeKind::Void, false),
        }
    }
}

impl Func {
    // スタックフレームのサイズを取得
    pub(crate) fn get_stack_size(&self) -> usize {
        self.stack_frame.next_offset.align_up(16) // 16バイトアラインメント
    }

    pub(crate) fn enter_scope(&mut self) {
        self.local_symbol_table.enter_scope();
        self.local_tag_table.enter_scope();
    }

    pub(crate) fn leave_scope(&mut self) {
        self.local_symbol_table.leave_scope();
        self.local_tag_table.leave_scope();
    }

    pub(crate) fn register_param(&mut self, decl: Decl) -> Result<(), CompileError> {
        if self.param_symbol_table.find(&decl.name).is_some() {
            return Err(CompileError::Redecl {
                name: decl.name,
                span: decl.span,
            });
        }
        let offset = self.stack_frame.alloc(&decl.ty);
        let symbol = Symbol::new(&decl.name, decl.ty, offset);
        self.param_symbol_table.insert(decl.name, symbol);
        Ok(())
    }

    pub(crate) fn find_param(&self, name: &str) -> Option<&Symbol> {
        self.param_symbol_table.find(name)
    }

    pub(crate) fn get_params_iter(&self) -> impl Iterator<Item = &Symbol> {
        self.param_symbol_table.iter()
    }

    pub(crate) fn register_local_var(&mut self, decl: Decl) -> Result<(), CompileError> {
        if self
            .local_symbol_table
            .find_in_current_scope(&decl.name)
            .is_some()
        {
            return Err(CompileError::Redecl {
                name: decl.name,
                span: decl.span,
            });
        }
        let offset = self.stack_frame.alloc(&decl.ty);
        let symbol = Symbol::new(&decl.name, decl.ty, offset);
        let symbol_id = self.local_symbol_table.insert(decl.name, symbol);
        self.locals.push(Variable {
            symbol_id,
            init: decl.init,
        });
        Ok(())
    }

    pub(crate) fn find_local_var(&self, name: &str) -> Option<&Symbol> {
        self.local_symbol_table.find(name)
    }

    pub(crate) fn get_local_symbol_by_id(&self, symbol_id: usize) -> Option<&Symbol> {
        self.local_symbol_table.items.get(symbol_id)
    }

    pub(crate) fn register_struct_tag(
        &mut self,
        name: String,
        ty: Type,
        span: (usize, usize),
    ) -> Result<(), CompileError> {
        if self.local_tag_table.find_in_current_scope(&name).is_some() {
            return Err(CompileError::Redecl { name, span });
        }
        self.local_tag_table.insert(name, ty);
        Ok(())
    }

    pub(crate) fn find_struct_tag(&self, name: &str) -> Option<&Type> {
        self.local_tag_table.find(name)
    }
}
