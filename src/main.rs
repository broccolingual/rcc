use std::env;

pub mod ast;
pub mod parser;
pub mod x86;

use crate::ast::Ast;
use crate::parser::Tokenizer;
use crate::x86::Generator;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("引数の数が正しくありません");
        return;
    }

    let mut tokenizer = Tokenizer::new();
    let tokens = tokenizer.tokenize(&args[1]);
    // println!("{:#?}", tokens);

    let mut ast = Ast::new(tokens);
    ast.program();
    // println!("{:#?}", ast.locals);

    let mut generator = Generator::new();

    // おまじない
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    // 変数26個分の領域を確保
    println!("  push rbp");
    println!("  mov rbp, rsp");
    println!("  sub rsp, 208");

    for node in ast.code.iter() {
        generator.gen_asm_from_expr(node.as_ref().unwrap());
        println!("  pop rax");
    }

    println!("  mov rsp, rbp");
    println!("  pop rbp");
    println!("  ret");

    // スタックを実行不可に設定
    println!(".section .note.GNU-stack,\"\",@progbits");
}
