use std::env;

pub mod ast;
pub mod node;
pub mod parser;
pub mod token;
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

    generator.gen_asm(&ast);

    // スタックを実行不可に設定
    println!(".section .note.GNU-stack,\"\",@progbits");
}
