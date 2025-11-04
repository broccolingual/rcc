use std::env;

pub mod ast;
pub mod codegen;
pub mod parser;

use crate::codegen::gen_asm_from_expr;
use crate::parser::Tokenizer;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("引数の数が正しくありません");
        return;
    }

    let mut tokenizer = Tokenizer::new();
    tokenizer.tokenize(&args[1]);
    tokenizer.program();
    // println!("{:?}", tokenizer);

    // おまじない
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    // 変数26個分の領域を確保
    println!("  push rbp");
    println!("  mov rbp, rsp");
    println!("  sub rsp, 208");

    for node in tokenizer.code.iter() {
        gen_asm_from_expr(node.as_ref().unwrap());
        println!("  pop rax");
    }

    println!("  mov rsp, rbp");
    println!("  pop rbp");
    println!("  ret");
}
