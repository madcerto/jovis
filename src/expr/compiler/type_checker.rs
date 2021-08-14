use std::{fmt::{Debug, Display}, rc::Rc};
use crate::{expr::parser::Parser, token::{Token, TokenType, scanner::Scanner}};
use super::{Expr, env::Environment, interpreter::Interpret, core_lib::*, dtype::{DType, Msg}, decl::Decl};

pub trait TypeCheck {
    fn check(&mut self, env: &mut Environment) -> Result<DType, TypeError>;
    fn check_new_env(&mut self) -> Result<DType, TypeError>;
}

impl TypeCheck for Expr {
    fn check(&mut self, env: &mut Environment) -> Result<DType, TypeError> {
        match self {
            Expr::Binary(left, op, right) => if op.ttype == TokenType::Equal {
                let (decl_bytes, decl_type) = match left.interpret(env) {
                    Some(v) => v,
                    None => return Err(TypeError::new("expected static expression".into(), Some(op.clone()))),
                };
                if decl_type != DECL { return Err(TypeError::new("expected declaration expression".into(), Some(op.clone()))) }
                
                let mut decl_slice = [0; 22]; // TODO: find more efficient way to do this
                for i in 0..22 {
                    decl_slice[i] = decl_bytes[i];
                }
                let decl = match Decl::from_bytes(decl_slice, env) {
                    Some(v) => v,
                    None => return Err(TypeError::new("cannot get declaration name from stack".into(), Some(op.clone()))),
                };

                decl.initialize(*right.clone(), env)
            } else { panic!("unexpected binary operator") },
            Expr::MsgEmission(self_opt, msg_name, arg_opt) => {
                let self_t = match self_opt {
                    Some(inner) => inner.check(env)?,
                    None => env.rt_stack_type.clone(),
                };
                match self_t.get_msg(&msg_name.lexeme) {
                    Some(msg) => {
                        // check if arg matched msg's arg type
                        if let Some(arg) = arg_opt {
                            match &msg.arg_type {
                                Some(arg_type) => if &arg.check(env)? != arg_type
                                    { return Err(TypeError::new("argument is of incorrect type".into(), Some(msg_name.clone()))) },
                                None => return Err(TypeError::new("argument passed when not expected".into(), Some(msg_name.clone()))),
                            };
                        } else {
                            if let Some(_) = msg.arg_type {
                                return Err(TypeError::new("expected argument".into(), Some(msg_name.clone())))
                            }
                        }

                        let mut constructed_expr = msg.construct(self_opt.clone(), env, arg_opt.clone());
                        let dtype = constructed_expr.check(env)?;
                        if dtype != msg.ret_type { return Err(TypeError::new("incorrect type of constructed expression".into(), Some(msg_name.clone()))) }
                        *self = constructed_expr;
                        Ok(dtype)
                    },
                    None => Err(TypeError::new(
                        format!("object of type {:?} has no msg {}", self_t, msg_name.lexeme), Some(msg_name.clone())
                    )),
                }
            },
            Expr::BinaryOpt(_left, op, _right_opt) => {
                if op.ttype == TokenType::Semicolon {
                    Ok(DECL)
                } else { panic!("unexpected binary_opt operator") }
            },
            Expr::Asm(_, text_expr) => {
                let text = match text_expr.interpret(env) { // TODO: if string literal, get string directly
                    Some((text_bytes, text_type)) => if text_type == STRING {
                        if text_bytes.len() as u32 == STRING.size {
                            let mut text_slice: [u8; 16] = [0; 16]; // TODO: find more efficient way to do this
                            for i in 0..16 {
                                text_slice[i] = text_bytes[i];
                            }
                            str_from_jstr(text_slice, env).expect("could not get string from stack")
                        }
                        else { panic!("jstr is of incorrect size") }
                    } else { return Err(TypeError::new("expected string".into(), None)) },
                    None => return Err(TypeError::new("expected static expression".into(), None))
                };
                // check embedded jovis expressions
                for (i,_) in text.match_indices("j#") {
                    let mut scanner  = Scanner::new(text.get((i+2)..).unwrap().to_string());
                    let mut parser = Parser::new(scanner.scan_tokens_err_ignore().0);
                    let mut expr = parser.parse();

                    expr.check(env)?;
                }
                // check return expressions
                for (i,_) in text.clone().match_indices("jret#") {
                    let mut n = match text.get((i+5)..) {
                        Some(text) => if let Some((i, _)) = text.match_indices("addr(").next() { i }
                        else if let Some((i, _)) = text.match_indices("val(").next() { i }
                        else { return Err(TypeError::new("expected 'addr' or 'val'".into(), None)) },
                        None => panic!("value of i+5 seems to be too large"),
                    };
                    loop {
                        let c = match text.chars().nth(n) {
                            Some(c) => c,
                            None => panic!("unterminated jret expression in asm"),
                        };
                        if c == ')' { break };
                        n += 1;
                    }
                }
                Ok(DType::new(0, vec![], true, true))
            },
            Expr::Object(exprs) => { // TODO msg body
                let mut size = 0;
                let mut msgs = vec![];
                for expr in exprs {
                    match expr {
                        Expr::Binary(left, op, right) => if op.ttype == TokenType::Equal {
                            let (decl_bytes, decl_type) = match left.interpret(env) {
                                Some(v) => v,
                                None => return Err(TypeError::new("expected static expression".into(), Some(op.clone()))),
                            };
                            if decl_type != DECL { return Err(TypeError::new("expected declaration expression".into(), Some(op.clone()))) }
                            
                            let mut decl_slice = [0; 22]; // TODO: find more efficient way to do this
                            for i in 0..22 {
                                decl_slice[i] = decl_bytes[i];
                            }
                            let decl = match Decl::from_bytes(decl_slice, env) {
                                Some(v) => v,
                                None => return Err(TypeError::new("cannot get declaration name from stack".into(), Some(op.clone()))),
                            };
                            let name  = decl.name;
                            let dtype = decl.dtype;
                            if right.check(env)? != dtype {
                                return Err(TypeError::new("value does not match declaration".into(), None));
                            }
                            let constructor = move |_self_expr: Option<Box<Expr>>, _env: &Environment, _arg: Option<Box<Expr>>|
                            { Expr::Object(vec![]) }; // TODO: replace with asm node, use size for offset
                            size += dtype.size;
                            msgs.push(Msg::new(name, Rc::new(constructor), dtype.clone(), None));
                        }
                        _ => {
                            size += expr.check(env)?.size;
                        }
                    }
                }
                Ok(DType::new(size, msgs, false, true)) // TODO: unknown or not?
            },
            Expr::CodeBlock(exprs) => {
                let mut last_type = DType::new(0, vec![], false, true);
                for expr in exprs {
                    last_type = expr.check(env)?;
                }
                Ok(last_type)
            },
            Expr::Fn(capture_list, expr) => { // TODO
                let mut new_env = Environment::new();
                // add capture list to new environment
                for expr in capture_list {
                    // TODO: try interpret the expression
                    match expr {
                        Expr::MsgEmission(self_opt, msg_name, arg_opt) => {
                            match arg_opt {
                                Some(_) => {
                                    let dtype = expr.check(env)?;
                                    new_env.rt_stack_type.compose(dtype);
                                },
                                None => {
                                    let tmp;
                                    let self_t = match self_opt {
                                        Some(inner) => {tmp = inner.check(env)?; &tmp},
                                        None => &env.rt_stack_type,
                                    };
                                    let msg = match self_t.get_msg(&msg_name.lexeme) {
                                        Some(inner) => inner,
                                        None => return Err(TypeError::new("".into(), Some(msg_name.clone())))
                                    };
                                    // TODO: transform all references to stack base to reference a stack value
                                    // which stores old stack base
                                    new_env.add_rt_msg(msg);
                                },
                            }
                        },
                        _ => {
                            let dtype = expr.check(env)?;
                            new_env.rt_stack_type.compose(dtype);
                        } // TODO
                    }
                }
                expr.check(&mut new_env)?;
                Ok(FN)
            },
            Expr::Type(exprs) => {
                for expr in exprs {
                    let dtype = expr.check(env)?;
                    if dtype != TYPE
                    && dtype != FN
                        { return Err(TypeError::new("unexpected expression in type definition".into(), None)) }
                }
                Ok(TYPE)
            },
            Expr::Literal(inner) => Ok(DType::from_literal(inner.clone())),
        }
    }

    fn check_new_env(&mut self) -> Result<DType, TypeError> {
        self.check(&mut Environment::new())
    }
}

pub struct TypeError {
    msg: String,
    tkn_opt: Option<Token>
}
impl TypeError {
    pub fn new(msg: String, tkn_opt: Option<Token>) -> Self { Self { msg, tkn_opt } }
}
impl Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.tkn_opt {
            Some(tkn) => write!(f, "err: {} at {}", self.msg, tkn.to_string()),
            None => write!(f, "err: {}", self.msg)
        }
    }
}
impl Debug for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}