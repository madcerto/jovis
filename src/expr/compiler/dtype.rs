use std::collections::HashMap;
use crate::token::literal::Literal;

use super::Environment;

#[derive(Clone, Debug)]
pub struct DType {
    pub size: usize,
    pub msgs: HashMap<String, fn(obj: Object, env: Environment, arg: Option<Object>) -> Object>
}

impl DType {
    pub fn from_literal(lit: Literal) -> Self {
        match lit {
            Literal::String(_) => todo!(),
            Literal::Char(_) => todo!(),
            Literal::Integer(_) => todo!(),
            Literal::Float(_) => todo!(),
        }
    }
}

pub struct Object {
    pub dtype: DType,
    pub address: usize
}