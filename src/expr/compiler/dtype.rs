use std::{fmt::Debug, rc::Rc};

use crate::{expr::Expr, token::literal::Literal};
use super::{Environment, core_lib::*};

#[derive(Clone, Debug, PartialEq)]
pub struct DType {
    pub size: usize,
    pub msgs: Vec<Msg>
}

impl DType {
    pub fn from_literal(lit: Literal) -> Self {
        match lit {
            Literal::String(_) => STRING,
            Literal::Char(_) => CHAR,
            Literal::Integer(_) => I32,
            Literal::Float(_) => F32,
            Literal::Byte(_) => B8,
        }
    }

    pub fn get_msg(&self, msg_name: &String) -> Option<Msg> {
        for msg in &self.msgs {
            if &msg.name == msg_name { return Some(msg.clone()) }
        }
        None
    }
}

#[derive(Clone)]
pub struct Msg {
    pub name: String,
    constructor: Rc<dyn Fn(Option<Box<Expr>>, &Environment, Option<Box<Expr>>) -> Expr>,
    pub ret_type: DType,
    pub arg_type: Option<DType>
}
impl Msg {
    pub fn new(name: String,
        constructor: Rc<dyn Fn(Option<Box<Expr>>, &Environment, Option<Box<Expr>>) -> Expr>,
        ret_type: DType,
        arg_type: Option<DType>
    ) -> Self {
        Self { name, constructor, ret_type, arg_type }
    }

    pub fn construct(&self, self_expr: Option<Box<Expr>>, env: &Environment, arg: Option<Box<Expr>>) -> Expr {
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