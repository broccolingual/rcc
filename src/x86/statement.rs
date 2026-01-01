use crate::errors::CompileError;
use crate::node::{Node, NodeKind};
use crate::x86::Generator;

impl Generator<'_> {
    // 文のコード生成
    pub(super) fn gen_stmt(&mut self, node: &Node) -> Result<(), CompileError> {
        match &node.kind {
            NodeKind::If {
                cond,
                then,
                els,
                label,
            } => {
                self.gen_expr(cond)?;
                self.test_zero();
                if let Some(els_node) = els {
                    // else節あり
                    self.builder.add_row(&format!("je .L.else.{}", label), true);
                    self.gen_stmt(then)?;
                    self.builder.add_row(&format!("jmp .L.end.{}", label), true);
                    self.builder.add_row(&format!(".L.else.{}:", label), false);
                    self.gen_stmt(els_node)?;
                    self.builder.add_row(&format!(".L.end.{}:", label), false);
                } else {
                    // else節なし
                    self.builder.add_row(&format!("je .L.end.{}", label), true);
                    self.gen_stmt(then)?;
                    self.builder.add_row(&format!(".L.end.{}:", label), false);
                }
            }
            NodeKind::While { cond, then, label } => {
                self.builder
                    .add_row(&format!(".L.continue.{}:", label), false);
                self.gen_expr(cond)?;
                self.test_zero();
                self.builder
                    .add_row(&format!("je .L.break.{}", label), true);
                self.gen_stmt(then)?;
                self.builder
                    .add_row(&format!("jmp .L.continue.{}", label), true);
                self.builder.add_row(&format!(".L.break.{}:", label), false);
            }
            NodeKind::For {
                init,
                cond,
                inc,
                then,
                label,
            } => {
                if let Some(init) = init.as_ref() {
                    if init.is_expr() {
                        self.gen_expr(init)?;
                        self.builder.add_row("pop rax", true); // 初期化式の結果を捨てる
                    } else {
                        self.gen_stmt(init)?;
                    }
                }
                self.builder.add_row(&format!(".L.begin.{}:", label), false);
                if let Some(cond) = cond.as_ref() {
                    self.gen_expr(cond)?;
                    self.test_zero();
                    self.builder
                        .add_row(&format!("je .L.break.{}", label), true);
                }
                self.gen_stmt(then)?;
                self.builder
                    .add_row(&format!(".L.continue.{}:", label), false);
                if let Some(inc) = inc.as_ref() {
                    if inc.is_expr() {
                        self.gen_expr(inc)?;
                        self.builder.add_row("pop rax", true); // 増分式の結果を捨てる
                    } else {
                        self.gen_stmt(inc)?;
                    }
                }
                self.builder
                    .add_row(&format!("jmp .L.begin.{}", label), true);
                self.builder.add_row(&format!(".L.break.{}:", label), false);
            }
            NodeKind::Do { cond, then, label } => {
                self.builder.add_row(&format!(".L.begin.{}:", label), false);
                self.gen_stmt(then)?;
                self.builder
                    .add_row(&format!(".L.continue.{}:", label), false);
                self.gen_expr(cond)?;
                self.test_zero();
                self.builder
                    .add_row(&format!("jne .L.begin.{}", label), true);
                self.builder.add_row(&format!(".L.break.{}:", label), false);
            }
            NodeKind::Block { body } => {
                for node in body.iter() {
                    if node.is_expr() {
                        self.gen_expr(node)?;
                        self.builder.add_row("pop rax", true); // ブロック内の各文の結果を捨てる
                    } else {
                        self.gen_stmt(node)?;
                    }
                }
            }
            NodeKind::Break { label } => {
                self.builder
                    .add_row(&format!("jmp .L.break.{}", label), true);
            }
            NodeKind::Continue { label } => {
                self.builder
                    .add_row(&format!("jmp .L.continue.{}", label), true);
            }
            NodeKind::Goto { name } => {
                self.builder.add_row(
                    &format!("jmp .L.label.{}.{}", self.current_func_name, name),
                    true,
                );
            }
            NodeKind::Label { name, expr } => {
                self.builder.add_row(
                    &format!(".L.label.{}.{}:", self.current_func_name, name),
                    false,
                );
                if expr.is_expr() {
                    self.gen_expr(expr)?;
                    self.builder.add_row("pop rax", true); // ラベル付き文の結果を捨てる
                } else {
                    self.gen_stmt(expr)?;
                }
            }
            NodeKind::Return { expr } => {
                if let Some(expr) = expr {
                    self.gen_expr(expr)?;
                    self.builder.add_row("pop rax", true);
                }
                self.builder
                    .add_row(&format!("jmp .L.return.{}", self.current_func_name), true);
            }
            NodeKind::Nop => {}
            _ => {
                self.gen_expr(node)?;
                self.builder.add_row("pop rax", true); // 式の結果を捨てる
            }
        }
        Ok(())
    }
}
