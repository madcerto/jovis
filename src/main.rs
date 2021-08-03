use std::io::prelude::*;
use std::io::Result;
use expr::compiler::TypeCheck;
use expr::parser::Parser;
use token::scanner::Scanner;

mod token;
mod expr;
mod pprint;
mod code_generator;
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
            // for token in tokens.clone() {
            //     println!("{}", token.to_string());
            // }
            let mut parser = Parser::new(tokens);
            let mut ast = parser.parse();
            // ast.pprint();
            ast.check_new_env().unwrap();
        },
        Err((line, message)) => {println!("{} at {}", message, line)}
    }

    Ok(())
}