mod expression;
mod register;
mod statement;

use register::ARG_REGS;

use crate::asm_builder::AsmBuilder;
use crate::ast::Ast;
use crate::errors::CompileError;
use crate::node::{Node, NodeKind, UnaryOp};
use crate::symbol::Symbol;
use crate::types::{Type, TypeKind};

pub(crate) struct Generator<'a> {
    ast: &'a Ast<'a>,
    current_func_name: &'a str,
    pub(crate) builder: AsmBuilder,
}

impl<'a> Generator<'a> {
    pub(crate) fn new(ast: &'a Ast<'a>) -> Self {
        Generator {
            ast,
            current_func_name: "",
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
        for (string, i) in ast.string_literals.iter() {
            self.builder.add_row(&format!(".L.str.{}:", i), false);
            self.builder
                .add_row(&format!(".string \"{}\"", string), true);
        }
    }

    fn emit_data_value(&mut self, val: &i64, size: usize) -> Result<(), CompileError> {
        let directive = match size {
            1 => "byte",
            2 => "word",
            4 => "long",
            8 => "quad",
            _ => {
                return Err(CompileError::InternalError {
                    msg: format!("不正なデータサイズ: {}", size),
                });
            }
        };
        self.builder
            .add_row(&format!(".{} {}", directive, val), true);
        Ok(())
    }

    fn emit_data(&mut self, ast: &Ast) -> Result<(), CompileError> {
        if ast.globals.is_empty() {
            return Ok(());
        }
        self.builder.add_row(".data", true);
        for var in ast.globals.iter() {
            let symbol = &ast.get_global_symbol_by_id(var.symbol_id).ok_or_else(|| {
                CompileError::InternalError {
                    msg: "グローバル変数のシンボルが見つかりません".to_string(),
                }
            })?;
            self.builder
                .add_row(&format!(".globl {}", symbol.name), true);
            self.builder
                .add_row(&format!(".align {}", symbol.ty.align_of()), true);
            self.builder
                .add_row(&format!(".type {}, @object", symbol.name), true);
            self.builder.add_row(
                &format!(".size {}, {}", symbol.name, symbol.ty.size_of()),
                true,
            );
            self.builder.add_row(&format!("{}:", symbol.name), false);
            if !var.init.is_empty() {
                self.emit_global_init(symbol, &var.init)?;
            } else {
                self.builder
                    .add_row(&format!(".zero {}", symbol.ty.size_of()), true);
            }
        }
        Ok(())
    }

    fn emit_global_init(&mut self, symbol: &Symbol, init: &[Node]) -> Result<(), CompileError> {
        if let TypeKind::Array { ref base, size } = symbol.ty.kind {
            // TODO: 多次元配列の初期化、文字列リテラルによる初期化
            let init_len = init.len().min(size);
            for i in 0..init_len {
                let init = &init[i];
                match &init.kind {
                    NodeKind::UnaryOp {
                        op: UnaryOp::Addr,
                        expr,
                    } => match &expr.kind {
                        NodeKind::Var { name, is_local, .. } => {
                            if !is_local {
                                self.builder.add_row(&format!(".quad {}", name), true);
                            } else {
                                return Err(CompileError::InvalidExpr {
                                    msg: format!(
                                        "グローバル変数の初期化式にローカル変数のアドレスは使用できません: {}",
                                        name
                                    ),
                                    span: expr.span,
                                });
                            }
                        }
                        _ => {
                            return Err(CompileError::InvalidExpr {
                                msg: format!(
                                    "未対応のグローバル変数初期化式のアドレス指定: {:?}",
                                    expr.kind
                                ),
                                span: expr.span,
                            });
                        }
                    },
                    NodeKind::String { index, .. } => {
                        self.builder
                            .add_row(&format!(".quad .L.str.{}", index), true);
                    }
                    _ => {
                        let val = init.eval_const_expr()?;
                        self.emit_data_value(&val, base.size_of())?;
                    }
                }
            }
            if init.len() < size {
                let zero_fill_size = (size - init.len()) * base.size_of();
                self.builder
                    .add_row(&format!(".zero {}", zero_fill_size), true);
            }
        } else if let TypeKind::Struct { .. } = symbol.ty.kind {
            // TODO: 構造体の初期化式
            unimplemented!("構造体のグローバル変数初期化には未対応です");
        } else if init.len() == 1 {
            let init = &init[0];
            match &init.kind {
                NodeKind::UnaryOp {
                    op: UnaryOp::Addr,
                    expr,
                } => match &expr.kind {
                    NodeKind::Var { name, is_local, .. } => {
                        if !is_local {
                            self.builder.add_row(&format!(".quad {}", name), true);
                        } else {
                            return Err(CompileError::InvalidExpr {
                                msg: format!(
                                    "グローバル変数の初期化式にローカル変数のアドレスは使用できません: {}",
                                    name
                                ),
                                span: expr.span,
                            });
                        }
                    }
                    _ => {
                        return Err(CompileError::InvalidExpr {
                            msg: format!(
                                "未対応のグローバル変数初期化式のアドレス指定: {:?}",
                                expr.kind
                            ),
                            span: expr.span,
                        });
                    }
                },
                NodeKind::String { index, .. } => {
                    self.builder
                        .add_row(&format!(".quad .L.str.{}", index), true);
                }
                _ => {
                    self.emit_data_value(&init.eval_const_expr()?, symbol.ty.size_of())?;
                }
            }
        } else {
            return Err(CompileError::InvalidExpr {
                msg: format!("スカラー変数の初期化式が複数あります: {}", symbol.name),
                span: init[0].span,
            });
        }
        Ok(())
    }

    fn emit_epilogue(&mut self) {
        self.builder
            .add_row(".section .note.GNU-stack,\"\",@progbits", true); // スタックを実行不可にする
    }

    // ASTからアセンブリコードを生成
    pub(crate) fn gen_asm(&mut self) -> Result<(), CompileError> {
        self.emit_prologue();
        self.emit_rodata(self.ast); // 文字列リテラルの定義
        self.emit_data(self.ast)?; // グローバル変数の定義

        // 関数の定義
        self.builder.add_row(".text", true);
        for func in self.ast.funcs.iter() {
            self.current_func_name = &func.name;
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
            let stack_size = func.get_stack_size();
            if stack_size > 0 {
                self.builder
                    .add_row(&format!("sub rsp, {}", stack_size), true);
            }

            // 引数をレジスタからスタックへ読み出し
            for (i, param) in func.get_params_iter().enumerate() {
                if i >= ARG_REGS.len() {
                    unimplemented!("6個を超える引数の関数には未対応です");
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
            for var in func.locals.iter() {
                if !var.init.is_empty() {
                    let symbol = &func.get_local_symbol_by_id(var.symbol_id).ok_or_else(|| {
                        CompileError::InternalError {
                            msg: "ローカル変数のシンボルが見つかりません".to_string(),
                        }
                    })?;
                    self.gen_local_init(symbol, &var.init)?;
                }
            }

            // 関数本体のコード生成
            for node in func.body.iter() {
                if node.is_expr() {
                    self.gen_expr(node)?;
                    self.builder.add_row("pop rax", true); // 式の結果を捨てる
                } else {
                    self.gen_stmt(node)?;
                }
            }

            // 関数エピローグ
            self.builder
                .add_row(&format!(".L.return.{}:", self.current_func_name), false);
            self.builder.add_row("leave", true);
            self.builder.add_row("ret", true);
        }
        self.emit_epilogue();
        Ok(())
    }

    fn gen_local_init(&mut self, symbol: &Symbol, init: &[Node]) -> Result<(), CompileError> {
        if let TypeKind::Array { ref base, size } = symbol.ty.kind {
            // 配列の初期化式
            // TODO: 多次元配列の初期化、文字列リテラルによる初期化
            let init_len = init.len().min(size);
            for (i, init) in init.iter().enumerate().take(init_len) {
                let elem_offset = symbol.offset - i * base.size_of();
                self.builder
                    .add_row(&format!("lea rax, [rbp-{}]", elem_offset), true);
                self.builder.add_row("push rax", true); // 配列要素のアドレスをスタックに積む
                self.gen_expr(init)?; // 初期化式のコードを生成し、スタックに値を積む
                self.store(base)?; // スタックトップの値を配列要素に格納
            }
            // 初期化式の数が配列サイズに満たない場合、残りを0で埋める
            if init.len() < size {
                let zero_fill_offset = symbol.offset - init_len * base.size_of();
                let zero_fill_size = (size - init_len) * base.size_of();
                self.builder
                    .add_row(&format!("lea rdi, [rbp-{}]", zero_fill_offset), true); // 初期化開始アドレス
                self.builder
                    .add_row(&format!("mov rcx, {}", zero_fill_size), true); // 初期化するバイト数
                self.builder.add_row("xor rax, rax", true); // raxを0クリア
                self.builder.add_row("rep stosb", true); // 0で初期化
            }
        } else if let TypeKind::Struct { .. } = symbol.ty.kind {
            // TODO: 構造体の初期化式
            unimplemented!("構造体のローカル変数初期化には未対応です");
        } else if init.len() == 1 {
            self.gen_addr(&Box::new(Node {
                kind: NodeKind::Var {
                    name: symbol.name.to_string(),
                    offset: symbol.offset,
                    is_local: true,
                },
                ..Default::default()
            }))?; // 変数のアドレスをスタックに積む
            self.gen_expr(&init[0])?; // 初期化式のコードを生成し、スタックに値を積む
            self.store(&symbol.ty)?; // スタックトップの値を変数に格納
        } else {
            return Err(CompileError::InvalidExpr {
                msg: format!("スカラー変数の初期化式が複数あります: {}", symbol.name),
                span: init[0].span,
            });
        }
        Ok(())
    }

    // 変数やデリファレンスのアドレスをスタックに積む
    fn gen_addr(&mut self, node: &Node) -> Result<(), CompileError> {
        match &node.kind {
            NodeKind::UnaryOp {
                op: UnaryOp::Deref,
                expr,
            } => {
                self.gen_expr(expr)?; // ポインタの値を取得
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
            NodeKind::Member { obj, offset, .. } => {
                self.gen_addr(obj)?; // オブジェクトのアドレスを取得
                self.builder.add_row("pop rax", true); // オブジェクトのアドレス
                self.builder.add_row(&format!("add rax, {}", offset), true); // メンバのオフセットを加算
                self.builder.add_row("push rax", true); // メンバのアドレスをスタックに積む
            }
            _ => {
                return Err(CompileError::InvalidExpr {
                    msg: format!("アドレスを取得できない式です: {:?}", node.kind),
                    span: node.span,
                });
            }
        }
        Ok(())
    }

    // スタックトップのアドレスから値を読み出してスタックに積む
    fn load(&mut self, ty: &Type) -> Result<(), CompileError> {
        self.builder.add_row("pop rax", true); // ロード先のアドレス
        let inst = match ty.size_of() {
            1 => "movsx rax, BYTE PTR [rax]",
            2 => "movsx rax, WORD PTR [rax]",
            4 => "movsxd rax, DWORD PTR [rax]",
            8 => "mov rax, QWORD PTR [rax]",
            _ => {
                return Err(CompileError::InternalError {
                    msg: format!("不正なロードサイズ: {}", ty.size_of()),
                });
            }
        };
        self.builder.add_row(inst, true);
        self.builder.add_row("push rax", true); // 読み出した値をスタックに積む
        Ok(())
    }

    // スタックトップの値をアドレスに格納する
    fn store(&mut self, ty: &Type) -> Result<(), CompileError> {
        self.builder.add_row("pop rdi", true); // ストアする値
        self.builder.add_row("pop rax", true); // ストア先のアドレス
        let instruction = match ty.size_of() {
            1 => "mov BYTE PTR [rax], dil",
            2 => "mov WORD PTR [rax], di",
            4 => "mov DWORD PTR [rax], edi",
            8 => "mov QWORD PTR [rax], rdi",
            _ => {
                return Err(CompileError::InternalError {
                    msg: format!("不正なストアサイズ: {}", ty.size_of()),
                });
            }
        };
        self.builder.add_row(instruction, true);
        self.builder.add_row("push rdi", true); // ストアした値をスタックに戻す
        Ok(())
    }

    // int を 1 加算
    fn inc(&mut self) {
        self.builder.add_row("inc QWORD PTR [rsp]", true);
    }

    // int を 1 減算
    fn dec(&mut self) {
        self.builder.add_row("dec QWORD PTR [rsp]", true);
    }
}
