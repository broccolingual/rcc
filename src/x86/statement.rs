use crate::errors::CompileError;
use crate::node::{Node, NodeKind};
use crate::x86::Generator;

impl Generator<'_> {
    // 文のコード生成
    pub(super) fn gen_stmt(&mut self, node: &Node) -> Result<(), CompileError> {
        match &node.kind {
            NodeKind::If { cond, then, els } => {
                let seq = self.next_label();

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
                let seq = self.next_label();
                self.push_loop(seq);

                self.builder
                    .add_row(&format!(".L.continue.{}:", seq), false);
                self.gen_expr(cond)?;
                self.test_zero();
                self.builder.add_row(&format!("je .L.break.{}", seq), true);
                self.gen_stmt(then)?;
                self.builder
                    .add_row(&format!("jmp .L.continue.{}", seq), true);
                self.builder.add_row(&format!(".L.break.{}:", seq), false);

                self.pop_loop();
            }
            NodeKind::For {
                init,
                cond,
                inc,
                then,
            } => {
                let seq = self.next_label();
                self.push_loop(seq);

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

                self.pop_loop();
            }
            NodeKind::Do { cond, then } => {
                let seq = self.next_label();
                self.push_loop(seq);

                self.builder.add_row(&format!(".L.begin.{}:", seq), false);
                self.gen_stmt(then)?;
                self.builder
                    .add_row(&format!(".L.continue.{}:", seq), false);
                self.gen_expr(cond)?;
                self.test_zero();
                self.builder.add_row(&format!("jne .L.begin.{}", seq), true);
                self.builder.add_row(&format!(".L.break.{}:", seq), false);

                self.pop_loop();
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
                let seq = self
                    .current_loop_label()
                    .ok_or_else(|| CompileError::InvalidStmt {
                        msg: "break文がループの外で使われています".to_string(),
                        span: node.span,
                    })?;
                self.builder.add_row(&format!("jmp .L.break.{}", seq), true);
            }
            NodeKind::Continue => {
                let seq = self
                    .current_loop_label()
                    .ok_or_else(|| CompileError::InvalidStmt {
                        msg: "continue文がループの外で使われています".to_string(),
                        span: node.span,
                    })?;
                self.builder
                    .add_row(&format!("jmp .L.continue.{}", seq), true);
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
