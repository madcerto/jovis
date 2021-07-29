use std::{fmt::Debug, rc::Rc};

use crate::{expr::Expr, token::literal::Literal};
use super::Environment;

#[derive(Clone, Debug, PartialEq)]
pub struct DType {
    pub size: usize,
    pub msgs: Vec<Msg>
}

pub const VOID: DType = DType {
    size: 0,
    msgs: vec![]
};

const B8: DType = DType {
    size: 8,
    msgs: vec![]
};
// const B16: DType = DType {
//     size: 8,
//     msgs: vec![]
// };
// const B32: DType = DType {
//     size: 8,
//     msgs: vec![]
// };
const B64: DType = DType {
    size: 8,
    msgs: vec![]
};

const CHAR: DType = DType {
    size: 8,
    msgs: vec![]
};
pub const I32: DType = DType {
    size: 32,
    msgs: vec![]
};
const F32: DType = DType {
    size: 32,
    msgs: vec![]
};

impl DType {
    pub fn from_literal(lit: Literal) -> Self {
        match lit {
            Literal::String(_) => {
                DType {
                    size: 96,
                    msgs: vec![]
                }
            },
            Literal::Char(_) => CHAR,
            Literal::Integer(_) => I32,
            Literal::Float(_) => F32,
            Literal::Byte(_) => B8,
        }
    }
}

#[derive(Clone)]
pub struct Msg {
    pub name: String,
    constructor: Rc<dyn Fn(Expr, &Environment, Option<Expr>) -> Expr>,
    pub ret_type: DType
}
impl Msg {
    pub fn new(name: String,
        constructor: Rc<dyn Fn(Expr, &Environment, Option<Expr>) -> Expr>,
        ret_type: DType
    ) -> Self {
        Self { name, constructor, ret_type }
    }

    pub fn construct(&self, self_expr: Expr, env: &Environment, arg: Option<Expr>) -> Expr {
        (self.constructor) (self_expr, env, arg)
    }
}

impl PartialEq for Msg {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.ret_type == other.ret_type
    }
}
impl Debug for Msg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Msg")
         .field("name", &self.name)
         .field("ret_type", &self.ret_type)
         .finish()
    }
}