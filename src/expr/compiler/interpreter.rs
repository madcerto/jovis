use std::rc::Rc;

use super::{Expr, Environment, DType, dtype::Msg};
use crate::{token::{TokenType, literal::Literal}};

pub trait Interpret {
    fn interpret(&mut self, env: &mut Environment) -> Option<(Vec<u8>, DType)>;
    fn interpret_new_env(&mut self) -> Option<(Vec<u8>, DType)>;
}

impl Interpret for Expr {
    fn interpret(&mut self, env: &mut Environment) -> Option<(Vec<u8>, DType)> {
        match self {
            Expr::Binary(left, op, right) => match op.ttype {
                TokenType::Equal => {
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
                    let val = right.interpret(env);
                    match val.clone() {
                        Some((bytes, dtype)) => {
                            let mut byte_lits = vec![];
                            for byte in bytes.clone() { byte_lits.push(Expr::Literal(Literal::Byte(byte))) }
                            let constructor = move |self_address: Expr, env: &Environment, arg: Option<Expr>|
                            { Expr::Object(byte_lits.clone()) };
                            env.add_ct_msg(Msg::new(name.clone(), Rc::new(constructor), dtype));
                        },
                        None => return None,
                    }

                    val
                },
                _ => panic!("unexpected binary operator")
            },
            Expr::MsgEmission(self_expr, msg_name, arg_opt) => {
                // TODO: check if arg matched msg's arg type
                let self_t = match self_expr {
                    Some(inner) => inner.interpret(env)?.1,
                    None => env.ct_stack_type.clone(),
                };
                for msg in self_t.msgs {
                    if msg.name == msg_name.lexeme {
                        // TODO: send self_expr and arg
                        let mut constructed_expr = msg.construct(Expr::Object(vec![]), env, None);
                        let (bytes, dtype) = constructed_expr.interpret(env)?;
                        if dtype != msg.ret_type { return None }
                        *self = constructed_expr;
                        return Some((bytes, dtype)) // TODO: emit message
                    }
                }
                None
            },
            Expr::BinaryOpt(left, op, right_opt) => {
                match op.ttype {
                    TokenType::Colon => {
                        let name = match *left.clone() {
                            Expr::MsgEmission(None, name, None) => name.lexeme,
                            _ => panic!("expected identifier")
                        };
                        let type_opt = match right_opt {
                            Some(right) => {
                                match right.interpret(env) {
                                    Some((bytes, dtype)) => {
                                        // TODO: check dtype
                                        // TODO: convert bytes to DType
                                        Some(bytes)
                                    },
                                    None => panic!("type in declaration is not static"),
                                }
                            },
                            None => None
                        };
                        todo!() // TODO: return declaration data type
                    },
                    _ => panic!("unexpected operator in binary_opt")
                }
            },
            Expr::Object(exprs) => {
                let mut bytes = vec![];
                let mut msgs = vec![];
                let mut size = 0;
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
                            let val = right.interpret(env);
                            match val {
                                Some((mut val_bytes, dtype)) => {
                                    bytes.append(&mut val_bytes);
                                    let mut byte_lits = vec![];
                                    for byte in val_bytes.clone() { byte_lits.push(Expr::Literal(Literal::Byte(byte))) }
                                    let constructor = move |self_address: Expr, env: &Environment, arg: Option<Expr>|
                                    { Expr::Object(byte_lits.clone()) };
                                    msgs.push(Msg::new(name, Rc::new(constructor), dtype.clone() ));
                                    size += dtype.size;
                                },
                                None => return None,
                            }
                        }
                        _ => {
                            let val = expr.interpret(env);
                            match val {
                                Some((mut val_bytes, dtype)) => {
                                    bytes.append(&mut val_bytes);
                                    size += dtype.size;
                                },
                                None => return None,
                            }
                        }
                    }
                }
                Some((bytes,  DType { size, msgs }))
            },
            Expr::CodeBlock(exprs) => {
                let mut last = vec![];
                let mut last_type = DType { size: 0, msgs: vec![] };
                for expr in exprs {
                    let (last, dtype) = expr.interpret(env)?;
                }
                Some((last, last_type))
            },
            Expr::Fn(capture_list, expr) => todo!(),
            Expr::Type(exprs) => todo!(),
            // Expr::Identifier(name) => env.get(name.lexeme),
            Expr::Literal(inner) => match inner.clone() {
                Literal::String(val) => {
                    let mut bytes = vec![];
                    val.chars().for_each(|c|{ bytes.push(c as u8) });
                    Some((bytes, DType::from_literal(inner.clone())))
                },
                Literal::Char(c) => Some((vec![c as u8], DType::from_literal(inner.clone()))),
                Literal::Integer(i) => {
                    let mut vec = vec![];
                    vec.extend_from_slice(&i.to_le_bytes());
                    Some((vec, DType::from_literal(inner.clone())))
                },
                Literal::Float(f) => {
                    let mut vec = vec![];
                    vec.extend_from_slice(&f.to_le_bytes());
                    Some((vec, DType::from_literal(inner.clone())))
                },
                Literal::Byte(b) => Some((vec![b], DType::from_literal(inner.clone()))),
            },
        }
    }

    fn interpret_new_env(&mut self) -> Option<(Vec<u8>, DType)> {
        self.interpret(&mut Environment::new())
    }
}