use std::{fmt::{Debug, Display}, rc::Rc};
use crate::{expr::parser::Parser, pprint::PPrint, token::{Token, TokenType, literal::Literal, scanner::Scanner}};
use super::{Expr, core_lib::*, decl::Decl, dtype::{DType, Msg}, env::Environment, fill_slice_with_vec, interpreter::Interpret};

pub trait TypeCheck {
    fn check(&mut self, env: &mut Environment) -> Result<DType, TypeError>;
    fn check_new_env(&mut self) -> Result<DType, TypeError>;
    fn to_syntax(&self) -> String;
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
                
                let mut decl_slice = [0; 22];
                fill_slice_with_vec(&mut decl_slice, decl_bytes);
                let decl = match Decl::from_bytes(decl_slice, env) {
                    Some(v) => v,
                    None => return Err(TypeError::new("cannot get declaration name from stack".into(), Some(op.clone()))),
                };

                decl.initialize(right, env)
            } else { panic!("unexpected binary operator") },
            Expr::MsgEmission(self_opt, msg_name, arg_opt) => {
                let self_t = match self_opt {
                    Some(inner) => inner.check(env)?,
                    None => env.get_rt_stack_type(),
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
            Expr::Asm(_, ret_type, text_expr) => {
                let mut text = match *text_expr.clone() {
                    Expr::Literal(Literal::String(string)) => string,
                    _ => match text_expr.interpret(env) { // TODO: if string literal, get string directly
                        Some((text_bytes, text_type)) => if text_type == STRING {
                            if text_bytes.len() as u32 == STRING.size {
                                let mut text_slice: [u8; 16] = [0; 16];
                                fill_slice_with_vec(&mut text_slice, text_bytes);
                                str_from_jstr(text_slice, env).expect("could not get string from stack")
                            }
                            else { panic!("jstr is of incorrect size") }
                        } else { return Err(TypeError::new("expected string".into(), None)) },
                        None => return Err(TypeError::new("expected static expression".into(), None))
                    }
                };
                
                // check embedded jovis expressions
                for (i,_) in text.clone().match_indices("j#") {
                    let mut scanner  = Scanner::new(text.get((i+2)..).unwrap().to_string());
                    let (tokens, n) = scanner.scan_tokens_err_ignore();
                    let mut parser = Parser::new(tokens);
                    let mut expr = parser.parse();

                    expr.check(env)?;
                    text.replace_range((i+2)..(n+i), expr.to_syntax().as_str());
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
                
                **text_expr = Expr::Literal(Literal::String(text));

                let mut ret_type_slice = [0; 6];
                let ret_type_bytes = ret_type.interpret(env)
                    .ok_or(TypeError::new("expected static expression for asm return type".into(), None))?.0;
                fill_slice_with_vec(&mut ret_type_slice, ret_type_bytes);
                let ret_type = DType::from_bytes(ret_type_slice);

                Ok(ret_type)
            },
            Expr::Object(exprs) => { // TODO msg body
                let mut size = 0;
                let mut msgs = vec![];
                for expr in exprs {
                    match expr {
                        Expr::Binary(left, op, right) => if op.ttype == TokenType::Equal {
                            let (decl_bytes, decl_type) = left.interpret(env)
                                .ok_or(TypeError::new("expected static expression".into(), Some(op.clone())))?;
                            if decl_type != DECL { return Err(TypeError::new("expected declaration expression".into(), Some(op.clone()))) }
                            
                            let mut decl_slice = [0; 22];
                            fill_slice_with_vec(&mut decl_slice, decl_bytes);
                            let decl = Decl::from_bytes(decl_slice, env)
                                .ok_or(TypeError::new("cannot get declaration name from stack".into(), Some(op.clone())))?;
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
            Expr::Fn(capture_list, expr) => { // 2 TODOs 1 future
                let mut new_env = Environment::new();
                // add capture list to new environment
                for expr in capture_list {
                    match expr.clone() {
                        Expr::MsgEmission(_, msg_name, arg_opt) => {
                            match arg_opt {
                                Some(_) => {
                                    let _dtype = expr.check(env)?;
                                    todo!() // captured expressions have to be named
                                },
                                None => {
                                    let (bytes, dtype) = expr.interpret(env)
                                        .ok_or(TypeError::new("expected static expression".into(), Some(msg_name.clone())))?;
                                    let byte_lits: Vec<Expr> = bytes.iter().map(|b| Expr::Literal(Literal::Byte(*b))).collect();
                                    let constructor = move |_: Option<Box<Expr>>, _: &Environment, _: Option<Box<Expr>>|
                                        { Expr::Object(byte_lits.clone()) };
                                    let msg = Msg::new(msg_name.lexeme, Rc::new(constructor), dtype, None);
                                    new_env.add_rt_msg(msg.clone());
                                    new_env.add_ct_msg(msg);
                                    new_env.push(bytes);
                                },
                            }
                        },
                        Expr::Binary(mut left, Token{ ttype: TokenType::Equal, lexeme, line }, mut right) => {
                            let tkn_opt = Some(Token::new(TokenType::Equal, lexeme, line));
                            let msg_name = if let Expr::BinaryOpt(_, Token{ ttype: TokenType::Semicolon, lexeme, line }, _) = *left.clone() {
                                let tkn_opt = Some(Token::new(TokenType::Semicolon, lexeme, line));
                                let decl = Decl::from_expr(&mut *left, env)
                                    .ok_or(TypeError::new("could not form declaration".into(), tkn_opt))?;
                                decl.name
                            }
                            else { return Err(TypeError::new("expected declaration".into(), tkn_opt)) };

                            let (bytes, dtype) = right.interpret(env) // TODO: make expr be mutated here
                                .ok_or(TypeError::new("expected static expression".into(), tkn_opt))?;
                            let byte_lits: Vec<Expr> = bytes.iter().map(|b| Expr::Literal(Literal::Byte(*b))).collect();
                            let constructor = move |_: Option<Box<Expr>>, _: &Environment, _: Option<Box<Expr>>|
                                { Expr::Object(byte_lits.clone()) };
                            let msg = Msg::new(msg_name, Rc::new(constructor), dtype, None);
                            new_env.add_rt_msg(msg.clone());
                            new_env.add_ct_msg(msg);
                            new_env.push(bytes);
                        },
                        _ => {
                            // let (val, dtype) = expr.interpret(&mut new_env)
                            //     .ok_or(TypeError::new("captured expression is not static".into(), None))?;
                            // for msg in dtype.msgs {
                            //     new_env.add_rt_msg(msg.clone());
                            //     new_env.add_ct_msg(msg);
                            // }
                            // new_env.add_rt_size(dtype.size);
                            // new_env.add_ct_size(dtype.size);
                            // new_env.push(val);
                            return Err(TypeError::new("unnamed captures not supported yet. put your value in an assignment".into(), None))
                        }
                    }
                }
                expr.check(&mut new_env)?;
                new_env.propogate_fns(env);
                env.add_fn(new_env);
                Ok(FN)
            },
            Expr::Type(exprs) => {
                for expr in exprs {
                    let dtype = expr.check(env)?;
                    if dtype != TYPE
                    && dtype != DECL
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

    fn to_syntax(&self) -> String {
        match self {
            Expr::Binary(left, op, right) => {
                let mut str = left.to_syntax();
                str.push_str(&op.lexeme);
                str.push_str(right.to_syntax().as_str());
                str
            },
            Expr::MsgEmission(_, _, _) => panic!("unexpected msg emission in checked ast"),
            Expr::BinaryOpt(left, op, right_opt) => {
                let mut str = left.to_syntax();
                str.push_str(&op.lexeme);
                if let Some(right) = right_opt {
                    str.push_str(right.to_syntax().as_str());
                }
                str
            },
            Expr::Asm(asm_type, ret_type, text) => {
                let mut str = "asm ".to_string();
                str.push_str(asm_type.to_syntax().as_str());
                str.push_str(ret_type.to_syntax().as_str());
                str.push_str(text.to_syntax().as_str());
                str
            },
            Expr::Object(exprs) => {
                let mut str = "[ ".to_string();

                for expr in exprs {
                    str.push_str(expr.to_syntax().as_str());
                    str.push_str(" ");
                }

                str.push(']');
                str
            },
            Expr::Fn(exprs, expr) => {
                let mut str = "|".to_string();
                for expr in exprs {
                    str.push_str(expr.to_syntax().as_str());
                    str.push_str(" ");
                }
                str.push('|');
                str.push_str(expr.to_syntax().as_str());
                str
            },
            Expr::CodeBlock(exprs) => {
                let mut str = "{ ".to_string();

                for expr in exprs {
                    str.push_str(expr.to_syntax().as_str());
                    str.push_str(" ");
                }

                str.push('}');
                str
            },
            Expr::Type(exprs) => {
                let mut str = "( ".to_string();

                for expr in exprs {
                    str.push_str(expr.to_syntax().as_str());
                    str.push_str(" ");
                }

                str.push(')');
                str
            },
            Expr::Literal(inner) => inner.prettify(),
        }
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