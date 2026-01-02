use clap::Parser;
use clap_derive::Parser;

mod asm_builder;
mod ast;
mod errors;
mod function;
mod lexer;
mod node;
mod symbol;
mod token;
mod types;
mod x86;

use crate::ast::Ast;
use crate::lexer::Lexer;
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
    let mut args = Args::parse();

    if !args.file.is_empty() {
        match std::fs::read_to_string(&args.file) {
            Ok(content) => {
                if !args.input.is_empty() {
                    eprintln!("Warning: Both input string and file provided. Using file content.");
                }
                args.input = content;
            }
            Err(e) => {
                eprintln!("File Read Error: {}", e);
                return;
            }
        }
    }

    let lexer = Lexer::new(&args.input);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("{}", e.format_error(&args.input));
            return;
        }
    };
    let mut ast = Ast::new(&tokens);
    if let Err(e) = ast.translation_unit() {
        eprintln!("{}", e.format_error(&args.input));
        return;
    }
    let mut generator = Generator::new(&ast);
    if let Err(e) = generator.gen_asm() {
        eprintln!("{}", e.format_error(&args.input));
        return;
    }

    if args.debug {
        println!("=== Global Variables ===");
        println!("{:#?}", ast.global_symbol_table.iter().collect::<Vec<_>>());
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
