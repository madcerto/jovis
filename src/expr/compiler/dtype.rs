use crate::{expr::Expr, token::literal::Literal};

use super::Environment;

#[derive(Clone, Debug, PartialEq)]
pub struct DType {
    pub size: usize,
    pub msgs: Vec<Msg>
}

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
const I32: DType = DType {
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
                fn constructor(_: usize,_:Environment,_:Option<Expr>) -> Expr { Expr::Object(vec![]) }
                DType {
                    size: 96,
                    msgs: vec![Msg::new("addr".into(), constructor, B64)]
                }
            },
            Literal::Char(_) => CHAR,
            Literal::Integer(_) => I32,
            Literal::Float(_) => F32,
            Literal::Byte(_) => B8,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Msg {
    pub name: String,
    constructor: fn(self_address: usize, env: Environment, arg: Option<Expr>) -> Expr,
    pub ret_type: DType
}

impl Msg {
    pub fn new(name: String,
        constructor: fn(self_address: usize, env: Environment, arg: Option<Expr>) -> Expr,
        ret_type: DType
    ) -> Self {
        Self { name, constructor, ret_type }
    }

    pub fn construct(&self) -> Expr {
        Expr::Object(vec![])
    }
}