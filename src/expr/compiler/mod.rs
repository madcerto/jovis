mod env;
mod dtype;
mod interpreter;

use env::Environment;
use dtype::{DType, Object};
use super::Expr;

pub use interpreter::Interpret;