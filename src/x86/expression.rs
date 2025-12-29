use crate::errors::CompileError;
use crate::node::{BinaryOp, Node, NodeKind, UnaryOp};
use crate::types::Type;
use crate::x86::Generator;
use crate::x86::register::ARG_REGS;

impl Generator {
    // 式のコード生成
    pub(super) fn gen_expr(&mut self, node: &Node) -> Result<(), CompileError> {
        match &node.kind {
            NodeKind::Number { val } => {
                self.builder.add_row(&format!("push {}", val), true);
            }
            NodeKind::String { index, .. } => {
                self.builder
                    .add_row(&format!("lea rax, .L.str.{}[rip]", index), true); // RIP相対アドレッシング
                self.builder.add_row("push rax", true); // 文字列リテラルのアドレスをスタックに積む
            }
            NodeKind::Var { .. } => {
                self.gen_addr(node)?;
                if !node.ty.is_array() {
                    self.load(&node.ty)?;
                }
            }
            NodeKind::Member { obj, offset, .. } => {
                self.gen_addr(obj)?;
                self.builder.add_row("pop rax", true); // オブジェクトのアドレス
                self.builder.add_row(&format!("add rax, {}", offset), true);
                self.builder.add_row("push rax", true); // メンバのアドレスをスタックに積む
                if !node.ty.is_array() {
                    self.load(&node.ty)?;
                }
            }
            NodeKind::BinaryOp { op, lhs, rhs } => {
                // 二項演算子
                self.gen_expr(lhs)?;
                self.gen_expr(rhs)?;
                self.gen_binary(op)?;
            }
            NodeKind::UnaryOp { op, expr } => {
                // 単項演算子
                self.gen_unary(op, expr, &node.ty)?;
            }
            NodeKind::Assign { op, lhs, rhs } => {
                self.gen_assign(op, lhs, rhs)?;
            }
            NodeKind::LogicalAnd { lhs, rhs } => {
                let seq = self.label_seq;
                self.label_seq += 1;
                self.gen_expr(lhs)?;
                self.test_zero();
                self.builder.add_row(&format!("je .L.false.{}", seq), true);
                self.gen_expr(rhs)?;
                self.test_zero();
                self.builder.add_row(&format!("je .L.false.{}", seq), true);
                self.builder.add_row("push 1", true); // true
                self.builder.add_row(&format!("jmp .L.end.{}", seq), true);
                self.builder.add_row(&format!(".L.false.{}:", seq), false);
                self.builder.add_row("push 0", true); // false
                self.builder.add_row(&format!(".L.end.{}:", seq), false);
            }
            NodeKind::LogicalOr { lhs, rhs } => {
                let seq = self.label_seq;
                self.label_seq += 1;
                self.gen_expr(lhs)?;
                self.test_zero();
                self.builder.add_row(&format!("jne .L.true.{}", seq), true);
                self.gen_expr(rhs)?;
                self.test_zero();
                self.builder.add_row(&format!("jne .L.true.{}", seq), true);
                self.builder.add_row("push 0", true); // false
                self.builder.add_row(&format!("jmp .L.end.{}", seq), true);
                self.builder.add_row(&format!(".L.true.{}:", seq), false);
                self.builder.add_row("push 1", true); // true
                self.builder.add_row(&format!(".L.end.{}:", seq), false);
            }
            NodeKind::Ternary { cond, then, els } => {
                let seq = self.label_seq;
                self.label_seq += 1;
                self.gen_expr(cond)?;
                self.test_zero();
                self.builder.add_row(&format!("je .L.else.{}", seq), true);
                self.gen_expr(then)?;
                self.builder.add_row(&format!("jmp .L.end.{}", seq), true);
                self.builder.add_row(&format!(".L.else.{}:", seq), false);
                self.gen_expr(els)?;
                self.builder.add_row(&format!(".L.end.{}:", seq), false);
            }
            NodeKind::Call { name, args } => {
                let arg_count = args.len();

                if arg_count > 6 {
                    unimplemented!("6個を超える引数の関数呼び出しには対応していません");
                }

                // 引数をスタックに積む（逆順）
                for arg in args.iter().rev() {
                    self.gen_expr(arg)?;
                }

                // 引数をレジスタに移動
                for reg in ARG_REGS.iter().take(arg_count) {
                    self.builder.add_row(&format!("pop {}", reg.qword()), true);
                }

                // 関数呼び出し（アラインメントは揃っているはず）
                self.builder.add_row("xor al, al", true); // 浮動小数点は使わないので0に設定
                self.builder.add_row(&format!("call {}", name), true); // 関数呼び出し
                self.builder.add_row("push rax", true); // 戻り値をスタックに積む
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn gen_assign(&mut self, op: &BinaryOp, lhs: &Node, rhs: &Node) -> Result<(), CompileError> {
        match op {
            BinaryOp::Assign => {
                self.gen_addr(lhs)?;
                self.gen_expr(rhs)?;
                self.store(&lhs.ty)?;
            }
            _ => {
                self.gen_addr(lhs)?;
                self.builder.add_row("push [rsp]", true);
                self.load(&lhs.ty)?;
                self.gen_expr(rhs)?;
                self.gen_binary(op)?;
                self.store(&lhs.ty)?;
            }
        }
        Ok(())
    }

    fn gen_unary(&mut self, op: &UnaryOp, expr: &Node, ty: &Type) -> Result<(), CompileError> {
        match op {
            UnaryOp::BitNot => {
                self.gen_expr(expr)?;
                self.builder.add_row("not QWORD PTR [rsp]", true);
            }
            UnaryOp::LogicalNot => {
                self.gen_expr(expr)?;
                self.test_zero();
                self.builder.add_row("sete al", true);
                self.builder.add_row("movzx rax, al", true);
                self.builder.add_row("push rax", true);
            }
            UnaryOp::Addr => {
                self.gen_addr(expr)?;
            }
            UnaryOp::Deref => {
                self.gen_expr(expr)?;
                // 型が配列でない場合にロード
                // 配列型の場合、アドレスをスタックに積むだけ
                if !ty.is_array() {
                    self.load(ty)?;
                }
            }
            UnaryOp::PreInc => {
                self.gen_addr(expr)?;
                self.builder.add_row("push [rsp]", true);
                self.load(&expr.ty)?;
                self.inc();
                self.store(&expr.ty)?;
            }
            UnaryOp::PreDec => {
                self.gen_addr(expr)?;
                self.builder.add_row("push [rsp]", true);
                self.load(&expr.ty)?;
                self.dec();
                self.store(&expr.ty)?;
            }
            UnaryOp::PostInc => {
                self.gen_addr(expr)?;
                self.builder.add_row("push [rsp]", true);
                self.load(&expr.ty)?;
                self.inc();
                self.store(&expr.ty)?;
                self.dec();
            }
            UnaryOp::PostDec => {
                self.gen_addr(expr)?;
                self.builder.add_row("push [rsp]", true);
                self.load(&expr.ty)?;
                self.dec();
                self.store(&expr.ty)?;
                self.inc();
            }
        }
        Ok(())
    }

    fn gen_binary(&mut self, op: &BinaryOp) -> Result<(), CompileError> {
        self.builder.add_row("pop rdi", true); // 右オペランド
        self.builder.add_row("pop rax", true); // 左オペランド

        match op {
            BinaryOp::Add => self.builder.add_row("add rax, rdi", true),
            BinaryOp::Sub => self.builder.add_row("sub rax, rdi", true),
            BinaryOp::Mul => self.builder.add_row("imul rax, rdi", true),
            BinaryOp::Div => {
                self.builder.add_row("cqo", true);
                self.builder.add_row("idiv rdi", true);
            }
            BinaryOp::Rem => {
                self.builder.add_row("cqo", true);
                self.builder.add_row("idiv rdi", true);
                self.builder.add_row("mov rax, rdx", true);
            }
            BinaryOp::BitAnd => {
                self.builder.add_row("and rax, rdi", true);
            }
            BinaryOp::BitOr => {
                self.builder.add_row("or rax, rdi", true);
            }
            BinaryOp::BitXor => {
                self.builder.add_row("xor rax, rdi", true);
            }
            BinaryOp::Shl => {
                self.builder.add_row("mov cl, dil", true);
                self.builder.add_row("shl rax, cl", true);
            }
            BinaryOp::Shr => {
                self.builder.add_row("mov cl, dil", true);
                self.builder.add_row("shr rax, cl", true);
            }
            BinaryOp::Eq => self.gen_compare("sete"),
            BinaryOp::Ne => self.gen_compare("setne"),
            BinaryOp::Lt => self.gen_compare("setl"),
            BinaryOp::Le => self.gen_compare("setle"),
            BinaryOp::Assign => unreachable!(),
        }
        self.builder.add_row("push rax", true); // 演算結果をスタックに積む
        Ok(())
    }
}
