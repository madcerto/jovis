use crate::{expr::Expr, token::literal::Literal};

use super::Environment;

#[derive(Clone, Debug)]
pub struct DType {
    pub size: usize,
    pub msgs: Vec<Msg>
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

#[derive(Clone, Debug)]
pub struct Msg {
    name: String,
    constructor: fn(self_address: usize, arg: Option<Expr>) -> Expr,
    ret_type: DType
}

impl Msg {
    pub fn new(name: String,
        constructor: fn(self_address: usize, arg: Option<Expr>) -> Expr,
        ret_type: DType
    ) -> Self {
        Self { name, constructor, ret_type }
    }
}