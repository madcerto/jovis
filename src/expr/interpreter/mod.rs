mod env;
mod dtype;

use env::Environment;
use dtype::{DType, Object};

use std::collections::HashMap;
use crate::token::literal::Literal;
use super::Expr;
use crate::token::TokenType;

pub trait Interpreter {
    fn interpret(self, env: &mut Environment) -> Vec<u8>;
}

impl Interpreter for Expr {
    fn interpret(self, env: &mut Environment) -> Vec<u8> {
        match self {
            Expr::Unary(_, _) => todo!(),
            Expr::Binary(left, op, right) => match op.ttype {
                TokenType::Equal => {
                    let name  = match *left {
                        Expr::BinaryOpt(left, op, _) => match op.ttype {
                            TokenType::Colon => match *left {
                                Expr::Identifier(name) => name.lexeme,
                                _ => panic!("expected identifier")
                            },
                            _ => panic!("expected declaration")
                        },
                        _ => panic!("expected declaration")
                    };
                    let val = right.interpret(env);
                    // env.define(name, val.clone(), Object { dtype: DType { size: 0, msgs: HashMap::new() }, address: 0 });
                    env.define(name, DType { size: 0, msgs: HashMap::new() }, Object { dtype: DType { size: 0, msgs: HashMap::new() }, address: 0 });
                    val
                },
                _ => panic!("unexpected binary operator")
            },
            Expr::MsgEmission(_, _) => todo!(),
            Expr::BinaryOpt(_, _, _) => todo!(),
            Expr::Object(exprs) => {
                let mut vals = vec![];
                for expr in exprs {
                    vals.append(&mut expr.interpret(env));
                }
                vals
            },
            Expr::Call(_, _) => todo!(),
            Expr::CodeBlock(_, exprs) => {
                let mut last = vec![];
                for expr in exprs {
                    last = expr.interpret(env);
                }
                last
            },
            // Expr::Identifier(name) => env.get(name.lexeme),
            Expr::Identifier(name) => todo!(),
            Expr::Literal(inner) => todo!(),
        }
    }
}

pub fn new_env() -> Environment {
    Environment::new()
}