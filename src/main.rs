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

    let tokenizer = Tokenizer::new(&args[1]);
    let ast = Ast::new(tokenizer.tokens.clone());

    let debug = false;
    if debug {
        println!("=== Tokens ===");
        println!("{:#?}", tokenizer.tokens);
        println!("=== Functions ===");
        println!("{:#?}", ast.funcs);
        println!("=== Local Variables ===");
        println!("{:#?}", ast.locals);
    }

    let mut generator = Generator::new();

    generator.gen_asm(&ast);

    // スタックを実行不可に設定
    println!(".section .note.GNU-stack,\"\",@progbits");
}
