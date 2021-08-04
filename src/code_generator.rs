use std::ffi::CString;
use crate::linker::j_link;
use super::expr::{Expr, compiler::{TypeCheck}};

pub fn _generate_code(mut ast: Expr, out_file: String) {
    ast.check_new_env().unwrap();
    // TODO: Generate IR
    // TODO: Write IR to file
    // call linker on IR file
    unsafe {
        j_link(CString::new(out_file).unwrap().as_ptr());
    }
}