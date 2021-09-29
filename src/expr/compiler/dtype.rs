use std::{fmt::Debug, rc::Rc};

use crate::{expr::Expr, token::literal::Literal};
use super::{Environment, core_lib::*};

#[derive(Clone, Debug)]
pub struct DType {
    pub size: u32,
    pub msgs: Vec<Msg>,
    pub size_unknown: bool,
    pub msgs_unknown: bool
}

impl DType {
    pub fn new(size: u32, msgs: Vec<Msg>, size_unknown: bool, msgs_unknown: bool) -> Self {
        Self {
            size,
            msgs,
            size_unknown,
            msgs_unknown
        }
    }

    pub fn from_literal(lit: Literal) -> Self {
        match lit {
            Literal::String(_) => STRING,
            Literal::Char(_) => CHAR,
            Literal::Integer(_) => I32,
            Literal::Float(_) => F32,
            Literal::Byte(_) => B8,
        }
    }
    pub fn from_bytes(bytes: [u8; 6]) -> Self {
        let mut size_slice = [0; 4];
        for i in 0..4 {
            size_slice[i] = bytes[i];
        }
        let size = u32::from_ne_bytes(size_slice);
        let size_unknown = bytes[4] == 1;
        let msgs_unknown = bytes[5] == 1;
        Self { size, msgs: vec![], size_unknown, msgs_unknown }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.size.to_ne_bytes().to_vec();
        bytes.push(self.size_unknown as u8);
        bytes.push(self.msgs_unknown as u8);
        bytes
    }
    pub fn to_expr(&self) -> Expr {
        let mut byte_lits = vec![];
        for byte in self.to_bytes().clone() { byte_lits.push(Expr::Literal(Literal::Byte(byte))) }
        Expr::Object(byte_lits)
    }

    pub fn get_msg(&self, msg_name: &String) -> Option<Msg> {
        for msg in &self.msgs {
            if &msg.name == msg_name { return Some(msg.clone()) }
        }
        None
    }

    pub fn union(&self, other: &Self) -> Option<Self> {
        if self != other { return None }
        let size = self.size.max(other.size);
        let msgs = if self.msgs.len() > other.msgs.len() { self.msgs.clone() }
            else { other.msgs.clone() };
        let size_unknown = self.size_unknown && other.size_unknown;
        let msgs_unknown = self.msgs_unknown && other.msgs_unknown;
        Some(Self::new(size, msgs, size_unknown, msgs_unknown))
    }

    pub fn compose(&mut self, other: DType) {
        self.size += other.size;
        self.msgs.extend(other.msgs.into_iter());
    }
}
impl PartialEq for DType {
    // used when a value of other is trying to be used as a value of self
    fn eq(&self, other: &Self) -> bool {
        if self.size_unknown {
            if other.size <= self.size {
                return false
            }
        } else if other.size_unknown {
            if self.size <= other.size {
                return false
            }
        } else {
            if self.size != other.size {
                return  false
            }
        }

        if self.msgs_unknown {
            for msg in &self.msgs {
                if other.get_msg(&msg.name) == None {
                    return false
                }
            }
        } else if other.msgs_unknown {
            for msg in &other.msgs {
                if self.get_msg(&msg.name) == None {
                    return false
                }
            }
        } else {
            return self.msgs == other.msgs
        }
        true
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
        f.debug_struct(format!("Msg {}", self.name).as_str())
         .field("type", &self.ret_type)
         .finish()
    }
}