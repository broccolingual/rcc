use crate::errors::CompileError;
use crate::node::{Node, NodeKind};
use crate::x86::Generator;

impl Generator<'_> {
    // 文のコード生成
    pub(super) fn gen_stmt(&mut self, node: &Node) -> Result<(), CompileError> {
        match &node.kind {
            NodeKind::If { cond, then, els } => {
                let seq = self.label_seq;
                self.label_seq += 1;

                self.gen_expr(cond)?;
                self.test_zero();
                if let Some(els_node) = els {
                    // else節あり
                    self.builder.add_row(&format!("je .L.else.{}", seq), true);
                    self.gen_stmt(then)?;
                    self.builder.add_row(&format!("jmp .L.end.{}", seq), true);
                    self.builder.add_row(&format!(".L.else.{}:", seq), false);
                    self.gen_stmt(els_node)?;
                    self.builder.add_row(&format!(".L.end.{}:", seq), false);
                } else {
                    // else節なし
                    self.builder.add_row(&format!("je .L.end.{}", seq), true);
                    self.gen_stmt(then)?;
                    self.builder.add_row(&format!(".L.end.{}:", seq), false);
                }
            }
            NodeKind::While { cond, then } => {
                let seq = self.label_seq;
                self.label_seq += 1;
                let current_break_seq = self.break_seq;
                let current_continue_seq = self.continue_seq;
                self.break_seq = seq;
                self.continue_seq = seq;

                self.builder
                    .add_row(&format!(".L.continue.{}:", seq), false);
                self.gen_expr(cond)?;
                self.test_zero();
                self.builder.add_row(&format!("je .L.break.{}", seq), true);
                self.gen_stmt(then)?;
                self.builder
                    .add_row(&format!("jmp .L.continue.{}", seq), true);
                self.builder.add_row(&format!(".L.break.{}:", seq), false);

                self.break_seq = current_break_seq;
                self.continue_seq = current_continue_seq;
            }
            NodeKind::For {
                init,
                cond,
                inc,
                then,
            } => {
                let seq = self.label_seq;
                self.label_seq += 1;
                let current_break_seq = self.break_seq;
                let current_continue_seq = self.continue_seq;
                self.break_seq = seq;
                self.continue_seq = seq;
                if let Some(init) = init.as_ref() {
                    if init.is_expr() {
                        self.gen_expr(init)?;
                        self.builder.add_row("pop rax", true); // 初期化式の結果を捨てる
                    } else {
                        self.gen_stmt(init)?;
                    }
                }
                self.builder.add_row(&format!(".L.begin.{}:", seq), false);
                if let Some(cond) = cond.as_ref() {
                    self.gen_expr(cond)?;
                    self.test_zero();
                    self.builder.add_row(&format!("je .L.break.{}", seq), true);
                }
                self.gen_stmt(then)?;
                self.builder
                    .add_row(&format!(".L.continue.{}:", seq), false);
                if let Some(inc) = inc.as_ref() {
                    if inc.is_expr() {
                        self.gen_expr(inc)?;
                        self.builder.add_row("pop rax", true); // 増分式の結果を捨てる
                    } else {
                        self.gen_stmt(inc)?;
                    }
                }
                self.builder.add_row(&format!("jmp .L.begin.{}", seq), true);
                self.builder.add_row(&format!(".L.break.{}:", seq), false);

                self.break_seq = current_break_seq;
                self.continue_seq = current_continue_seq;
            }
            NodeKind::Do { cond, then } => {
                let seq = self.label_seq;
                self.label_seq += 1;
                let current_break_seq = self.break_seq;
                let current_continue_seq = self.continue_seq;
                self.break_seq = seq;
                self.continue_seq = seq;

                self.builder.add_row(&format!(".L.begin.{}:", seq), false);
                self.gen_stmt(then)?;
                self.builder
                    .add_row(&format!(".L.continue.{}:", seq), false);
                self.gen_expr(cond)?;
                self.test_zero();
                self.builder.add_row(&format!("jne .L.begin.{}", seq), true);
                self.builder.add_row(&format!(".L.break.{}:", seq), false);

                self.break_seq = current_break_seq;
                self.continue_seq = current_continue_seq;
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
            NodeKind::Break => {
                self.builder
                    .add_row(&format!("jmp .L.break.{}", self.break_seq), true);
            }
            NodeKind::Continue => {
                self.builder
                    .add_row(&format!("jmp .L.continue.{}", self.continue_seq), true);
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
