use super::expr::{Expr, compiler::{TypeCheck}};

pub fn generate_code(mut ast: Expr, out_file: String) {
    ast.check_new_env().unwrap();
    // TODO: Generate IR
    // TODO: Write IR to file
    // TODO: call linker on IR file
}