use std::{rc::Rc, str::FromStr};
use super::{Expr, Environment, DType, dtype::Msg, core_lib::*, TypeCheck, decl::Decl};
use crate::{expr::compiler::fill_slice_with_vec, token::{Token, TokenType, literal::Literal}};

pub trait Interpret {
    fn interpret(&mut self, env: &mut Environment) -> Option<(Vec<u8>, DType)>;
    fn interpret_new_env(&mut self) -> Option<(Vec<u8>, DType)>;
}

impl Interpret for Expr {
    fn interpret(&mut self, env: &mut Environment) -> Option<(Vec<u8>, DType)> {
        match self {
            Expr::Binary(left, op, right) => if op.ttype == TokenType::Equal {
                let (decl_bytes, decl_type) = match left.interpret(env) {
                    Some(v) => v,
                    None => return None,
                };
                if decl_type != DECL { return None }
                
                let mut decl_slice = [0; 22]; // TODO: find more efficient way to do this
                for i in 0..22 {
                    decl_slice[i] = decl_bytes[i];
                }
                let decl = match Decl::from_bytes(decl_slice, env) {
                    Some(v) => v,
                    None => return None,
                };
                
                decl.ct_initialize(*right.clone(), env)
            } else { panic!("unexpected binary operator") },
            Expr::MsgEmission(self_opt, msg_name, arg_opt) => {
                let self_t = match self_opt {
                    Some(inner) => inner.interpret(env)?.1,
                    None => env.get_ct_stack_type(),
                };
                let msg = self_t.get_msg(&msg_name.lexeme)?;
                // check if arg matched msg's arg type
                if let Some(arg) = arg_opt {
                    let arg_type = match &msg.arg_type {
                        Some(arg_type) => arg_type,
                        None => return None,
                    };
                    if &arg.interpret(env)?.1 != arg_type { return None }
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
            Expr::BinaryOpt(left, op, right_opt) => {
                match op.ttype {
                    TokenType::Semicolon => {
                        let name = match *left.clone() {
                            Expr::MsgEmission(None, name, None) => name.lexeme,
                            _ => panic!("expected identifier")
                        };
                        let mut bytes = vec![];
                        name.chars().for_each(|c|{ bytes.push(c as u8) });
                        let str_size = bytes.len();
                        let addr = env.push(bytes);
                        let mut name = addr.to_ne_bytes().to_vec();
                        name.extend_from_slice(&str_size.to_ne_bytes());

                        let type_opt = right_opt.as_mut().map(|right|
                            right.interpret(env).map(|(bytes, dtype)| {
                                // check dtype
                                if dtype != TYPE { return None }
                                Some(bytes)
                            }).unwrap_or(None)
                        ).unwrap_or(None);
                        let type_bytes = type_opt.unwrap_or(vec![0,0,0,0,1,1]);

                        let mut decl_bytes = name;
                        decl_bytes.extend(type_bytes.into_iter());
                        Some((decl_bytes, DECL))
                    },
                    _ => panic!("unexpected operator in binary_opt")
                }
            },
            Expr::Asm(_, _, text_expr) => { // TODO
                let mut text = match text_expr.interpret(env) { // TODO: if string literal, get string directly
                    Some((text_bytes, text_type)) => if text_type == STRING {
                        if text_bytes.len() as u32 == STRING.size {
                            let mut text_slice = [0; 16]; // TODO: find more efficient way to do this
                            for i in 0..16 {
                                text_slice[i] = text_bytes[i];
                            }
                            str_from_jstr(text_slice, env).expect("could not get string from stack")
                        }
                        else { panic!("jstr is of incorrect size") }
                    } else { return None },
                    None => return None
                };
                // TODO
                // for (i,_) in text.match_indices("j#") {
                //     // TODO: handle scanner and parser errors
                //     let mut scanner  = Scanner::new(text.get((i+2)..).unwrap().to_string());
                //     let mut parser = Parser::new(scanner.scan_tokens_err_ignore());
                //     let mut expr = parser.parse();

                //     let _etype = expr.interpret(env)?;
                //     // TODO: replace expression in string with generated code for expression
                // }
                let mut val = vec![];
                for (i,_) in text.clone().match_indices("jret(") {
                    let mut n = i+5;
                    let mut operand = "".to_string();
                    loop {
                        let c = text.chars().nth(n)?;
                        if c == ')' { break };
                        operand.push(c); n+=1
                    }
                    let addr = usize::from_str(operand.trim()).ok()?;
                    val = env.get_stack(addr)?.clone();
                    // remove return from text
                    text.replace_range(i..(n+1), "");
                }
                // TODO: simulate running assembly
                Some((val, VOID))
            },
            Expr::Object(exprs) => {
                let mut bytes = vec![];
                let mut msgs = vec![];
                let mut size = 0;
                for expr in exprs {
                    match expr {
                        Expr::Binary(left, Token { ttype: TokenType::Equal, .. }, right) => {
                            let decl = Decl::from_expr(left, env)?;
                            let val = right.interpret(env);
                            if let Some((mut val_bytes, dtype)) = val {
                                bytes.append(&mut val_bytes);
                                let byte_lits: Vec<Expr> = val_bytes.iter().map(|b|
                                    Expr::Literal(Literal::Byte(*b))
                                ).collect();
                                let constructor = move |_: Option<Box<Expr>>, _: &Environment, _: Option<Box<Expr>>|
                                    { Expr::Object(byte_lits.clone()) };
                                msgs.push(Msg::new(decl.name, Rc::new(constructor), dtype.clone(), None));
                                size += dtype.size;
                            } else { return None }
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
                Some((bytes,  DType::new(size, msgs, false, true)))
            },
            Expr::CodeBlock(exprs) => {
                let mut last_bytes = vec![];
                let mut last_type = DType::new(0, vec![], false, true);
                for expr in exprs {
                    let (bytes, dtype) = expr.interpret(env)?;
                    last_bytes = bytes;
                    last_type = dtype;
                }
                Some((last_bytes, last_type))
            },
            Expr::Fn(_capture_list, _expr) => None, // TODO
            Expr::Type(exprs) => { // TODO
                let mut type_val = VOID;
                for expr in exprs {
                    let dtype = expr.check(env).ok()?;
                    if dtype == TYPE {
                        let (bytes,_) = match expr.interpret(env) {
                            Some(v) => v,
                            None => return None,
                        };
                        let composing_type = if bytes.len() as u32 == TYPE.size {
                            let mut type_slice = [0; 6]; // TODO: find more efficient way to do this
                            for i in 0..6 {
                                type_slice[i] = bytes[i];
                            }
                            DType::from_bytes(type_slice)
                        } else { panic!("value of unexpected size") };
                        type_val.size += composing_type.size;
                        type_val.msgs.extend(composing_type.msgs.into_iter());
                    }
                    else if dtype == DECL {
                        let mut decl_slice = [0; 22];
                        fill_slice_with_vec(&mut decl_slice, expr.interpret(env)?.0);
                        let decl = Decl::from_bytes(decl_slice, env).unwrap(); // TODO: error handling
                        let name = decl.name;
                        let composing_type = decl.dtype;
                        type_val.size += composing_type.size;
                        let constructor = move |_: Option<Box<Expr>>, _: &Environment, _: Option<Box<Expr>>|
                        { Expr::Object(vec![]) }; // TODO: add asm node
                        type_val.msgs.push(Msg::new(name, Rc::new(constructor), composing_type, None));
                    }
                    else { return None }
                }
                Some((type_val.to_bytes(), TYPE))
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