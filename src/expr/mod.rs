pub mod compiler;
pub mod parser;
pub mod checked_expr;

use super::token::{Token, literal::Literal};

#[derive(Clone, Debug)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    MsgEmission(Option<Box<Expr>>, Token, Option<Box<Expr>>),
    BinaryOpt(Box<Expr>, Token, Option<Box<Expr>>),
    Asm(Box<Expr>, Box<Expr>, Box<Expr>),
    Object(Vec<Expr>),
    Fn(Vec<Expr>, Box<Expr>),
    CodeBlock(Vec<Expr>),
    Type(Vec<Expr>),
    Literal(Literal),
}