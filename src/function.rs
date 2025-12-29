use crate::errors::CompileError;
use crate::node::Node;
use crate::symbol::{FlatTable, ScopedTable, Symbol, Variable};
use crate::types::{AlignUp, Type, TypeKind};

#[derive(Debug)]
struct StackFrame {
    next_offset: usize,
}

impl Default for StackFrame {
    fn default() -> Self {
        Self::new()
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
pub(crate) struct Function {
    pub(crate) name: String,
    pub(crate) body: Vec<Node>,
    pub(crate) locals: Vec<Variable>,
    local_symbol_table: ScopedTable<Symbol>,
    param_symbol_table: FlatTable<Symbol>,
    local_tag_table: ScopedTable<Type>,
    stack_frame: StackFrame,
    pub(crate) return_ty: Type,
}

impl Function {
    pub(crate) fn new(name: &str) -> Self {
        Function {
            name: name.to_string(),
            body: Vec::new(),
            locals: Vec::new(),
            local_symbol_table: ScopedTable::<Symbol>::new(),
            param_symbol_table: FlatTable::<Symbol>::new(),
            local_tag_table: ScopedTable::<Type>::new(),
            stack_frame: StackFrame::new(),
            return_ty: Type::from(TypeKind::Void, false),
        }
    }
}

impl Function {
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

    pub(crate) fn register_param(&mut self, name: String, ty: Type) -> Result<(), CompileError> {
        if self.param_symbol_table.find(&name).is_some() {
            return Err(CompileError::Redeclaration { name });
        }
        let offset = self.stack_frame.alloc(&ty);
        let symbol = Symbol::new(&name, ty, offset);
        self.param_symbol_table.insert(name, symbol);
        Ok(())
    }

    pub(crate) fn find_param(&self, name: &str) -> Option<&Symbol> {
        self.param_symbol_table.find(name)
    }

    pub(crate) fn get_params_iter(&self) -> impl Iterator<Item = &Symbol> {
        self.param_symbol_table.iter()
    }

    pub(crate) fn register_local_var(
        &mut self,
        name: String,
        ty: Type,
        init: Vec<Node>,
    ) -> Result<(), CompileError> {
        if self
            .local_symbol_table
            .find_in_current_scope(&name)
            .is_some()
        {
            return Err(CompileError::Redeclaration { name });
        }
        let offset = self.stack_frame.alloc(&ty);
        let symbol = Symbol::new(&name, ty, offset);
        let symbol_id = self.local_symbol_table.insert(name, symbol);
        self.locals.push(Variable { symbol_id, init });
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
    ) -> Result<(), CompileError> {
        if self.local_tag_table.find_in_current_scope(&name).is_some() {
            return Err(CompileError::Redeclaration { name });
        }
        self.local_tag_table.insert(name, ty);
        Ok(())
    }

    pub(crate) fn find_struct_tag(&self, name: &str) -> Option<&Type> {
        self.local_tag_table.find(name)
    }
}
