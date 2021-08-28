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