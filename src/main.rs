use std::env;

pub mod asm_builder;
pub mod ast;
pub mod node;
pub mod parser;
pub mod token;
pub mod types;
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

    let tokenizer = Tokenizer::default();
    let tokens = match tokenizer.tokenize(&args[1]) {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Tokenizer Error: {}", e);
            return;
        }
    };
    let mut ast = Ast::new(&tokens);
    ast.translation_unit();

    let mut generator = Generator::default();
    generator.gen_asm(&ast);

    let debug = false;
    if debug {
        // println!("=== Tokens ===");
        // println!("{:#?}", tokens);
        println!("=== Global Variables ===");
        println!("{:#?}", ast.globals);
        println!("=== Functions ===");
        println!("{:#?}", ast.funcs);
    } else {
        generator.builder.optimize();
        let code = generator.builder.build();
        println!("{}", code);
    }
}
