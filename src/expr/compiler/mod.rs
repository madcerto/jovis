mod env;
mod dtype;
mod interpreter;
mod type_checker;
pub mod core_lib;

use env::Environment;
use dtype::DType;
use super::Expr;

pub use type_checker::TypeCheck;