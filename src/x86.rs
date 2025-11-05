use crate::ast::{Node, NodeKind};

pub struct Generator {
    label_seq: usize,
}

impl Generator {
    pub fn new() -> Self {
        Generator { label_seq: 0 }
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
            NodeKind::If => {
                let seq = self.label_seq;
                self.label_seq += 1;
                if node.els.is_some() {
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
                println!(".Lbegin{}:", seq);
                self.gen_asm_from_expr(node.cond.as_ref().unwrap());
                println!("  pop rax");
                println!("  cmp rax, 0");
                println!("  je .Lend{}", seq);
                self.gen_asm_from_expr(node.then.as_ref().unwrap());
                println!("  jmp .Lbegin{}", seq);
                println!(".Lend{}:", seq);
                return;
            }
            NodeKind::For => {
                let seq = self.label_seq;
                self.label_seq += 1;
                if let Some(init) = node.init.as_ref() {
                    self.gen_asm_from_expr(init);
                }
                println!(".Lbegin{}:", seq);
                if let Some(cond) = node.cond.as_ref() {
                    self.gen_asm_from_expr(cond);
                    println!("  pop rax");
                    println!("  cmp rax, 0");
                    println!("  je .Lend{}", seq);
                }
                self.gen_asm_from_expr(node.then.as_ref().unwrap());
                if let Some(inc) = node.inc.as_ref() {
                    self.gen_asm_from_expr(inc);
                }
                println!("  jmp .Lbegin{}", seq);
                println!(".Lend{}:", seq);
                return;
            }
            NodeKind::Do => {
                let seq = self.label_seq;
                self.label_seq += 1;
                println!(".Lbegin{}:", seq);
                self.gen_asm_from_expr(node.then.as_ref().unwrap());
                self.gen_asm_from_expr(node.cond.as_ref().unwrap());
                println!("  pop rax");
                println!("  cmp rax, 0");
                println!("  jne .Lbegin{}", seq);
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
            _ => {}
        }

        self.gen_asm_from_expr(node.lhs.as_ref().unwrap());
        self.gen_asm_from_expr(node.rhs.as_ref().unwrap());

        println!("  pop rdi");
        println!("  pop rax");

        match node.kind {
            NodeKind::Add => println!("  add rax, rdi"),
            NodeKind::Sub => println!("  sub rax, rdi"),
            NodeKind::Mul => println!("  imul rax, rdi"),
            NodeKind::Div => {
                println!("  cqo");
                println!("  idiv rdi");
            }
            NodeKind::Rem => {
                println!("  cqo");
                println!("  idiv rdi");
                println!("  mov rax, rdx");
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
