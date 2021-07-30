use std::rc::Rc;

use crate::{expr::compiler::dtype::*, token::{Token, TokenType, literal::Literal}};
use super::{Expr, env::Environment, interpreter::Interpret};
pub trait TypeCheck {
    fn check(&mut self, env: &mut Environment) -> Result<DType, ()>;
    fn check_new_env(&mut self) -> Result<DType, ()>;
}

impl TypeCheck for Expr {
    fn check(&mut self, env: &mut Environment) -> Result<DType, ()> {
        match self {
            Expr::Binary(left, op, right) => match op.ttype {
                TokenType::Equal => {
                    let name  = match *left.clone() {
                        Expr::BinaryOpt(left, Token { ttype: TokenType::Colon, lexeme:_,line:_ }, _) => match *left {
                            Expr::MsgEmission(None, name, None) => name.lexeme,
                            _ => panic!("expected identifier")
                        },
                        _ => panic!("expected declaration")
                    }; // TODO: parse declaration data and send value and env to it for it to initialize
                    match right.clone().interpret(env) {
                        Some((bytes, dtype)) => {
                            let mut byte_lits = vec![];
                            for byte in bytes {
                                byte_lits.push(Expr::Literal(Literal::Byte(byte)));
                            }
                            let constructor = move |_: Option<Box<Expr>>, _: &Environment, _: Option<Box<Expr>>|
                            { Expr::Object(byte_lits.clone()) };
                            env.add_ct_msg(Msg::new(name.clone(), Rc::new(constructor), dtype, None));
                        },
                        None => todo!(),
                    }

                    // add runtime msg
                    let dtype = right.check(env)?;
                    let constructor = move |_: Option<Box<Expr>>, _: &Environment, _: Option<Box<Expr>>|
                    { Expr::Object(vec![]) }; // TODO: return asm node
                    env.add_rt_msg(Msg::new(name, Rc::new(constructor), dtype.clone(), None));

                    Ok(dtype)
                },
                _ => panic!("expected identifier")
            },
            Expr::MsgEmission(self_opt, msg_name, arg_opt) => {
                let self_t = match self_opt {
                    Some(inner) => inner.check(env)?,
                    None => env.rt_stack_type.clone(),
                };
                for msg in self_t.msgs {
                    if msg.name == msg_name.lexeme {
                        // check if arg matched msg's arg type
                        if let Some(arg) = arg_opt {
                            let arg_type = match &msg.arg_type {
                                Some(arg_type) => arg_type,
                                None => return Err(()),
                            };
                            if &arg.check(env)? != arg_type { return Err(()) }
                        } else {
                            if let Some(_) = msg.arg_type {
                                return Err(())
                            }
                        }

                        let mut constructed_expr = msg.construct(self_opt.clone(), env, arg_opt.clone());
                        let dtype = constructed_expr.check(env)?;
                        if dtype != msg.ret_type { return Err(()) }
                        *self = constructed_expr;
                        return Ok(dtype)
                    }
                }
                Err(())
            },
            Expr::BinaryOpt(left, op, right_opt) => todo!(),
            Expr::Object(exprs) => {
                let mut size = 0;
                let mut msgs = vec![];
                for expr in exprs {
                    match expr {
                        Expr::Binary(left, op, right) => if let TokenType::Equal = op.ttype {
                            let name  = match *left.clone() {
                                Expr::BinaryOpt(left, op, _) => match op.ttype {
                                    TokenType::Colon => match *left {
                                        Expr::MsgEmission(None, name, None) => name.lexeme,
                                        _ => panic!("expected identifier")
                                    },
                                    _ => panic!("expected declaration")
                                },
                                _ => panic!("expected declaration")
                            };
                            let dtype = right.check(env)?;
                            let constructor = move |self_address: Option<Box<Expr>>, env: &Environment, arg: Option<Box<Expr>>|
                            { Expr::Object(vec![]) }; // TODO: replace with asm node
                            msgs.push(Msg::new(name, Rc::new(constructor), dtype.clone(), None));
                            size += dtype.size;
                        }
                        _ => {
                            size += expr.check(env)?.size;
                        }
                    }
                }
                Ok(DType { size, msgs })
            },
            Expr::CodeBlock(exprs) => {
                let mut last_type = VOID;
                for expr in exprs {
                    println!("{:?}", expr);
                    last_type = expr.check(env)?;
                }
                Ok(last_type)
            },
            Expr::Fn(capture_list, expr) => {
                let mut new_env = Environment::new();
                // TODO: add capture list to new environment
                // for expr in capture_list {
                //     match expr {
                //         Expr::MsgEmission(self_opt, msg_name, arg_opt) => {
                //             // TODO: err if argument is provided
                //             let tmp;
                //             let self_t = match self_opt {
                //                 Some(inner) => {tmp = inner.check(env)?; &tmp},
                //                 None => &env.rt_stack_type,
                //             };
                //             let msg = match self_t.get_msg(&msg_name.lexeme) {
                //                 Some(inner) => inner,
                //                 None => return Err(())
                //             };
                //             new_env.add_rt_msg(msg);
                //         },
                //         _ => return Err(()) // TODO
                //     }
                // }
                expr.check(&mut new_env)?;
                Ok(I32)
            },
            Expr::Type(exprs) => {
                for expr in exprs {
                    let dtype = expr.check(env)?;
                    if dtype != VOID // TODO: replace with types for type and declaration
                    && dtype != I32
                        { panic!("unexpected expression in type definition") }
                }
                todo!() // TODO: replace with type for type
            },
            Expr::Literal(inner) => Ok(DType::from_literal(inner.clone())),
        }
    }

    fn check_new_env(&mut self) -> Result<DType, ()> {
        self.check(&mut Environment::new())
    }
}