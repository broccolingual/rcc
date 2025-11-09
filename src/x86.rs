use crate::ast::{Ast, Node, NodeKind};

pub struct Generator {
    label_seq: usize,
    break_seq: usize,
    continue_seq: usize,
    func_name: String,
}

impl Generator {
    pub fn new() -> Self {
        Generator {
            label_seq: 1,
            break_seq: 0,
            continue_seq: 0,
            func_name: String::new(),
        }
    }

    pub fn gen_asm(&mut self, ast: &Ast) {
        println!(".intel_syntax noprefix"); // おまじない

        println!(".text");
        for func in ast.funcs.iter() {
            self.func_name = func.name.clone();
            println!(".globl {}", self.func_name); // 関数をグローバルシンボルとして宣言
            println!("{}:", self.func_name); // 関数ラベル

            // 関数プロローグ
            println!("  push rbp");
            println!("  mov rbp, rsp");
            println!("  sub rsp, 208"); // 変数26個分の領域を確保

            // 引数を逆順でスタックに読み出し
            let arg_regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
            for (i, arg) in func.args.iter().enumerate().rev() {
                println!("  mov [rbp-{}], {}", arg.offset, arg_regs[i]);
            }

            // 関数本体のコード生成
            for node in func.nodes.iter() {
                self.gen_asm_from_expr(node);
            }

            // 関数エピローグ
            println!(".L.return.{}:", self.func_name);
            println!("  mov rsp, rbp");
            println!("  pop rbp");
            println!("  ret");
        }
    }

    pub fn gen_asm_from_lval(&self, node: &Node) {
        if node.kind != NodeKind::LVar {
            panic!("代入の左辺値が変数ではありません");
        }
        println!("  lea rax, [rbp-{}]", node.offset);
        println!("  push rax");
    }

    // スタックトップのアドレスから値を読み出してスタックに積む
    fn load(&self) {
        println!("  pop rax");
        println!("  mov rax, [rax]");
        println!("  push rax");
    }

    // スタックトップの値をアドレスに格納する
    fn store(&self) {
        println!("  pop rdi");
        println!("  pop rax");
        println!("  mov [rax], rdi");
        println!("  push rdi");
    }

    // int を 1 加算
    fn inc(&self) {
        println!("  pop rax");
        println!("  add rax, 1");
        println!("  push rax");
    }

    // int を 1 減算
    fn dec(&self) {
        println!("  pop rax");
        println!("  sub rax, 1");
        println!("  push rax");
    }

    pub fn gen_asm_from_expr(&mut self, node: &Node) {
        match node.kind {
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
            NodeKind::PreInc => {
                self.gen_asm_from_lval(node.lhs.as_ref().unwrap());
                println!("  push [rsp]");
                self.load();
                self.inc();
                self.store();
                return;
            }
            NodeKind::PreDec => {
                self.gen_asm_from_lval(node.lhs.as_ref().unwrap());
                println!("  push [rsp]");
                self.load();
                self.dec();
                self.store();
                return;
            }
            NodeKind::PostInc => {
                self.gen_asm_from_lval(node.lhs.as_ref().unwrap());
                println!("  push [rsp]");
                self.load();
                self.inc();
                self.store();
                self.dec();
                return;
            }
            NodeKind::PostDec => {
                self.gen_asm_from_lval(node.lhs.as_ref().unwrap());
                println!("  push [rsp]");
                self.load();
                self.dec();
                self.store();
                self.inc();
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
                self.gen_asm_from_lval(node.lhs.as_ref().unwrap());
                println!("  push [rsp]");
                self.load();
                self.gen_asm_from_expr(node.rhs.as_ref().unwrap());
                self.gen_asm_from_binary_op(node);
                self.store();
                return;
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
                for stmt in node.body.iter() {
                    self.gen_asm_from_expr(stmt);
                    println!("  pop rax"); // ブロック内の各文の結果を捨てる
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
            NodeKind::Call => {
                let arg_count = node.args.len();

                if arg_count > 6 {
                    panic!("6個を超える引数の関数呼び出しには対応していません");
                }

                // 引数をスタックに積む
                for arg in node.args.iter() {
                    self.gen_asm_from_expr(arg);
                }

                // 引数を逆順でレジスタに移動
                let arg_regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
                for i in (0..arg_count).rev() {
                    println!("  pop {}", arg_regs[i]);
                }

                // 関数呼び出し
                // アラインメントを保つためにrspを調整
                let seq = self.label_seq;
                self.label_seq += 1;
                println!("  mov rax, rsp"); // 現在のrspをraxにコピー
                println!("  and rax, 15"); // rspを16の倍数にする
                println!("  jnz .Lcall{}", seq); // もし16の倍数でなければ調整
                println!("  mov rax, 0"); // ダミーのrax設定
                println!("  call {}", node.func_name); // 関数呼び出し
                println!("  jmp .Lend{}", seq);
                println!(".Lcall{}:", seq); // 16の倍数でない場合の処理
                println!("  sub rsp, 8"); // スタックを8バイト下げる
                println!("  mov rax, 0"); // ダミーのrax設定
                println!("  call {}", node.func_name); // 関数呼び出し
                println!("  add rsp, 8"); // スタックを元に戻す
                println!(".Lend{}:", seq);
                println!("  push rax"); // 戻り値をスタックに積む
                return;
            }
            NodeKind::Return => {
                if node.lhs.is_some() {
                    self.gen_asm_from_expr(node.lhs.as_ref().unwrap());
                    println!("  pop rax");
                }
                println!("  jmp .L.return.{}", self.func_name);
                return;
            }
            _ => {}
        }

        self.gen_asm_from_expr(node.lhs.as_ref().unwrap());
        self.gen_asm_from_expr(node.rhs.as_ref().unwrap());
        self.gen_asm_from_binary_op(node);
    }

    fn gen_asm_from_binary_op(&self, node: &Node) {
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
