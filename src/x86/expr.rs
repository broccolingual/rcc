use super::{ARG_REGS, Generator};
use crate::errors::CompileError;
use crate::node::{BinaryOp, Node, NodeKind, UnaryOp};
use crate::types::TypeRef;

impl Generator<'_> {
    // 式のコード生成
    pub(super) fn gen_expr(&mut self, node: &Node) -> Result<(), CompileError> {
        match &node.kind {
            NodeKind::Number { val } => {
                self.builder.add_row(&format!("push {}", val), true);
            }
            NodeKind::String { index, .. } => {
                self.builder.add_row(&format!("lea rax, .L.str.{}[rip]", index), true); // RIP相対アドレッシング
                self.builder.add_row("push rax", true); // 文字列リテラルのアドレスをスタックに積む
            }
            NodeKind::Var { .. } | NodeKind::Member { .. } => {
                self.gen_addr(node)?;
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
            NodeKind::LogicalAnd { lhs, rhs, label } => {
                self.gen_expr(lhs)?;
                self.test_zero();
                self.builder.add_row(&format!("je .L.false.{}", label), true);
                self.gen_expr(rhs)?;
                self.test_zero();
                self.builder.add_row(&format!("je .L.false.{}", label), true);
                self.builder.add_row("push 1", true); // true
                self.builder.add_row(&format!("jmp .L.end.{}", label), true);
                self.builder.add_row(&format!(".L.false.{}:", label), false);
                self.builder.add_row("push 0", true); // false
                self.builder.add_row(&format!(".L.end.{}:", label), false);
            }
            NodeKind::LogicalOr { lhs, rhs, label } => {
                self.gen_expr(lhs)?;
                self.test_zero();
                self.builder.add_row(&format!("jne .L.true.{}", label), true);
                self.gen_expr(rhs)?;
                self.test_zero();
                self.builder.add_row(&format!("jne .L.true.{}", label), true);
                self.builder.add_row("push 0", true); // false
                self.builder.add_row(&format!("jmp .L.end.{}", label), true);
                self.builder.add_row(&format!(".L.true.{}:", label), false);
                self.builder.add_row("push 1", true); // true
                self.builder.add_row(&format!(".L.end.{}:", label), false);
            }
            NodeKind::Comma { lhs, rhs } => {
                // カンマ演算子: 左辺を評価して結果を捨て、右辺を評価して返す
                self.gen_expr(lhs)?;
                self.builder.add_row("pop rax", true); // 左辺の結果を捨てる
                self.gen_expr(rhs)?;
            }
            NodeKind::Ternary { cond, then, els, label } => {
                self.gen_expr(cond)?;
                self.test_zero();
                self.builder.add_row(&format!("je .L.else.{}", label), true);
                self.gen_expr(then)?;
                self.builder.add_row(&format!("jmp .L.end.{}", label), true);
                self.builder.add_row(&format!(".L.else.{}:", label), false);
                self.gen_expr(els)?;
                self.builder.add_row(&format!(".L.end.{}:", label), false);
            }
            NodeKind::Cast { expr } => {
                self.gen_expr(expr)?;
                self.gen_cast(expr.ty, node.ty)?;
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

    fn gen_unary(&mut self, op: &UnaryOp, expr: &Node, ty: &TypeRef) -> Result<(), CompileError> {
        match op {
            UnaryOp::Plus => {
                // unary plus: 式を評価してそのまま返す
                self.gen_expr(expr)?;
            }
            UnaryOp::Minus => {
                // unary minus: 式を評価して符号を反転
                self.gen_expr(expr)?;
                self.builder.add_row("pop rax", true);
                self.builder.add_row("neg rax", true);
                self.builder.add_row("push rax", true);
            }
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
            BinaryOp::Gt => self.gen_compare("setg"),
            BinaryOp::Ge => self.gen_compare("setge"),
            BinaryOp::Assign => unreachable!(),
        }
        self.builder.add_row("push rax", true); // 演算結果をスタックに積む
        Ok(())
    }

    fn gen_cast(&mut self, from: TypeRef, to: TypeRef) -> Result<(), CompileError> {
        if to.is_void() {
            // to が void 型の場合、スタックトップを破棄
            self.builder.add_row("pop rax", true);
            return Ok(());
        }

        let from_size = from.size_of();
        let to_size = to.size_of();

        if to_size >= from_size {
            return Ok(());
        }

        self.builder.add_row("pop rax", true);
        match to_size {
            1 => self.builder.add_row("movsx rax, al", true),
            2 => self.builder.add_row("movsx rax, ax", true),
            4 => self.builder.add_row("movsxd rax, eax", true),
            _ => {}
        }
        self.builder.add_row("push rax", true);
        Ok(())
    }
}
