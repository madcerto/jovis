use crate::{expr::compiler::{Interpret, dtype::Msg}, token::{Token, TokenType}};
use super::{DType, Expr, env::Environment};

// Type checker function takes an expression and the environment of the block that the expression is in.
// it returns the type of the expression, and records type info about the stack in the environment
// it records an overall type of the stack,
// recording the messages needed to get the values on the stack, and the memory needed for them
// it also records a comptime-type of the stack
// recording the same as for overall but for comptime-available data, and also storing that in a virtual stack
// it checks whether an expression is available at comptime by sending it to the interpreter
// which is basically a type checker that also returns a value if it possible to be evaluated
pub trait TypeCheck {
    fn check(&mut self, env: &mut Environment) -> Result<DType, ()>;
}

impl TypeCheck for Expr {
    fn check(&mut self, env: &mut Environment) -> Result<DType, ()> {
        match self {
            Expr::Unary(_, _) => todo!(),
            Expr::Binary(left, op, right) => match op.ttype {
                TokenType::Equal => {
                    let name  = match *left.clone() {
                        Expr::BinaryOpt(left, Token { ttype: TokenType::Colon, lexeme:_,line:_ }, _) => match *left {
                            Expr::MsgEmission(None, name) => name.lexeme,
                            _ => panic!("expected identifier")
                        },
                        _ => panic!("expected declaration")
                    };
                    // TODO: add name to stack type's msgs
                    match right.clone().interpret(env) {
                        Some((bytes, dtype)) => {
                            fn constructor(self_address: usize, env: Environment, arg: Option<Expr>) -> Expr
                            { Expr::Object(vec![]) }
                            env.add_ct_msg(Msg::new(name, constructor, dtype));
                        },
                        None => todo!(),
                    }
                    right.check(env)
                },
                _ => panic!("expected identifier")
            },
            Expr::MsgEmission(self_expr, msg_name) => {
                let self_t = match self_expr {
                    Some(inner) => inner.check(env)?,
                    None => env.rt_stack_type.clone(),
                };
                for msg in self_t.msgs {
                    if msg.name == msg_name.lexeme {
                        let dtype = msg.construct().check(env)?; // TODO: replace msg emission expression with constructed expression
                        if dtype != msg.ret_type { return Err(()) }
                        return Ok(msg.ret_type)
                    }
                }
                Err(())
            },
            Expr::BinaryOpt(left, op, right_opt) => todo!(),
            Expr::Object(exprs) => {
                let mut size = 0;
                for expr in exprs {
                    // if initialization, add name to object's msgs
                    size += expr.check(env)?.size;
                }
                Ok(DType { size, msgs: vec![] })
            },
            Expr::Call(_, _) => todo!(),
            Expr::CodeBlock(_, exprs) => {
                let mut last = &mut Expr::Object(vec![]);
                for expr in exprs {
                    last = expr;
                }
                last.check(env)
            },
            Expr::Literal(inner) => Ok(DType::from_literal(inner.clone())),
        }
    }
}