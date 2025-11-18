use clap::Parser;
use clap_derive::Parser;

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

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    debug: bool,

    #[arg(short, long, default_value_t = true)]
    optimize: bool,

    #[arg(short, long, default_value = "")]
    input: String,

    #[arg(short, long, default_value = "")]
    file: String,
}

fn main() {
    let args = Args::parse();

    let tokenizer = Tokenizer::default();
    let tokens = match tokenizer.tokenize(&args.input) {
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

    if args.debug {
        // println!("=== Tokens ===");
        // println!("{:#?}", tokens);
        println!("=== Global Variables ===");
        println!("{:#?}", ast.globals);
        println!("=== Functions ===");
        println!("{:#?}", ast.funcs);
        println!("=== String Literals ===");
        println!("{:#?}", ast.string_literals);
    } else {
        if args.optimize {
            generator.builder.optimize();
        }
        let code = generator.builder.build();
        println!("{}", code);
    }
}
