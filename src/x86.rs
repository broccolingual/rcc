use crate::asm_builder::AsmBuilder;
use crate::ast::Ast;
use crate::node::{Node, NodeKind};
use crate::types::TypeKind;

const ARG_BYTE_REGS: [&str; 6] = ["dil", "sil", "dl", "cl", "r8b", "r9b"];
const ARG_WORD_REGS: [&str; 6] = ["di", "si", "dx", "cx", "r8w", "r9w"];
const ARG_DWORD_REGS: [&str; 6] = ["edi", "esi", "edx", "ecx", "r8d", "r9d"];
const ARG_QWORD_REGS: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

pub struct Generator {
    label_seq: usize,
    break_seq: usize,
    continue_seq: usize,
    func_name: String,
    pub builder: AsmBuilder,
}

impl Default for Generator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator {
    pub fn new() -> Self {
        Generator {
            label_seq: 1,
            break_seq: 0,
            continue_seq: 0,
            func_name: String::new(),
            builder: AsmBuilder::new(),
        }
    }

    pub fn gen_asm(&mut self, ast: &Ast) {
        self.builder.add_row(".intel_syntax noprefix", true);
        self.builder.add_row(".text", true);

        // グローバル変数の定義
        self.builder.add_row(".bss", true);
        for gvar in ast.globals.iter() {
            self.builder.add_row(&format!(".globl {}", gvar.name), true);
            self.builder.add_row(".align 8", true); // TODO: アラインメントは仮で8に固定
            self.builder
                .add_row(&format!(".type {}, @object", gvar.name), true);
            self.builder.add_row(
                &format!(".size {}, {}", gvar.name, gvar.ty.actual_size_of()),
                true,
            );
            self.builder.add_row(&format!("{}:", gvar.name), false);
            self.builder
                .add_row(&format!(".zero {}", gvar.ty.actual_size_of()), true);
        }

        // 文字列リテラルの定義
        self.builder.add_row(".data", true);
        for (i, string) in ast.string_literals.iter().enumerate() {
            self.builder.add_row(&format!(".L.str.{}:", i), false);
            self.builder
                .add_row(&format!(".string \"{}\"", string), true);
        }

        // 関数の定義
        self.builder.add_row(".text", true);
        for func in ast.funcs.iter() {
            self.func_name = func.name.clone();
            self.builder
                .add_row(&format!(".globl {}", self.func_name), true);
            self.builder
                .add_row(&format!(".type {}, @function", self.func_name), true);
            self.builder.add_row(&format!("{}:", self.func_name), false);

            // 関数プロローグ
            self.builder.add_row("push rbp", true);
            self.builder.add_row("mov rbp, rsp", true);

            // 関数のローカル変数に対応するスタック領域を確保
            // ローカル変数の最大オフセットに基づいてスタック領域を計算
            let max_offset = func.locals.first().map_or(0, |arg| arg.offset);
            let stack_size = ((max_offset + 15) / 16) * 16; // 16バイトアラインメント
            self.builder
                .add_row(&format!("sub rsp, {}", stack_size), true);

            // 引数を逆順でスタックから読み出し
            for (i, arg) in func.locals.iter().rev().enumerate() {
                match arg.ty.size_of() {
                    1 => {
                        self.builder.add_row(
                            &format!("  mov [rbp-{}], {}", arg.offset, ARG_BYTE_REGS[i]),
                            true,
                        );
                    }
                    2 => {
                        self.builder.add_row(
                            &format!("  mov [rbp-{}], {}", arg.offset, ARG_WORD_REGS[i]),
                            true,
                        );
                    }
                    4 => {
                        self.builder.add_row(
                            &format!("  mov [rbp-{}], {}", arg.offset, ARG_DWORD_REGS[i]),
                            true,
                        );
                    }
                    8 => {
                        self.builder.add_row(
                            &format!("  mov [rbp-{}], {}", arg.offset, ARG_QWORD_REGS[i]),
                            true,
                        );
                    }
                    _ => panic!("未対応の引数サイズ: {}", arg.ty.size_of()),
                }
            }

            // 関数本体のコード生成
            for node in func.body.iter() {
                self.gen_asm_from_expr(node);
            }

            // 関数エピローグ
            self.builder
                .add_row(&format!(".L.return.{}:", self.func_name), false);
            self.builder.add_row("mov rsp, rbp", true);
            self.builder.add_row("pop rbp", true);
            self.builder.add_row("ret", true);
        }
        // スタックを実行不可に設定
        self.builder
            .add_row(".section .note.GNU-stack,\"\",@progbits", true);
    }

    pub fn get_val(&mut self, node: &Node) {
        match node.kind {
            NodeKind::Deref => {
                self.gen_asm_from_expr(node.lhs.as_ref().unwrap());
            }
            NodeKind::LVar => {
                self.builder
                    .add_row(&format!("lea rax, [rbp-{}]", node.offset), true);
                self.builder.add_row("push rax", true);
            }
            NodeKind::GVar => {
                self.builder
                    .add_row(&format!("lea rax, {}[rip]", node.name), true);
                self.builder.add_row("push rax", true);
            }
            _ => panic!("代入の左辺値が変数ではありません: {:?}", node.kind),
        }
    }

    // スタックトップのアドレスから値を読み出してスタックに積む
    fn load(&mut self, node: &Node) {
        self.builder.add_row("pop rax", true);
        if node.ty.is_none() {
            panic!("load先の型情報がありません: {:?}", node);
        }
        match node.ty.as_ref().unwrap().size_of() {
            1 => {
                self.builder.add_row("movsx rax, BYTE PTR [rax]", true);
            }
            2 => {
                self.builder.add_row("movsx rax, WORD PTR [rax]", true);
            }
            4 => {
                self.builder.add_row("movsxd rax, DWORD PTR [rax]", true);
            }
            8 => {
                self.builder.add_row("mov rax, QWORD PTR [rax]", true);
            }
            _ => panic!(
                "未対応のロードサイズ: {}",
                node.ty.as_ref().unwrap().size_of()
            ),
        }
        self.builder.add_row("push rax", true);
    }

    // スタックトップの値をアドレスに格納する
    fn store(&mut self, node: &Node) {
        self.builder.add_row("pop rdi", true);
        self.builder.add_row("pop rax", true);
        if node.ty.is_none() {
            panic!("store先の型情報がありません: {:?}", node);
        }
        match node.ty.as_ref().unwrap().size_of() {
            1 => {
                self.builder.add_row("mov BYTE PTR [rax], dil", true);
            }
            2 => {
                self.builder.add_row("mov WORD PTR [rax], di", true);
            }
            4 => {
                self.builder.add_row("mov DWORD PTR [rax], edi", true);
            }
            8 => {
                self.builder.add_row("mov QWORD PTR [rax], rdi", true);
            }
            _ => panic!(
                "未対応のストアサイズ: {}",
                node.ty.as_ref().unwrap().size_of()
            ),
        }
        self.builder.add_row("push rdi", true);
    }

    // int を 1 加算
    fn inc(&mut self) {
        self.builder.add_row("pop rax", true);
        self.builder.add_row("add rax, 1", true);
        self.builder.add_row("push rax", true);
    }

    // int を 1 減算
    fn dec(&mut self) {
        self.builder.add_row("pop rax", true);
        self.builder.add_row("sub rax, 1", true);
        self.builder.add_row("push rax", true);
    }

    pub fn gen_asm_from_expr(&mut self, node: &Node) {
        match node.kind {
            NodeKind::Nop => {
                return;
            }
            NodeKind::Number => {
                self.builder.add_row(&format!("push {}", node.val), true);
                return;
            }
            NodeKind::String => {
                self.builder
                    .add_row(&format!("lea rax, .L.str.{}[rip]", node.offset), true);
                self.builder.add_row("push rax", true);
                return;
            }
            NodeKind::LVar | NodeKind::GVar => {
                self.get_val(node);
                if node.ty.as_ref().unwrap().kind != TypeKind::Array {
                    self.load(node); // 配列型以外は値を読み出す
                }
                return;
            }
            NodeKind::Assign => {
                self.get_val(node.lhs.as_ref().unwrap());
                self.gen_asm_from_expr(node.rhs.as_ref().unwrap());
                self.store(node.lhs.as_ref().unwrap());
                return;
            }
            NodeKind::Ternary => {
                let seq = self.label_seq;
                self.label_seq += 1;
                self.gen_asm_from_expr(node.cond.as_ref().unwrap());
                self.builder.add_row("pop rax", true);
                self.builder.add_row("cmp rax, 0", true);
                self.builder.add_row(&format!("je .L.else.{}", seq), true);
                self.gen_asm_from_expr(node.then.as_ref().unwrap());
                self.builder.add_row(&format!("jmp .L.end.{}", seq), true);
                self.builder.add_row(&format!(".L.else.{}:", seq), false);
                self.gen_asm_from_expr(node.els.as_ref().unwrap());
                self.builder.add_row(&format!(".L.end.{}:", seq), false);
                return;
            }
            NodeKind::PreInc => {
                self.get_val(node.lhs.as_ref().unwrap());
                self.builder.add_row("push [rsp]", true);
                self.load(node.lhs.as_ref().unwrap());
                self.inc();
                self.store(node.lhs.as_ref().unwrap());
                return;
            }
            NodeKind::PreDec => {
                self.get_val(node.lhs.as_ref().unwrap());
                self.builder.add_row("push [rsp]", true);
                self.load(node.lhs.as_ref().unwrap());
                self.dec();
                self.store(node.lhs.as_ref().unwrap());
                return;
            }
            NodeKind::PostInc => {
                self.get_val(node.lhs.as_ref().unwrap());
                self.builder.add_row("push [rsp]", true);
                self.load(node.lhs.as_ref().unwrap());
                self.inc();
                self.store(node.lhs.as_ref().unwrap());
                self.dec();
                return;
            }
            NodeKind::PostDec => {
                self.get_val(node.lhs.as_ref().unwrap());
                self.builder.add_row("push [rsp]", true);
                self.load(node.lhs.as_ref().unwrap());
                self.dec();
                self.store(node.lhs.as_ref().unwrap());
                self.inc();
                return;
            }
            NodeKind::AddAssign
            | NodeKind::SubAssign
            | NodeKind::MulAssign
            | NodeKind::DivAssign
            | NodeKind::RemAssign
            | NodeKind::BitAndAssign
            | NodeKind::BitOrAssign
            | NodeKind::BitXorAssign
            | NodeKind::ShlAssign
            | NodeKind::ShrAssign => {
                self.get_val(node.lhs.as_ref().unwrap());
                self.builder.add_row("push [rsp]", true);
                self.load(node.lhs.as_ref().unwrap());
                self.gen_asm_from_expr(node.rhs.as_ref().unwrap());
                self.gen_asm_from_binary_op(node);
                self.store(node.lhs.as_ref().unwrap());
                return;
            }
            NodeKind::LogicalNot => {
                self.gen_asm_from_expr(node.lhs.as_ref().unwrap());
                self.builder.add_row("pop rax", true);
                self.builder.add_row("cmp rax, 0", true);
                self.builder.add_row("sete al", true);
                self.builder.add_row("movzb rax, al", true);
                self.builder.add_row("push rax", true);
                return;
            }
            NodeKind::BitNot => {
                self.gen_asm_from_expr(node.lhs.as_ref().unwrap());
                self.builder.add_row("pop rax", true);
                self.builder.add_row("not rax", true);
                self.builder.add_row("push rax", true);
                return;
            }
            NodeKind::Addr => {
                self.get_val(node.lhs.as_ref().unwrap());
                return;
            }
            NodeKind::Deref => {
                self.gen_asm_from_expr(node.lhs.as_ref().unwrap());
                self.load(node.lhs.as_ref().unwrap());
                return;
            }
            NodeKind::LogicalAnd => {
                let seq = self.label_seq;
                self.label_seq += 1;
                self.gen_asm_from_expr(node.lhs.as_ref().unwrap());
                self.builder.add_row("pop rax", true);
                self.builder.add_row("cmp rax, 0", true);
                self.builder.add_row(&format!("je .L.false.{}", seq), true);
                self.gen_asm_from_expr(node.rhs.as_ref().unwrap());
                self.builder.add_row("pop rax", true);
                self.builder.add_row("cmp rax, 0", true);
                self.builder.add_row(&format!("je .L.false.{}", seq), true);
                self.builder.add_row("push 1", true);
                self.builder.add_row(&format!("jmp .L.end.{}", seq), true);
                self.builder.add_row(&format!(".L.false.{}:", seq), false);
                self.builder.add_row("push 0", true);
                self.builder.add_row(&format!(".L.end.{}:", seq), false);
                return;
            }
            NodeKind::LogicalOr => {
                let seq = self.label_seq;
                self.label_seq += 1;
                self.gen_asm_from_expr(node.lhs.as_ref().unwrap());
                self.builder.add_row("pop rax", true);
                self.builder.add_row("cmp rax, 0", true);
                self.builder.add_row(&format!("jne .L.true.{}", seq), true);
                self.gen_asm_from_expr(node.rhs.as_ref().unwrap());
                self.builder.add_row("pop rax", true);
                self.builder.add_row("cmp rax, 0", true);
                self.builder.add_row(&format!("jne .L.true.{}", seq), true);
                self.builder.add_row("push 0", true);
                self.builder.add_row(&format!("jmp .L.end.{}", seq), true);
                self.builder.add_row(&format!(".L.true.{}:", seq), false);
                self.builder.add_row("push 1", true);
                self.builder.add_row(&format!(".L.end.{}:", seq), false);
                return;
            }
            NodeKind::If => {
                let seq = self.label_seq;
                self.label_seq += 1;
                if node.els.is_some() {
                    // else節あり
                    self.gen_asm_from_expr(node.cond.as_ref().unwrap());
                    self.builder.add_row("pop rax", true);
                    self.builder.add_row("cmp rax, 0", true);
                    self.builder.add_row(&format!("je .L.else.{}", seq), true);
                    self.gen_asm_from_expr(node.then.as_ref().unwrap());
                    self.builder.add_row(&format!("jmp .L.end.{}", seq), true);
                    self.builder.add_row(&format!(".L.else.{}:", seq), false);
                    self.gen_asm_from_expr(node.els.as_ref().unwrap());
                    self.builder.add_row(&format!(".L.end.{}:", seq), false);
                    self.builder.add_row("push rax", true); // then節またはelse節の結果をスタックに積む
                } else {
                    // else節なし
                    self.gen_asm_from_expr(node.cond.as_ref().unwrap());
                    self.builder.add_row("pop rax", true);
                    self.builder.add_row("cmp rax, 0", true);
                    self.builder.add_row(&format!("je .L.end.{}", seq), true);
                    self.gen_asm_from_expr(node.then.as_ref().unwrap());
                    self.builder.add_row(&format!(".L.end.{}:", seq), false);
                    self.builder.add_row("push rax", true); // then節の結果をスタックに積む
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

                self.builder
                    .add_row(&format!(".L.continue.{}:", seq), false);
                self.gen_asm_from_expr(node.cond.as_ref().unwrap());
                self.builder.add_row("pop rax", true);
                self.builder.add_row("cmp rax, 0", true);
                self.builder.add_row(&format!("je .L.break.{}", seq), true);
                self.gen_asm_from_expr(node.then.as_ref().unwrap());
                self.builder
                    .add_row(&format!("jmp .L.continue.{}", seq), true);
                self.builder.add_row(&format!(".L.break.{}:", seq), false);

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
                self.builder.add_row(&format!(".L.begin.{}:", seq), false);
                if let Some(cond) = node.cond.as_ref() {
                    self.gen_asm_from_expr(cond);
                    self.builder.add_row("pop rax", true);
                    self.builder.add_row("cmp rax, 0", true);
                    self.builder.add_row(&format!("je .L.break.{}", seq), true);
                }
                self.gen_asm_from_expr(node.then.as_ref().unwrap());
                self.builder
                    .add_row(&format!(".L.continue.{}:", seq), false);
                if let Some(inc) = node.inc.as_ref() {
                    self.gen_asm_from_expr(inc);
                }
                self.builder.add_row(&format!("jmp .L.begin.{}", seq), true);
                self.builder.add_row(&format!(".L.break.{}:", seq), false);

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

                self.builder.add_row(&format!(".L.begin.{}:", seq), false);
                self.gen_asm_from_expr(node.then.as_ref().unwrap());
                self.builder
                    .add_row(&format!(".L.continue.{}:", seq), false);
                self.gen_asm_from_expr(node.cond.as_ref().unwrap());
                self.builder.add_row("pop rax", true);
                self.builder.add_row("cmp rax, 0", true);
                self.builder.add_row(&format!("jne .L.begin.{}", seq), true);
                self.builder.add_row(&format!(".L.break.{}:", seq), false);

                self.break_seq = current_break_seq;
                self.continue_seq = current_continue_seq;
                return;
            }
            NodeKind::Block => {
                for stmt in node.body.iter() {
                    self.gen_asm_from_expr(stmt);
                    self.builder.add_row("pop rax", true); // ブロック内の各文の結果を捨てる
                }
                return;
            }
            NodeKind::Break => {
                self.builder
                    .add_row(&format!("jmp .L.break.{}", self.break_seq), true);
                return;
            }
            NodeKind::Continue => {
                self.builder
                    .add_row(&format!("jmp .L.continue.{}", self.continue_seq), true);
                return;
            }
            NodeKind::Goto => {
                self.builder.add_row(
                    &format!("jmp .L.label.{}.{}", self.func_name, node.name),
                    true,
                );
                return;
            }
            NodeKind::Label => {
                self.builder.add_row(
                    &format!(".L.label.{}.{}:", self.func_name, node.name),
                    false,
                );
                self.gen_asm_from_expr(node.lhs.as_ref().unwrap());
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

                // 引数をレジスタに移動
                for reg in ARG_QWORD_REGS.iter().take(arg_count) {
                    self.builder.add_row(&format!("pop {}", reg), true);
                }

                self.builder.add_row("mov al, 0", true); // 浮動小数点は使わないので0に設定

                // 関数呼び出し
                // アラインメントを保つためにrspを調整
                let seq = self.label_seq;
                self.label_seq += 1;
                self.builder.add_row("mov rax, rsp", true); // 現在のrspをraxにコピー
                self.builder.add_row("and rax, 15", true); // rspを16
                self.builder.add_row(&format!("jnz .L.align.{}", seq), true); // もし16の倍数でなければ調整
                self.builder.add_row("mov rax, 0", true); // ダミーのrax設定
                self.builder.add_row(&format!("call {}", node.name), true); // 関数呼び出し
                self.builder.add_row(&format!("jmp .L.end.{}", seq), true);
                self.builder.add_row(&format!(".L.align.{}:", seq), false); // 16の倍数でない場合の処理
                self.builder.add_row("sub rsp, 8", true); // スタックを8バイト下げる
                self.builder.add_row("mov rax, 0", true); // ダミーのrax設定
                self.builder.add_row(&format!("call {}", node.name), true); // 関数呼び出し
                self.builder.add_row("add rsp, 8", true); // スタックを元に戻す
                self.builder.add_row(&format!(".L.end.{}:", seq), false);
                self.builder.add_row("push rax", true); // 戻り値をスタックに積む
                return;
            }
            NodeKind::Return => {
                if node.lhs.is_some() {
                    self.gen_asm_from_expr(node.lhs.as_ref().unwrap());
                    self.builder.add_row("pop rax", true);
                }
                self.builder
                    .add_row(&format!("jmp .L.return.{}", self.func_name), true);
                return;
            }
            _ => {}
        }

        self.gen_asm_from_expr(node.lhs.as_ref().unwrap());
        self.gen_asm_from_expr(node.rhs.as_ref().unwrap());
        self.gen_asm_from_binary_op(node);
    }

    fn gen_asm_from_binary_op(&mut self, node: &Node) {
        self.builder.add_row("pop rdi", true);
        self.builder.add_row("pop rax", true);

        match node.kind {
            NodeKind::Add | NodeKind::AddAssign => self.builder.add_row("add rax, rdi", true),
            NodeKind::Sub | NodeKind::SubAssign => self.builder.add_row("sub rax, rdi", true),
            NodeKind::Mul | NodeKind::MulAssign => self.builder.add_row("imul rax, rdi", true),
            NodeKind::Div | NodeKind::DivAssign => {
                self.builder.add_row("cqo", true);
                self.builder.add_row("idiv rdi", true);
            }
            NodeKind::Rem | NodeKind::RemAssign => {
                self.builder.add_row("cqo", true);
                self.builder.add_row("idiv rdi", true);
                self.builder.add_row("mov rax, rdx", true);
            }
            NodeKind::BitAnd | NodeKind::BitAndAssign => {
                self.builder.add_row("and rax, rdi", true);
            }
            NodeKind::BitOr | NodeKind::BitOrAssign => {
                self.builder.add_row("or rax, rdi", true);
            }
            NodeKind::BitXor | NodeKind::BitXorAssign => {
                self.builder.add_row("xor rax, rdi", true);
            }
            NodeKind::Shl | NodeKind::ShlAssign => {
                self.builder.add_row("mov cl, dil", true);
                self.builder.add_row("shl rax, cl", true);
            }
            NodeKind::Shr | NodeKind::ShrAssign => {
                self.builder.add_row("mov cl, dil", true);
                self.builder.add_row("shr rax, cl", true);
            }
            NodeKind::Eq => {
                self.builder.add_row("cmp rax, rdi", true);
                self.builder.add_row("sete al", true);
                self.builder.add_row("movzb rax, al", true);
            }
            NodeKind::Ne => {
                self.builder.add_row("cmp rax, rdi", true);
                self.builder.add_row("setne al", true);
                self.builder.add_row("movzb rax, al", true);
            }
            NodeKind::Lt => {
                self.builder.add_row("cmp rax, rdi", true);
                self.builder.add_row("setl al", true);
                self.builder.add_row("movzb rax, al", true);
            }
            NodeKind::Le => {
                self.builder.add_row("cmp rax, rdi", true);
                self.builder.add_row("setle al", true);
                self.builder.add_row("movzb rax, al", true);
            }
            _ => {}
        }
        self.builder.add_row("push rax", true);
    }
}
