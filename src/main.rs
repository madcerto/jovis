use std::io::prelude::*;
use std::io::Result;
use expr::compiler::Environment;
use expr::compiler::TypeCheck;
use expr::compiler::asm_type::AsmLanguage;
use expr::compiler::code_generator::CodeGenerator;
use expr::compiler::asm_type::AsmTarget;
use expr::parser::Parser;
use token::scanner::Scanner;

mod token;
mod expr;
mod pprint;
mod linker;

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
            let mut generator = CodeGenerator::new(AsmLanguage::NASM);
            
            generator.generate_ir(ast, "".into(), AsmTarget::X86Unix, &mut env);
        },
        Err((line, message)) => {println!("{} at {}", message, line)}
    }

    Ok(())
}