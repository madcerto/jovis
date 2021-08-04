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
            Expr::Binary(left, op, right) => match op.ttype { // TODO
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
                            let constructor = move |_: Option<Box<Expr>>, _: &Environment, _: Option<Box<Expr>>|
                            { Expr::Object(byte_lits.clone()) };
                            env.add_ct_msg(Msg::new(name.clone(), Rc::new(constructor), dtype, None));
                        },
                        None => return None,
                    }

                    val
                },
                _ => panic!("unexpected binary operator")
            },
            Expr::MsgEmission(self_opt, msg_name, arg_opt) => { // TODO
                let self_t = match self_opt {
                    Some(inner) => inner.interpret(env)?.1,
                    None => env.ct_stack_type.clone(),
                };
                let msg = self_t.get_msg(&msg_name.lexeme)?;
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
                let mut constructed_expr = msg.construct(self_opt.clone(), env, arg_opt.clone());
                let (bytes, dtype) = constructed_expr.interpret(env)?;
                if dtype != msg.ret_type { return None }
                *self = constructed_expr;
                Some((bytes, dtype))
            },
            Expr::BinaryOpt(left, op, right_opt) => { // TODO
                match op.ttype {
                    TokenType::Semicolon => {
                        let _name = match *left.clone() {
                            Expr::MsgEmission(None, name, None) => name.lexeme,
                            _ => panic!("expected identifier")
                        };
                        let _type_opt = match right_opt {
                            Some(right) => {
                                match right.interpret(env) {
                                    Some((bytes, _dtype)) => {
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
            Expr::Asm(_, code_expr) => { // TODO
                let code = match code_expr.interpret(env) {
                    Some((code_bytes, code_type)) => if code_type == STRING {
                        if code_bytes.len() == STRING.size {
                            let mut code_slice = [0; 16]; // TODO: find more efficient way to do this
                            for i in 0..16 {
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

                    let _etype = expr.interpret(env)?;
                    // TODO: replace expression in string with generated code for expression
                }
                // TODO: simulate running assembly
                todo!()
            },
            Expr::Object(exprs) => { // TODO
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
                let mut last_bytes = vec![];
                let mut last_type = VOID;
                for expr in exprs {
                    let (bytes, dtype) = expr.interpret(env)?;
                    last_bytes = bytes;
                    last_type = dtype 
                }
                Some((last_bytes, last_type))
            },
            Expr::Fn(_capture_list, _expr) => None, // TODO
            Expr::Type(exprs) => {
                let mut type_val = VOID;
                for expr in exprs {
                    let dtype = expr.check(env).ok()?;
                    if dtype == TYPE {
                        let (bytes,_) = match expr.interpret(env) {
                            Some(v) => v,
                            None => return None,
                        };
                        let composing_type = if bytes.len() == TYPE.size {
                            let mut type_slice = [0; 4]; // TODO: find more efficient way to do this
                            for i in 0..4 {
                                type_slice[i] = bytes[i];
                            }
                            DType::from_bytes(type_slice)
                        } else { panic!("value of unexpected size") };
                        type_val.size += composing_type.size;
                        type_val.msgs.extend(composing_type.msgs.into_iter());
                    }
                    else if dtype == I32 { // replace with decl type
                        let (name, type_expr) = match expr {
                            Expr::BinaryOpt(left, Token { ttype: TokenType::Semicolon, lexeme:_,line:_ }, right) => match right {
                                Some(v) => (
                                        match &**left {
                                            Expr::MsgEmission(None, name, None) => name.lexeme.clone(),
                                            _ => panic!("expected identifier")
                                        },
                                    v),
                                None => panic!("type inference for declarations in types is not yet implemented"),
                            },
                            _ => panic!("expected declaration")
                        };
                        let (bytes,_) = match type_expr.interpret(env) {
                            Some(v) => v,
                            None => return None,
                        };
                        let composing_type = if bytes.len() == TYPE.size {
                            let mut type_slice = [0; 4]; // TODO: find more efficient way to do this
                            for i in 0..4 {
                                type_slice[i] = bytes[i];
                            }
                            DType::from_bytes(type_slice)
                        } else { panic!("value has unexpected size") };
                        type_val.size += composing_type.size;
                        let constructor = move |_: Option<Box<Expr>>, _: &Environment, _: Option<Box<Expr>>|
                        { Expr::Object(vec![]) }; // TODO: add asm node
                        type_val.msgs.push(Msg::new(name, Rc::new(constructor), composing_type, None));
                    }
                    else { return None }
                }
                Some((type_val.size.to_ne_bytes().to_vec(), TYPE))
            },
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