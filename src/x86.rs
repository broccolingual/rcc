use crate::ast::{Node, NodeKind};

pub struct Generator {
    label_seq: usize,
    break_seq: usize,
    continue_seq: usize,
}

impl Generator {
    pub fn new() -> Self {
        Generator {
            label_seq: 1,
            break_seq: 0,
            continue_seq: 0,
        }
    }

    pub fn gen_asm_from_lval(&self, node: &Node) {
        if node.kind != NodeKind::LVar {
            panic!("代入の左辺値が変数ではありません");
        }
        println!("  mov rax, rbp");
        println!("  sub rax, {}", node.offset);
        println!("  push rax");
    }

    pub fn gen_asm_from_expr(&mut self, node: &Node) {
        match node.kind {
            NodeKind::Return => {
                self.gen_asm_from_expr(node.lhs.as_ref().unwrap());
                println!("  pop rax");
                println!("  mov rsp, rbp");
                println!("  pop rbp");
                println!("  ret");
                return;
            }
            NodeKind::Num => {
                println!("  push {}", node.val);
                return;
            }
            NodeKind::LVar => {
                self.gen_asm_from_lval(node);
                println!("  pop rax");
                println!("  mov rax, [rax]");
                println!("  push rax");
                return;
            }
            NodeKind::Assign => {
                self.gen_asm_from_lval(node.lhs.as_ref().unwrap());
                self.gen_asm_from_expr(node.rhs.as_ref().unwrap());
                println!("  pop rdi");
                println!("  pop rax");
                println!("  mov [rax], rdi");
                println!("  push rdi");
                return;
            }
            NodeKind::Ternary => {
                let seq = self.label_seq;
                self.label_seq += 1;
                self.gen_asm_from_expr(node.cond.as_ref().unwrap());
                println!("  pop rax");
                println!("  cmp rax, 0");
                println!("  je .Lelse{}", seq);
                self.gen_asm_from_expr(node.then.as_ref().unwrap());
                println!("  jmp .Lend{}", seq);
                println!(".Lelse{}:", seq);
                self.gen_asm_from_expr(node.els.as_ref().unwrap());
                println!(".Lend{}:", seq);
                return;
            }
            NodeKind::AddAssign
            | NodeKind::SubAssign
            | NodeKind::MulAssign
            | NodeKind::DivAssign
            | NodeKind::BitAndAssign
            | NodeKind::BitOrAssign
            | NodeKind::BitXorAssign
            | NodeKind::ShlAssign
            | NodeKind::ShrAssign => {
                unimplemented!("複合代入演算子は未実装です");
            }
            NodeKind::LogicalNot => {
                self.gen_asm_from_expr(node.lhs.as_ref().unwrap());
                println!("  pop rax");
                println!("  cmp rax, 0");
                println!("  sete al");
                println!("  movzb rax, al");
                println!("  push rax");
                return;
            }
            NodeKind::BitNot => {
                self.gen_asm_from_expr(node.lhs.as_ref().unwrap());
                println!("  pop rax");
                println!("  not rax");
                println!("  push rax");
                return;
            }
            NodeKind::LogicalAnd => {
                let seq = self.label_seq;
                self.label_seq += 1;
                self.gen_asm_from_expr(node.lhs.as_ref().unwrap());
                println!("  pop rax");
                println!("  cmp rax, 0");
                println!("  je .Lfalse{}", seq);
                self.gen_asm_from_expr(node.rhs.as_ref().unwrap());
                println!("  pop rax");
                println!("  cmp rax, 0");
                println!("  je .Lfalse{}", seq);
                println!("  push 1");
                println!("  jmp .Lend{}", seq);
                println!(".Lfalse{}:", seq);
                println!("  push 0");
                println!(".Lend{}:", seq);
                return;
            }
            NodeKind::LogicalOr => {
                let seq = self.label_seq;
                self.label_seq += 1;
                self.gen_asm_from_expr(node.lhs.as_ref().unwrap());
                println!("  pop rax");
                println!("  cmp rax, 0");
                println!("  jne .Ltrue{}", seq);
                self.gen_asm_from_expr(node.rhs.as_ref().unwrap());
                println!("  pop rax");
                println!("  cmp rax, 0");
                println!("  jne .Ltrue{}", seq);
                println!("  push 0");
                println!("  jmp .Lend{}", seq);
                println!(".Ltrue{}:", seq);
                println!("  push 1");
                println!(".Lend{}:", seq);
                return;
            }
            NodeKind::If => {
                let seq = self.label_seq;
                self.label_seq += 1;
                if node.els.is_some() {
                    // else節あり
                    self.gen_asm_from_expr(node.cond.as_ref().unwrap());
                    println!("  pop rax");
                    println!("  cmp rax, 0");
                    println!("  je .Lelse{}", seq);
                    self.gen_asm_from_expr(node.then.as_ref().unwrap());
                    println!("  jmp .Lend{}", seq);
                    println!(".Lelse{}:", seq);
                    self.gen_asm_from_expr(node.els.as_ref().unwrap());
                    println!(".Lend{}:", seq);
                } else {
                    // else節なし
                    self.gen_asm_from_expr(node.cond.as_ref().unwrap());
                    println!("  pop rax");
                    println!("  cmp rax, 0");
                    println!("  je .Lend{}", seq);
                    self.gen_asm_from_expr(node.then.as_ref().unwrap());
                    println!(".Lend{}:", seq);
                }
                return;
            }
            NodeKind::While => {
                let seq = self.label_seq;
                self.label_seq += 1;
                let current_break_seq = self.break_seq;
                let current_continue_seq = self.continue_seq;
                self.break_seq = seq;
                self.continue_seq = seq;

                println!(".Lcontinue{}:", seq);
                self.gen_asm_from_expr(node.cond.as_ref().unwrap());
                println!("  pop rax");
                println!("  cmp rax, 0");
                println!("  je .Lbreak{}", seq);
                self.gen_asm_from_expr(node.then.as_ref().unwrap());
                println!("  jmp .Lcontinue{}", seq);
                println!(".Lbreak{}:", seq);

                self.break_seq = current_break_seq;
                self.continue_seq = current_continue_seq;
                return;
            }
            NodeKind::For => {
                let seq = self.label_seq;
                self.label_seq += 1;
                let current_break_seq = self.break_seq;
                let current_continue_seq = self.continue_seq;
                self.break_seq = seq;
                self.continue_seq = seq;

                if let Some(init) = node.init.as_ref() {
                    self.gen_asm_from_expr(init);
                }
                println!(".Lbegin{}:", seq);
                if let Some(cond) = node.cond.as_ref() {
                    self.gen_asm_from_expr(cond);
                    println!("  pop rax");
                    println!("  cmp rax, 0");
                    println!("  je .Lbreak{}", seq);
                }
                self.gen_asm_from_expr(node.then.as_ref().unwrap());
                println!(".Lcontinue{}:", seq);
                if let Some(inc) = node.inc.as_ref() {
                    self.gen_asm_from_expr(inc);
                }
                println!("  jmp .Lbegin{}", seq);
                println!(".Lbreak{}:", seq);

                self.break_seq = current_break_seq;
                self.continue_seq = current_continue_seq;
                return;
            }
            NodeKind::Do => {
                let seq = self.label_seq;
                self.label_seq += 1;
                let current_break_seq = self.break_seq;
                let current_continue_seq = self.continue_seq;
                self.break_seq = seq;
                self.continue_seq = seq;

                println!(".Lbegin{}:", seq);
                self.gen_asm_from_expr(node.then.as_ref().unwrap());
                println!(".Lcontinue{}:", seq);
                self.gen_asm_from_expr(node.cond.as_ref().unwrap());
                println!("  pop rax");
                println!("  cmp rax, 0");
                println!("  jne .Lbegin{}", seq);
                println!(".Lbreak{}:", seq);

                self.break_seq = current_break_seq;
                self.continue_seq = current_continue_seq;
                return;
            }
            NodeKind::Block => {
                let mut cur = node.body.as_ref();
                while let Some(n) = cur {
                    self.gen_asm_from_expr(n);
                    println!("  pop rax"); // ブロック内の各文の結果を捨てる
                    cur = n.next.as_ref();
                }
                return;
            }
            NodeKind::Break => {
                println!("  jmp .Lbreak{}", self.break_seq);
                return;
            }
            NodeKind::Continue => {
                println!("  jmp .Lcontinue{}", self.continue_seq);
                return;
            }
            _ => {}
        }

        self.gen_asm_from_expr(node.lhs.as_ref().unwrap());
        self.gen_asm_from_expr(node.rhs.as_ref().unwrap());
        self.gen_asm_from_binary_op(node);
    }

    fn gen_asm_from_binary_op(&mut self, node: &Node) {
        println!("  pop rdi");
        println!("  pop rax");

        match node.kind {
            NodeKind::Add | NodeKind::AddAssign => println!("  add rax, rdi"),
            NodeKind::Sub | NodeKind::SubAssign => println!("  sub rax, rdi"),
            NodeKind::Mul | NodeKind::MulAssign => println!("  imul rax, rdi"),
            NodeKind::Div | NodeKind::DivAssign => {
                println!("  cqo");
                println!("  idiv rdi");
            }
            NodeKind::Rem => {
                println!("  cqo");
                println!("  idiv rdi");
                println!("  mov rax, rdx");
            }
            NodeKind::BitAnd | NodeKind::BitAndAssign => {
                println!("  and rax, rdi");
            }
            NodeKind::BitOr | NodeKind::BitOrAssign => {
                println!("  or rax, rdi");
            }
            NodeKind::BitXor | NodeKind::BitXorAssign => {
                println!("  xor rax, rdi");
            }
            NodeKind::Shl | NodeKind::ShlAssign => {
                println!("  mov cl, dil");
                println!("  shl rax, cl");
            }
            NodeKind::Shr | NodeKind::ShrAssign => {
                println!("  mov cl, dil");
                println!("  shr rax, cl");
            }
            NodeKind::Eq => {
                println!("  cmp rax, rdi");
                println!("  sete al");
                println!("  movzb rax, al");
            }
            NodeKind::Ne => {
                println!("  cmp rax, rdi");
                println!("  setne al");
                println!("  movzb rax, al");
            }
            NodeKind::Lt => {
                println!("  cmp rax, rdi");
                println!("  setl al");
                println!("  movzb rax, al");
            }
            NodeKind::Le => {
                println!("  cmp rax, rdi");
                println!("  setle al");
                println!("  movzb rax, al");
            }
            _ => {}
        }
        println!("  push rax");
    }
}
