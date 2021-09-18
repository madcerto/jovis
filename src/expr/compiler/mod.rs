mod env;
mod dtype;
mod interpreter;
mod type_checker;
mod decl;
pub mod asm_type;
pub mod core_lib;
pub mod code_generator;

pub use env::Environment;
use dtype::DType;
use super::Expr;

pub use type_checker::TypeCheck;

fn fill_slice_with_vec<T: Clone>(slice: &mut [T], vec: Vec<T>) {
    for i in 0..slice.len() {
        slice[i] = vec[i].clone();
    }
}