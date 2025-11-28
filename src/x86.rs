use std::ops::Deref;

use crate::asm_builder::AsmBuilder;
use crate::ast::Ast;
use crate::node::{Node, NodeKind};

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

    fn emit_prologue(&mut self) {
        self.builder.add_row(".intel_syntax noprefix", true);
        self.builder.add_row(".text", true);
    }

    fn emit_rodata(&mut self, ast: &Ast) {
        if ast.string_literals.is_empty() {
            return;
        }
        self.builder.add_row(".section .rodata", true);
        for (i, string) in ast.string_literals.iter().enumerate() {
            self.builder.add_row(&format!(".L.str.{}:", i), false);
            self.builder
                .add_row(&format!(".string \"{}\"", string), true);
        }
    }

    fn emit_data(&mut self, ast: &Ast) {
        if ast.globals.is_empty() {
            return;
        }
        self.builder.add_row(".data", true);
        for gvar in ast.globals.iter() {
            self.builder.add_row(&format!(".globl {}", gvar.name), true);
            self.builder
                .add_row(&format!(".align {}", gvar.ty.align_of()), true);
            self.builder
                .add_row(&format!(".type {}, @object", gvar.name), true);
            self.builder
                .add_row(&format!(".size {}, {}", gvar.name, gvar.ty.size_of()), true);
            self.builder.add_row(&format!("{}:", gvar.name), false);
            if let Some(init) = gvar.init.as_ref() {
                match init.kind {
                    NodeKind::Number { val } => match gvar.ty.size_of() {
                        1 => {
                            self.builder.add_row(&format!(".byte {}", val), true);
                        }
                        2 => {
                            self.builder.add_row(&format!(".word {}", val), true);
                        }
                        4 => {
                            self.builder.add_row(&format!(".long {}", val), true);
                        }
                        8 => {
                            self.builder.add_row(&format!(".quad {}", val), true);
                        }
                        _ => panic!("未対応のグローバル変数初期化サイズ: {}", gvar.ty.size_of()),
                    },
                    NodeKind::Addr => {
                        if let Some(lhs) = &init.lhs {
                            match &lhs.kind {
                                NodeKind::GVar { name } => {
                                    self.builder.add_row(&format!(".quad {}", name), true);
                                }
                                _ => {
                                    panic!(
                                        "未対応のグローバル変数初期化式のアドレス指定: {:?}",
                                        lhs.kind
                                    );
                                }
                            }
                        }
                    }
                    NodeKind::String { index, .. } => {
                        self.builder
                            .add_row(&format!(".quad .L.str.{}", index), true);
                    }
                    _ => panic!("未対応のグローバル変数初期化式: {:?}", init.kind),
                }
            } else {
                self.builder
                    .add_row(&format!(".zero {}", gvar.ty.size_of()), true);
            }
        }
    }

    fn emit_epilogue(&mut self) {
        self.builder
            .add_row(".section .note.GNU-stack,\"\",@progbits", true); // スタックを実行不可にする
    }

    // ASTからアセンブリコードを生成
    pub fn gen_asm(&mut self, ast: &Ast) {
        self.emit_prologue();
        self.emit_rodata(ast); // 文字列リテラルの定義
        self.emit_data(ast); // グローバル変数の定義

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
            if stack_size > 0 {
                self.builder
                    .add_row(&format!("sub rsp, {}", stack_size), true);
            }

            // ローカル変数を逆順でスタックから読み出し
            for (i, arg) in func.locals.iter().rev().enumerate() {
                match arg.ty.align_of() {
                    1 => {
                        self.builder.add_row(
                            &format!("  mov [rbp-{}], {}", arg.offset, ARG_BYTE_REGS[i]), // 1バイト
                            true,
                        );
                    }
                    2 => {
                        self.builder.add_row(
                            &format!("  mov [rbp-{}], {}", arg.offset, ARG_WORD_REGS[i]), // 2バイト
                            true,
                        );
                    }
                    4 => {
                        self.builder.add_row(
                            &format!("  mov [rbp-{}], {}", arg.offset, ARG_DWORD_REGS[i]), // 4バイト
                            true,
                        );
                    }
                    8 => {
                        self.builder.add_row(
                            &format!("  mov [rbp-{}], {}", arg.offset, ARG_QWORD_REGS[i]), // 8バイト
                            true,
                        );
                    }
                    _ => panic!("未対応の引数サイズ: {}", arg.ty.align_of()),
                }

                // initializerがある場合、初期化コードを生成
                if let Some(init) = arg.init.as_ref() {
                    self.gen_addr(&Node {
                        kind: NodeKind::LVar {
                            name: arg.name.clone(),
                            offset: arg.offset,
                        },
                        ..Default::default()
                    }); // 変数のアドレスをスタックに積む
                    self.gen_expr(init); // 初期化式のコードを生成し、スタックに値を積む
                    self.store(&Node {
                        ty: Some(arg.ty.clone()),
                        ..Default::default()
                    }); // スタックトップの値を変数に格納
                }
            }

            // 関数本体のコード生成
            for node in func.body.iter() {
                if node.is_expr() {
                    self.gen_expr(node);
                    self.builder.add_row("pop rax", true); // 式の結果を捨てる
                } else {
                    self.gen_stmt(node);
                }
            }

            // 関数エピローグ
            self.builder
                .add_row(&format!(".L.return.{}:", self.func_name), false);
            self.builder.add_row("leave", true);
            self.builder.add_row("ret", true);
        }
        self.emit_epilogue();
    }

    // 変数やデリファレンスのアドレスをスタックに積む
    fn gen_addr(&mut self, node: &Node) {
        match &node.kind {
            NodeKind::Deref => {
                self.gen_expr(node.lhs.as_ref().unwrap()); // ポインタの値を取得
            }
            NodeKind::LVar { offset, .. } => {
                self.builder
                    .add_row(&format!("lea rax, [rbp-{}]", offset), true); // ローカル変数のアドレスを計算して取得
                self.builder.add_row("push rax", true); // 変数のアドレスをスタックに積む
            }
            NodeKind::GVar { name } => {
                self.builder
                    .add_row(&format!("lea rax, {}[rip]", name), true); // RIP相対アドレッシングでアドレスを取得
                self.builder.add_row("push rax", true); // 変数のアドレスをスタックに積む
            }
            _ => panic!("代入の左辺値が変数ではありません: {:?}", node.kind),
        }
    }

    // スタックトップのアドレスから値を読み出してスタックに積む
    fn load(&mut self, node: &Node) {
        self.builder.add_row("pop rax", true); // ロード先のアドレス
        if node.ty.is_none() {
            panic!("load先の型情報がありません: {:?}", node);
        }
        match node.ty.as_ref().unwrap().align_of() {
            1 => {
                self.builder.add_row("movsx rax, BYTE PTR [rax]", true); // 1バイト
            }
            2 => {
                self.builder.add_row("movsx rax, WORD PTR [rax]", true); // 2バイト
            }
            4 => {
                self.builder.add_row("movsxd rax, DWORD PTR [rax]", true); // 4バイト
            }
            8 => {
                self.builder.add_row("mov rax, QWORD PTR [rax]", true); // 8バイト
            }
            _ => panic!(
                "未対応のロードサイズ: {}",
                node.ty.as_ref().unwrap().align_of()
            ),
        }
        self.builder.add_row("push rax", true); // 読み出した値をスタックに積む
    }

    // スタックトップの値をアドレスに格納する
    fn store(&mut self, node: &Node) {
        self.builder.add_row("pop rdi", true); // ストアする値
        self.builder.add_row("pop rax", true); // ストア先のアドレス
        if node.ty.is_none() {
            panic!("store先の型情報がありません: {:?}", node);
        }
        match node.ty.as_ref().unwrap().align_of() {
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
                node.ty.as_ref().unwrap().align_of()
            ),
        }
        self.builder.add_row("push rdi", true); // ストアした値をスタックに戻す
    }

    // int を 1 加算
    fn inc(&mut self) {
        self.builder.add_row("pop rax", true);
        self.builder.add_row("inc rax", true);
        self.builder.add_row("push rax", true);
    }

    // int を 1 減算
    fn dec(&mut self) {
        self.builder.add_row("pop rax", true);
        self.builder.add_row("dec rax", true);
        self.builder.add_row("push rax", true);
    }

    // 文のコード生成
    fn gen_stmt(&mut self, node: &Node) {
        match &node.kind {
            NodeKind::If { cond, then, els } => {
                let seq = self.label_seq;
                self.label_seq += 1;
                if els.is_some() {
                    // else節あり
                    self.gen_expr(cond.as_ref().unwrap());
                    self.builder.add_row("pop rax", true);
                    self.builder.add_row("cmp rax, 0", true);
                    self.builder.add_row(&format!("je .L.else.{}", seq), true);
                    self.gen_stmt(then.as_ref().unwrap());
                    self.builder.add_row(&format!("jmp .L.end.{}", seq), true);
                    self.builder.add_row(&format!(".L.else.{}:", seq), false);
                    self.gen_stmt(els.as_ref().unwrap());
                    self.builder.add_row(&format!(".L.end.{}:", seq), false);
                } else {
                    // else節なし
                    self.gen_expr(cond.as_ref().unwrap());
                    self.builder.add_row("pop rax", true);
                    self.builder.add_row("cmp rax, 0", true);
                    self.builder.add_row(&format!("je .L.end.{}", seq), true);
                    self.gen_stmt(then.as_ref().unwrap());
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
                self.gen_expr(cond.as_ref().unwrap());
                self.builder.add_row("pop rax", true);
                self.builder.add_row("cmp rax, 0", true);
                self.builder.add_row(&format!("je .L.break.{}", seq), true);
                self.gen_stmt(then.as_ref().unwrap());
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
                        self.gen_expr(init);
                        self.builder.add_row("pop rax", true); // 初期化式の結果を捨てる
                    } else {
                        self.gen_stmt(init);
                    }
                }
                self.builder.add_row(&format!(".L.begin.{}:", seq), false);
                if let Some(cond) = cond.as_ref() {
                    self.gen_expr(cond);
                    self.builder.add_row("pop rax", true);
                    self.builder.add_row("cmp rax, 0", true);
                    self.builder.add_row(&format!("je .L.break.{}", seq), true);
                }
                self.gen_stmt(then.as_ref().unwrap());
                self.builder
                    .add_row(&format!(".L.continue.{}:", seq), false);
                if let Some(inc) = inc.as_ref() {
                    if inc.is_expr() {
                        self.gen_expr(inc);
                        self.builder.add_row("pop rax", true); // 増分式の結果を捨てる
                    } else {
                        self.gen_stmt(inc);
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
                self.gen_stmt(then.as_ref().unwrap());
                self.builder
                    .add_row(&format!(".L.continue.{}:", seq), false);
                self.gen_expr(cond.as_ref().unwrap());
                self.builder.add_row("pop rax", true);
                self.builder.add_row("cmp rax, 0", true);
                self.builder.add_row(&format!("jne .L.begin.{}", seq), true);
                self.builder.add_row(&format!(".L.break.{}:", seq), false);

                self.break_seq = current_break_seq;
                self.continue_seq = current_continue_seq;
            }
            NodeKind::Block { body } => {
                for node in body.iter() {
                    if node.is_expr() {
                        self.gen_expr(node);
                        self.builder.add_row("pop rax", true); // ブロック内の各文の結果を捨てる
                    } else {
                        self.gen_stmt(node);
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
                self.builder
                    .add_row(&format!("jmp .L.label.{}.{}", self.func_name, name), true);
            }
            NodeKind::Label { name } => {
                self.builder
                    .add_row(&format!(".L.label.{}.{}:", self.func_name, name), false);
                if node.lhs.as_ref().unwrap().is_expr() {
                    self.gen_expr(node.lhs.as_ref().unwrap());
                    self.builder.add_row("pop rax", true); // ラベル付き文の結果を捨てる
                } else {
                    self.gen_stmt(node.lhs.as_ref().unwrap());
                }
            }
            NodeKind::Return => {
                if let Some(node) = node.lhs.as_ref() {
                    self.gen_expr(node);
                    self.builder.add_row("pop rax", true);
                }
                self.builder
                    .add_row(&format!("jmp .L.return.{}", self.func_name), true);
            }
            NodeKind::Nop => {}
            _ => {
                self.gen_expr(node);
                self.builder.add_row("pop rax", true); // 式の結果を捨てる
            }
        }
    }

    // 式のコード生成
    fn gen_expr(&mut self, node: &Node) {
        match &node.kind {
            NodeKind::Number { val } => {
                self.builder.add_row(&format!("push {}", val), true);
            }
            NodeKind::String { index, .. } => {
                self.builder
                    .add_row(&format!("lea rax, .L.str.{}[rip]", index), true); // RIP相対アドレッシング
                self.builder.add_row("push rax", true); // 文字列リテラルのアドレスをスタックに積む
            }
            NodeKind::LVar { .. } | NodeKind::GVar { .. } => {
                self.gen_addr(node);
                if !node.ty.as_ref().unwrap().deref().is_array() {
                    self.load(node); // 配列型以外は値を読み出す
                }
            }
            NodeKind::Assign => {
                self.gen_addr(node.lhs.as_ref().unwrap());
                self.gen_expr(node.rhs.as_ref().unwrap());
                self.store(node.lhs.as_ref().unwrap());
            }
            NodeKind::PreInc => {
                self.gen_addr(node.lhs.as_ref().unwrap());
                self.builder.add_row("push [rsp]", true);
                self.load(node.lhs.as_ref().unwrap());
                self.inc();
                self.store(node.lhs.as_ref().unwrap());
            }
            NodeKind::PreDec => {
                self.gen_addr(node.lhs.as_ref().unwrap());
                self.builder.add_row("push [rsp]", true);
                self.load(node.lhs.as_ref().unwrap());
                self.dec();
                self.store(node.lhs.as_ref().unwrap());
            }
            NodeKind::PostInc => {
                self.gen_addr(node.lhs.as_ref().unwrap());
                self.builder.add_row("push [rsp]", true);
                self.load(node.lhs.as_ref().unwrap());
                self.inc();
                self.store(node.lhs.as_ref().unwrap());
                self.dec();
            }
            NodeKind::PostDec => {
                self.gen_addr(node.lhs.as_ref().unwrap());
                self.builder.add_row("push [rsp]", true);
                self.load(node.lhs.as_ref().unwrap());
                self.dec();
                self.store(node.lhs.as_ref().unwrap());
                self.inc();
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
                self.gen_addr(node.lhs.as_ref().unwrap());
                self.builder.add_row("push [rsp]", true);
                self.load(node.lhs.as_ref().unwrap());
                self.gen_expr(node.rhs.as_ref().unwrap());
                self.gen_binary(node);
                self.store(node.lhs.as_ref().unwrap());
            }
            NodeKind::Ternary { cond, then, els } => {
                let seq = self.label_seq;
                self.label_seq += 1;
                self.gen_expr(cond.as_ref().unwrap());
                self.builder.add_row("pop rax", true);
                self.builder.add_row("cmp rax, 0", true);
                self.builder.add_row(&format!("je .L.else.{}", seq), true);
                self.gen_expr(then.as_ref().unwrap());
                self.builder.add_row(&format!("jmp .L.end.{}", seq), true);
                self.builder.add_row(&format!(".L.else.{}:", seq), false);
                self.gen_expr(els.as_ref().unwrap());
                self.builder.add_row(&format!(".L.end.{}:", seq), false);
            }
            NodeKind::LogicalNot => {
                self.gen_expr(node.lhs.as_ref().unwrap());
                self.builder.add_row("pop rax", true);
                self.builder.add_row("cmp rax, 0", true);
                self.builder.add_row("sete al", true);
                self.builder.add_row("movzb rax, al", true);
                self.builder.add_row("push rax", true);
            }
            NodeKind::BitNot => {
                self.gen_expr(node.lhs.as_ref().unwrap());
                self.builder.add_row("pop rax", true);
                self.builder.add_row("not rax", true);
                self.builder.add_row("push rax", true);
            }
            NodeKind::Addr => {
                self.gen_addr(node.lhs.as_ref().unwrap());
            }
            NodeKind::Deref => {
                self.gen_expr(node.lhs.as_ref().unwrap());
                self.load(node.lhs.as_ref().unwrap());
            }
            NodeKind::LogicalAnd => {
                let seq = self.label_seq;
                self.label_seq += 1;
                self.gen_expr(node.lhs.as_ref().unwrap());
                self.builder.add_row("pop rax", true);
                self.builder.add_row("cmp rax, 0", true);
                self.builder.add_row(&format!("je .L.false.{}", seq), true);
                self.gen_expr(node.rhs.as_ref().unwrap());
                self.builder.add_row("pop rax", true);
                self.builder.add_row("cmp rax, 0", true);
                self.builder.add_row(&format!("je .L.false.{}", seq), true);
                self.builder.add_row("push 1", true);
                self.builder.add_row(&format!("jmp .L.end.{}", seq), true);
                self.builder.add_row(&format!(".L.false.{}:", seq), false);
                self.builder.add_row("push 0", true);
                self.builder.add_row(&format!(".L.end.{}:", seq), false);
            }
            NodeKind::LogicalOr => {
                let seq = self.label_seq;
                self.label_seq += 1;
                self.gen_expr(node.lhs.as_ref().unwrap());
                self.builder.add_row("pop rax", true);
                self.builder.add_row("cmp rax, 0", true);
                self.builder.add_row(&format!("jne .L.true.{}", seq), true);
                self.gen_expr(node.rhs.as_ref().unwrap());
                self.builder.add_row("pop rax", true);
                self.builder.add_row("cmp rax, 0", true);
                self.builder.add_row(&format!("jne .L.true.{}", seq), true);
                self.builder.add_row("push 0", true);
                self.builder.add_row(&format!("jmp .L.end.{}", seq), true);
                self.builder.add_row(&format!(".L.true.{}:", seq), false);
                self.builder.add_row("push 1", true);
                self.builder.add_row(&format!(".L.end.{}:", seq), false);
            }
            NodeKind::Call { name, args } => {
                let arg_count = args.len();

                if arg_count > 6 {
                    panic!("6個を超える引数の関数呼び出しには対応していません");
                }

                // 引数をスタックに積む
                for arg in args.iter() {
                    self.gen_expr(arg);
                }

                // 引数をレジスタに移動
                for reg in ARG_QWORD_REGS.iter().take(arg_count) {
                    self.builder.add_row(&format!("pop {}", reg), true);
                }

                // 関数呼び出し（アラインメントは揃っているはず）
                self.builder.add_row("mov al, 0", true); // 浮動小数点は使わないので0に設定
                self.builder.add_row(&format!("call {}", name), true); // 関数呼び出し
                self.builder.add_row("push rax", true); // 戻り値をスタックに積む
            }
            _ => {
                // 二項演算子
                self.gen_expr(node.lhs.as_ref().unwrap());
                self.gen_expr(node.rhs.as_ref().unwrap());
                self.gen_binary(node);
            }
        }
    }

    fn gen_binary(&mut self, node: &Node) {
        self.builder.add_row("pop rdi", true); // 右オペランド
        self.builder.add_row("pop rax", true); // 左オペランド

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
        self.builder.add_row("push rax", true); // 演算結果をスタックに積む
    }
}
