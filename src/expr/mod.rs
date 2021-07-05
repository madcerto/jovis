pub mod interpreter;
pub mod parser;

use super::token::{Token, literal::Literal};
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub enum Expr {
    Unary(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    MsgEmission(Box<Expr>, Box<Expr>),
    BinaryOpt(Box<Expr>, Token, Option<Box<Expr>>),
    Object(Vec<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    CodeBlock(Vec<Expr>, Vec<Expr>),
    Identifier(Token),
    Literal(Literal),
}