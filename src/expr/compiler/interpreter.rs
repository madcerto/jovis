use std::rc::Rc;

use super::{Expr, Environment, DType, dtype::Msg, core_lib::*, TypeCheck};
use crate::{expr::parser::Parser, token::{Token, TokenType, literal::Literal, scanner::Scanner}};

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
                            let constructor = move |self_address: Option<Box<Expr>>, env: &Environment, arg: Option<Box<Expr>>|
                            { Expr::Object(byte_lits.clone()) };
                            env.add_ct_msg(Msg::new(name.clone(), Rc::new(constructor), dtype, None));
                        },
                        None => return None,
                    }

                    val
                },
                _ => panic!("unexpected binary operator")
            },
            Expr::MsgEmission(self_opt, msg_name, arg_opt) => {
                // TODO: check if arg matched msg's arg type
                let self_t = match self_opt {
                    Some(inner) => inner.interpret(env)?.1,
                    None => env.ct_stack_type.clone(),
                };
                for msg in self_t.msgs {
                    // check if arg matched msg's arg type
                    if let Some(arg) = arg_opt {
                        let arg_type = match &msg.arg_type {
                            Some(arg_type) => arg_type,
                            None => return None,
                        };
                        if &arg.check(env).ok()? != arg_type { return None }
                    } else {
                        if let Some(_) = msg.arg_type {
                            return None
                        }
                    }
                    if msg.name == msg_name.lexeme {
                        // TODO: send self_expr
                        let mut constructed_expr = msg.construct(self_opt.clone(), env, arg_opt.clone());
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
                    TokenType::Semicolon => {
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
                                    // None => panic!(format!("type in declaration is not static at {}", op.to_string())),
                                    None => None
                                }
                            },
                            None => None
                        };
                        todo!() // TODO: return declaration data type
                    },
                    _ => panic!("unexpected operator in binary_opt")
                }
            },
            Expr::Asm(_, code_expr) => {
                let code = match code_expr.interpret(env) {
                    Some((code_bytes, code_type)) => if code_type == STRING {
                        if code_bytes.len() == STRING.size {
                            let mut code_slice: [u8; 16] = [0; 16]; // TODO: find more efficient way to do this
                            for i in 0..12 {
                                code_slice[i] = code_bytes[i];
                            }
                            str_from_jstr(code_slice, env).expect("could not get string from stack")
                        }
                        else { panic!("jstr is of incorrect size") } // TODO: do these need to be reported to the user?
                    } else { return None },
                    None => return None
                };
                for (i,_) in code.match_indices("j#") {
                    // TODO: handle scanner and parser errors
                    let mut scanner  = Scanner::new(code.get((i+2)..).unwrap().to_string());
                    let mut parser = Parser::new(scanner.scan_tokens_err_ignore());
                    let mut expr = parser.parse();

                    let etype = expr.interpret(env)?;
                    // TODO: replace expression in string with generated code for expression
                }
                // TODO: simulate running assembly
                todo!()
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
                                    TokenType::Semicolon => match *left {
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
                                    let constructor = move |_: Option<Box<Expr>>, _: &Environment, _: Option<Box<Expr>>|
                                    { Expr::Object(byte_lits.clone()) };
                                    msgs.push(Msg::new(name, Rc::new(constructor), dtype.clone(), None));
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
                let mut last_type = VOID;
                for expr in exprs {
                    let (bytes, dtype) = expr.interpret(env)?;
                    last = bytes;
                    last_type = dtype 
                } // TODO: find more efficient way to do this
                Some((last, last_type))
            },
            Expr::Fn(_capture_list, _expr) => None,
            Expr::Type(_) => None,
            Expr::Literal(inner) => match inner.clone() {
                Literal::String(val) => {
                    let mut bytes = vec![];
                    val.chars().for_each(|c|{ bytes.push(c as u8) });
                    let str_size = bytes.len();
                    let addr = env.push(bytes);
                    let mut val = addr.to_ne_bytes().to_vec();
                    val.extend_from_slice(&str_size.to_ne_bytes());
                    Some((val, DType::from_literal(inner.clone())))
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