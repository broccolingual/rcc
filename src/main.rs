use std::env;

pub mod ast;
pub mod parser;
pub mod x86;

use crate::ast::Ast;
use crate::parser::Tokenizer;
use crate::x86::gen_asm_from_expr;

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

    // おまじない
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    // 変数26個分の領域を確保
    println!("  push rbp");
    println!("  mov rbp, rsp");
    println!("  sub rsp, 208");

    for node in ast.code.iter() {
        gen_asm_from_expr(node.as_ref().unwrap());
        println!("  pop rax");
    }

    println!("  mov rsp, rbp");
    println!("  pop rbp");
    println!("  ret");
}
