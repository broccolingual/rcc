use core::panic;

use crate::asm_builder::AsmBuilder;
use crate::ast::{Ast, Var};
use crate::node::{BinaryOp, Node, NodeKind, UnaryOp};
use crate::types::{Type, TypeKind};

const ARG_REGS: [Reg; 6] = [Reg::Rdi, Reg::Rsi, Reg::Rdx, Reg::Rcx, Reg::R8, Reg::R9];

#[allow(dead_code)]
#[derive(Hash, Eq, PartialEq, Clone)]
enum Reg {
    Rax,
    Rcx,
    Rdx,
    Rdi,
    Rsi,
    R8,
    R9,
    R10,
    R11,
}

impl Reg {
    fn by_size(&self, size: usize) -> &'static str {
        match size {
            1 => self.byte(),
            2 => self.word(),
            4 => self.dword(),
            8 => self.qword(),
            _ => panic!("Unsupported register size: {}", size),
        }
    }

    fn qword(&self) -> &'static str {
        match self {
            Reg::Rax => "rax",
            Reg::Rcx => "rcx",
            Reg::Rdx => "rdx",
            Reg::Rdi => "rdi",
            Reg::Rsi => "rsi",
            Reg::R8 => "r8",
            Reg::R9 => "r9",
            Reg::R10 => "r10",
            Reg::R11 => "r11",
        }
    }

    fn dword(&self) -> &'static str {
        match self {
            Reg::Rax => "eax",
            Reg::Rcx => "ecx",
            Reg::Rdx => "edx",
            Reg::Rdi => "edi",
            Reg::Rsi => "esi",
            Reg::R8 => "r8d",
            Reg::R9 => "r9d",
            Reg::R10 => "r10d",
            Reg::R11 => "r11d",
        }
    }

    fn word(&self) -> &'static str {
        match self {
            Reg::Rax => "ax",
            Reg::Rcx => "cx",
            Reg::Rdx => "dx",
            Reg::Rdi => "di",
            Reg::Rsi => "si",
            Reg::R8 => "r8w",
            Reg::R9 => "r9w",
            Reg::R10 => "r10w",
            Reg::R11 => "r11w",
        }
    }

    fn byte(&self) -> &'static str {
        match self {
            Reg::Rax => "al",
            Reg::Rcx => "cl",
            Reg::Rdx => "dl",
            Reg::Rdi => "dil",
            Reg::Rsi => "sil",
            Reg::R8 => "r8b",
            Reg::R9 => "r9b",
            Reg::R10 => "r10b",
            Reg::R11 => "r11b",
        }
    }
}

pub struct Generator {
    label_seq: usize,
    break_seq: usize,
    continue_seq: usize,
    current_func_name: String,
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
            current_func_name: String::new(),
            builder: AsmBuilder::new(),
        }
    }

    fn test_zero(&mut self) {
        self.builder.add_row("pop rax", true);
        self.builder.add_row("test rax, rax", true);
    }

    fn gen_compare(&mut self, inst: &str) {
        self.builder.add_row("cmp rax, rdi", true);
        self.builder.add_row(&format!("{} al", inst), true);
        self.builder.add_row("movzx eax, al", true);
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

    fn emit_data_value(&mut self, val: &i64, size: usize) {
        let directive = match size {
            1 => "byte",
            2 => "word",
            4 => "long",
            8 => "quad",
            _ => panic!("未対応のグローバル変数初期化サイズ: {}", size),
        };
        self.builder
            .add_row(&format!(".{} {}", directive, val), true);
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
            if !gvar.init.is_empty() {
                self.emit_global_init(gvar);
            } else {
                self.builder
                    .add_row(&format!(".zero {}", gvar.ty.size_of()), true);
            }
        }
    }

    fn emit_global_init(&mut self, gvar: &Var) {
        if let TypeKind::Array { ref base, size } = gvar.ty.kind {
            // TODO: 多次元配列の初期化、文字列リテラルによる初期化
            let init_len = gvar.init.len().min(size);
            for i in 0..init_len {
                let init = &gvar.init[i];
                match &init.kind {
                    NodeKind::Number { val } => {
                        self.emit_data_value(val, base.size_of());
                    }
                    NodeKind::UnaryOp {
                        op: UnaryOp::Addr,
                        expr,
                    } => match &expr.kind {
                        NodeKind::Var { name, is_local, .. } => {
                            if !is_local {
                                self.builder.add_row(&format!(".quad {}", name), true);
                            } else {
                                panic!(
                                    "グローバル変数の初期化式にローカル変数のアドレスは使用できません: {}",
                                    name
                                );
                            }
                        }
                        _ => {
                            panic!(
                                "未対応のグローバル変数初期化式のアドレス指定: {:?}",
                                expr.kind
                            );
                        }
                    },
                    NodeKind::String { index, .. } => {
                        self.builder
                            .add_row(&format!(".quad .L.str.{}", index), true);
                    }
                    _ => panic!("未対応のグローバル変数初期化式: {:?}", init.kind),
                }
            }
            if gvar.init.len() < size {
                let zero_fill_size = (size - gvar.init.len()) * base.size_of();
                self.builder
                    .add_row(&format!(".zero {}", zero_fill_size), true);
            }
        } else if let TypeKind::Struct { .. } = gvar.ty.kind {
            // TODO: 構造体の初期化式
            unimplemented!("構造体のグローバル変数初期化には未対応です");
        } else if gvar.init.len() == 1 {
            let init = &gvar.init[0];
            match &init.kind {
                NodeKind::Number { val } => {
                    self.emit_data_value(val, gvar.ty.size_of());
                }
                NodeKind::UnaryOp {
                    op: UnaryOp::Addr,
                    expr,
                } => match &expr.kind {
                    NodeKind::Var { name, is_local, .. } => {
                        if !is_local {
                            self.builder.add_row(&format!(".quad {}", name), true);
                        } else {
                            panic!(
                                "グローバル変数の初期化式にローカル変数のアドレスは使用できません: {}",
                                name
                            );
                        }
                    }
                    _ => {
                        panic!(
                            "未対応のグローバル変数初期化式のアドレス指定: {:?}",
                            expr.kind
                        );
                    }
                },
                NodeKind::String { index, .. } => {
                    self.builder
                        .add_row(&format!(".quad .L.str.{}", index), true);
                }
                _ => panic!("未対応のグローバル変数初期化式: {:?}", init.kind),
            }
        } else {
            panic!(
                "スカラー型グローバル変数の初期化式が複数あります: {}",
                gvar.name
            );
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
            self.current_func_name = func.name.clone();
            self.builder
                .add_row(&format!(".globl {}", self.current_func_name), true);
            self.builder.add_row(
                &format!(".type {}, @function", self.current_func_name),
                true,
            );
            self.builder
                .add_row(&format!("{}:", self.current_func_name), false);

            // 関数プロローグ
            self.builder.add_row("push rbp", true);
            self.builder.add_row("mov rbp, rsp", true);

            // 関数のローカル変数に対応するスタック領域を確保
            // ローカル変数の最大オフセットに基づいてスタック領域を計算
            let max_offset = func.locals.last().map_or(0, |arg| arg.offset);
            let stack_size = max_offset.div_ceil(16) * 16; // 16バイトアラインメント
            if stack_size > 0 {
                self.builder
                    .add_row(&format!("sub rsp, {}", stack_size), true);
            }

            // 引数をレジスタからスタックへ読み出し
            for (i, param) in func.locals.iter().take(func.params_len).enumerate() {
                if i >= ARG_REGS.len() {
                    panic!("6個を超える引数の関数には未対応です");
                }
                self.builder.add_row(
                    &format!(
                        "mov [rbp-{}], {}",
                        param.offset,
                        ARG_REGS[i].by_size(param.ty.align_of())
                    ),
                    true,
                );
            }

            // ローカル変数の初期化
            for lvar in func.locals.iter().skip(func.params_len) {
                if !lvar.init.is_empty() {
                    self.gen_local_init(lvar);
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
                .add_row(&format!(".L.return.{}:", self.current_func_name), false);
            self.builder.add_row("leave", true);
            self.builder.add_row("ret", true);
        }
        self.emit_epilogue();
    }

    fn gen_local_init(&mut self, lvar: &Var) {
        if let TypeKind::Array { ref base, size } = lvar.ty.kind {
            // 配列の初期化式
            // TODO: 多次元配列の初期化、文字列リテラルによる初期化
            let init_len = lvar.init.len().min(size);
            for i in 0..init_len {
                let elem_offset = lvar.offset - i * base.size_of();
                self.builder
                    .add_row(&format!("lea rax, [rbp-{}]", elem_offset), true);
                self.builder.add_row("push rax", true); // 配列要素のアドレスをスタックに積む
                self.gen_expr(&lvar.init[i]); // 初期化式のコードを生成し、スタックに値を積む
                self.store(base); // スタックトップの値を配列要素に格納
            }
            // 初期化式の数が配列サイズに満たない場合、残りを0で埋める
            if lvar.init.len() < size {
                let zero_fill_offset = lvar.offset - init_len * base.size_of();
                let zero_fill_size = (size - init_len) * base.size_of();
                self.builder
                    .add_row(&format!("lea rdi, [rbp-{}]", zero_fill_offset), true); // 初期化開始アドレス
                self.builder
                    .add_row(&format!("mov rcx, {}", zero_fill_size), true); // 初期化するバイト数
                self.builder.add_row("xor rax, rax", true); // raxを0クリア
                self.builder.add_row("rep stosb", true); // 0で初期化
            }
        } else if let TypeKind::Struct { .. } = lvar.ty.kind {
            // TODO: 構造体の初期化式
            unimplemented!("構造体のローカル変数初期化には未対応です");
        } else if lvar.init.len() == 1 {
            self.gen_addr(&Box::new(Node {
                kind: NodeKind::Var {
                    name: lvar.name.clone(),
                    offset: lvar.offset,
                    is_local: true,
                },
                ..Default::default()
            })); // 変数のアドレスをスタックに積む
            self.gen_expr(&lvar.init[0]); // 初期化式のコードを生成し、スタックに値を積む
            self.store(&lvar.ty); // スタックトップの値を変数に格納
        } else {
            panic!("スカラー変数の初期化式が複数あります: {}", lvar.name);
        }
    }

    // 変数やデリファレンスのアドレスをスタックに積む
    fn gen_addr(&mut self, node: &Node) {
        match &node.kind {
            NodeKind::UnaryOp {
                op: UnaryOp::Deref,
                expr,
            } => {
                self.gen_expr(expr); // ポインタの値を取得
            }
            NodeKind::Var {
                name,
                offset,
                is_local,
                ..
            } => {
                if *is_local {
                    self.builder
                        .add_row(&format!("lea rax, [rbp-{}]", offset), true); // ローカル変数のアドレスを計算して取得
                } else {
                    self.builder
                        .add_row(&format!("lea rax, {}[rip]", name), true); // グローバル変数のアドレスを計算して取得
                }
                self.builder.add_row("push rax", true); // 変数のアドレスをスタックに積む
            }
            _ => panic!("代入の左辺値が変数ではありません: {:?}", node.kind),
        }
    }

    // スタックトップのアドレスから値を読み出してスタックに積む
    fn load(&mut self, ty: &Type) {
        self.builder.add_row("pop rax", true); // ロード先のアドレス
        let inst = match ty.size_of() {
            1 => "movsx rax, BYTE PTR [rax]",
            2 => "movsx rax, WORD PTR [rax]",
            4 => "movsxd rax, DWORD PTR [rax]",
            8 => "mov rax, QWORD PTR [rax]",
            _ => panic!("未対応のロードサイズ: {}", ty.size_of()),
        };
        self.builder.add_row(inst, true);
        self.builder.add_row("push rax", true); // 読み出した値をスタックに積む
    }

    // スタックトップの値をアドレスに格納する
    fn store(&mut self, ty: &Type) {
        self.builder.add_row("pop rdi", true); // ストアする値
        self.builder.add_row("pop rax", true); // ストア先のアドレス
        let instruction = match ty.size_of() {
            1 => "mov BYTE PTR [rax], dil",
            2 => "mov WORD PTR [rax], di",
            4 => "mov DWORD PTR [rax], edi",
            8 => "mov QWORD PTR [rax], rdi",
            _ => panic!("未対応のストアサイズ: {}", ty.size_of()),
        };
        self.builder.add_row(instruction, true);
        self.builder.add_row("push rdi", true); // ストアした値をスタックに戻す
    }

    // int を 1 加算
    fn inc(&mut self) {
        self.builder.add_row("inc QWORD PTR [rsp]", true);
    }

    // int を 1 減算
    fn dec(&mut self) {
        self.builder.add_row("dec QWORD PTR [rsp]", true);
    }

    // 文のコード生成
    fn gen_stmt(&mut self, node: &Node) {
        match &node.kind {
            NodeKind::If { cond, then, els } => {
                let seq = self.label_seq;
                self.label_seq += 1;

                self.gen_expr(cond);
                self.test_zero();
                if let Some(els_node) = els {
                    // else節あり
                    self.builder.add_row(&format!("je .L.else.{}", seq), true);
                    self.gen_stmt(then);
                    self.builder.add_row(&format!("jmp .L.end.{}", seq), true);
                    self.builder.add_row(&format!(".L.else.{}:", seq), false);
                    self.gen_stmt(els_node);
                    self.builder.add_row(&format!(".L.end.{}:", seq), false);
                } else {
                    // else節なし
                    self.builder.add_row(&format!("je .L.end.{}", seq), true);
                    self.gen_stmt(then);
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
                self.gen_expr(cond);
                self.test_zero();
                self.builder.add_row(&format!("je .L.break.{}", seq), true);
                self.gen_stmt(then);
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
                    self.test_zero();
                    self.builder.add_row(&format!("je .L.break.{}", seq), true);
                }
                self.gen_stmt(then);
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
                self.gen_stmt(then);
                self.builder
                    .add_row(&format!(".L.continue.{}:", seq), false);
                self.gen_expr(cond);
                self.test_zero();
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
                    self.gen_expr(expr);
                    self.builder.add_row("pop rax", true); // ラベル付き文の結果を捨てる
                } else {
                    self.gen_stmt(expr);
                }
            }
            NodeKind::Return { expr } => {
                if let Some(expr) = expr {
                    self.gen_expr(expr);
                    self.builder.add_row("pop rax", true);
                }
                self.builder
                    .add_row(&format!("jmp .L.return.{}", self.current_func_name), true);
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
            NodeKind::Var { .. } => {
                self.gen_addr(node);
                if !node.ty.is_array() {
                    self.load(&node.ty);
                }
            }
            NodeKind::BinaryOp { op, lhs, rhs } => {
                // 二項演算子
                self.gen_expr(lhs);
                self.gen_expr(rhs);
                self.gen_binary(op);
            }
            NodeKind::UnaryOp { op, expr } => {
                // 単項演算子
                self.gen_unary(op, expr, &node.ty);
            }
            NodeKind::Assign { op, lhs, rhs } => {
                self.gen_assign(op, lhs, rhs);
            }
            NodeKind::LogicalAnd { lhs, rhs } => {
                let seq = self.label_seq;
                self.label_seq += 1;
                self.gen_expr(lhs);
                self.test_zero();
                self.builder.add_row(&format!("je .L.false.{}", seq), true);
                self.gen_expr(rhs);
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
                self.gen_expr(lhs);
                self.test_zero();
                self.builder.add_row(&format!("jne .L.true.{}", seq), true);
                self.gen_expr(rhs);
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
                self.gen_expr(cond);
                self.test_zero();
                self.builder.add_row(&format!("je .L.else.{}", seq), true);
                self.gen_expr(then);
                self.builder.add_row(&format!("jmp .L.end.{}", seq), true);
                self.builder.add_row(&format!(".L.else.{}:", seq), false);
                self.gen_expr(els);
                self.builder.add_row(&format!(".L.end.{}:", seq), false);
            }
            NodeKind::Call { name, args } => {
                let arg_count = args.len();

                if arg_count > 6 {
                    panic!("6個を超える引数の関数呼び出しには対応していません");
                }

                // 引数をスタックに積む（逆順）
                for arg in args.iter().rev() {
                    self.gen_expr(arg);
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
    }

    fn gen_assign(&mut self, op: &BinaryOp, lhs: &Node, rhs: &Node) {
        match op {
            BinaryOp::Assign => {
                self.gen_addr(lhs);
                self.gen_expr(rhs);
                self.store(&lhs.ty);
            }
            _ => {
                self.gen_addr(lhs);
                self.builder.add_row("push [rsp]", true);
                self.load(&lhs.ty);
                self.gen_expr(rhs);
                self.gen_binary(op);
                self.store(&lhs.ty);
            }
        }
    }

    fn gen_unary(&mut self, op: &UnaryOp, expr: &Node, ty: &Type) {
        match op {
            UnaryOp::BitNot => {
                self.gen_expr(expr);
                self.builder.add_row("not QWORD PTR [rsp]", true);
            }
            UnaryOp::LogicalNot => {
                self.gen_expr(expr);
                self.test_zero();
                self.builder.add_row("sete al", true);
                self.builder.add_row("movzx rax, al", true);
                self.builder.add_row("push rax", true);
            }
            UnaryOp::Addr => {
                self.gen_addr(expr);
            }
            UnaryOp::Deref => {
                self.gen_expr(expr);
                // 型が配列でない場合にロード
                // 配列型の場合、アドレスをスタックに積むだけ
                if !ty.is_array() {
                    self.load(ty);
                }
            }
            UnaryOp::PreInc => {
                self.gen_addr(expr);
                self.builder.add_row("push [rsp]", true);
                self.load(&expr.ty);
                self.inc();
                self.store(&expr.ty);
            }
            UnaryOp::PreDec => {
                self.gen_addr(expr);
                self.builder.add_row("push [rsp]", true);
                self.load(&expr.ty);
                self.dec();
                self.store(&expr.ty);
            }
            UnaryOp::PostInc => {
                self.gen_addr(expr);
                self.builder.add_row("push [rsp]", true);
                self.load(&expr.ty);
                self.inc();
                self.store(&expr.ty);
                self.dec();
            }
            UnaryOp::PostDec => {
                self.gen_addr(expr);
                self.builder.add_row("push [rsp]", true);
                self.load(&expr.ty);
                self.dec();
                self.store(&expr.ty);
                self.inc();
            }
        }
    }

    fn gen_binary(&mut self, op: &BinaryOp) {
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
    }
}
