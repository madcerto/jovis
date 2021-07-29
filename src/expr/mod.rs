pub mod compiler;
pub mod parser;

use super::token::{Token, literal::Literal};
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    MsgEmission(Option<Box<Expr>>, Token, Option<Box<Expr>>),
    BinaryOpt(Box<Expr>, Token, Option<Box<Expr>>),
    Object(Vec<Expr>),
    Fn(Vec<Expr>, Box<Expr>),
    CodeBlock(Vec<Expr>),
    Type(Vec<Expr>),
    Literal(Literal),
}