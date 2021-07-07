pub mod compiler;
pub mod parser;

use super::token::{Token, literal::Literal};
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub enum Expr {
    Unary(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    MsgEmission(Option<Box<Expr>>, Token),
    BinaryOpt(Box<Expr>, Token, Option<Box<Expr>>),
    Object(Vec<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    CodeBlock(Vec<Expr>, Vec<Expr>),
    Literal(Literal),
}