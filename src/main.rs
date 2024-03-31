use expr::compiler::code_generator::{
    asm_type::{AsmLanguage, AsmTarget},
    CodeGenerator,
};
use expr::compiler::Environment;
use expr::compiler::TypeCheck;
use expr::parser::Parser;
use scanner::Scanner;
use std::io::prelude::*;
use std::io::Result;

mod error;
mod expr;
mod linker;
mod pprint;
mod scanner;
mod token;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 2 {
        parse_file(args[1].clone())?;
    } else {
        println!("Usage: jovis <file name>")
    }

    Ok(())
}

fn parse_file(path: String) -> Result<()> {
    let mut file = std::fs::File::open(path)?;
    let mut contents = String::new();

    file.read_to_string(&mut contents).unwrap();

    let mut scanner = Scanner::new(contents);
    match scanner.scan_tokens() {
        Ok(tokens) => {
            let mut parser = Parser::new(tokens);
            let mut ast = parser.parse();
            let mut env = Environment::new();
            ast.check(&mut env).unwrap();
            // generate code from ast; go back down the mountain
            let generator = CodeGenerator::new(AsmLanguage::NASM);

            generator.generate_ir(ast, "test.jir".into(), AsmTarget::X86Unix, &mut env);
        }
        Err(e) => println!("{e}"),
    }

    Ok(())
}
