use std::env;

pub mod ast;
pub mod parser;
pub mod codegen;

use crate::parser::Tokenizer;
use crate::codegen::gen_asm_from_expr;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("引数の数が正しくありません");
        return;
    }

    let mut tokenizer = Tokenizer::new();
    tokenizer.tokenize(&args[1]);
    // println!("{:?}", tokenizer);
    let node = tokenizer.expr().unwrap();

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    gen_asm_from_expr(&node);

    println!("  pop rax");
    println!("  ret");
}
