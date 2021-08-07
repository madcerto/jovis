use std::{fmt::{Debug, Display}, rc::Rc, str::FromStr};
use crate::{expr::parser::Parser, token::{Token, TokenType, literal::Literal, scanner::Scanner}};
use super::{Expr, env::Environment, interpreter::Interpret, core_lib::*, dtype::{DType, Msg}};

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
            } else { panic!("expected identifier") },
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
            Expr::Asm(_, code_expr) => { // TODO
                let code = match code_expr.interpret(env) { // TODO: if string literal, get string directly
                    Some((code_bytes, code_type)) => if code_type == STRING {
                        if code_bytes.len() as u32 == STRING.size {
                            let mut code_slice: [u8; 16] = [0; 16]; // TODO: find more efficient way to do this
                            for i in 0..16 {
                                code_slice[i] = code_bytes[i];
                            }
                            str_from_jstr(code_slice, env).expect("could not get string from stack")
                        }
                        else { panic!("jstr is of incorrect size") } // TODO: do these need to be reported to the user?
                    } else { return Err(TypeError::new("expected string".into(), None)) },
                    None => return Err(TypeError::new("expected static expression".into(), None))
                };
                for (i,_) in code.match_indices("j#") {
                    // TODO: handle scanner and parser errors
                    let mut scanner  = Scanner::new(code.get((i+2)..).unwrap().to_string());
                    let mut parser = Parser::new(scanner.scan_tokens_err_ignore());
                    let mut expr = parser.parse();

                    let _etype = expr.check(env)?;
                    // TODO: replace expression in string with generated code for expression
                }
                // TODO: check if return has ending parentheses and if it's operand is a number
                for (i,_) in code.match_indices("jret(") {
                    let mut n = i+5;
                    loop {
                        let c = match code.chars().nth(n) {
                            Some(c) => c,
                            None => return Err(TypeError::new("unterminated jret expression in asm".into(), None)),
                        };
                        if c == ')' { break }; n+=1
                    }
                    let text = code.get(i..n).unwrap();
                    match usize::from_str(text) {
                        Ok(v) => v,
                        Err(e) => return Err(TypeError::new(format!("error parsing string into int: {}", e), None)),
                    };
                }
                Ok(VOID)
            },
            Expr::Object(exprs) => { // TODO
                let mut size = 0;
                let mut msgs = vec![];
                for expr in exprs {
                    match expr {
                        Expr::Binary(left, op, right) => if op.ttype == TokenType::Equal {
                            let name  = match *left.clone() {
                                Expr::BinaryOpt(left, op, _) => match op.ttype {
                                    TokenType::Semicolon => match *left {
                                        Expr::MsgEmission(None, name, None) => name.lexeme,
                                        _ => panic!("expected identifier")
                                    },
                                    _ => panic!("expected declaration")
                                },
                                Expr::MsgEmission(_, _, arg_opt) => match *arg_opt.expect("expected declaration").clone() {
                                    Expr::BinaryOpt(left, op, _) => match op.ttype {
                                        TokenType::Semicolon => match *left {
                                            Expr::MsgEmission(None, name, None) => name.lexeme,
                                            _ => panic!("expected identifier")
                                        },
                                        _ => panic!("expected declaration")
                                    },
                                    _ => panic!(format!("expected declaration at {}", op.to_string()))
                                },
                                _ => panic!(format!("expected declaration at {}", op.to_string()))
                            };
                            let dtype = right.check(env)?;
                            let constructor = move |_self_expr: Option<Box<Expr>>, _env: &Environment, _arg: Option<Box<Expr>>|
                            { Expr::Object(vec![]) }; // TODO: replace with asm node
                            msgs.push(Msg::new(name, Rc::new(constructor), dtype.clone(), None));
                            size += dtype.size;
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

pub struct Decl {
    pub name: String,
    pub dtype: DType
}
impl Decl {
    pub fn from_bytes(bytes: [u8; 22], env: &mut Environment) -> Option<Self> {
        let mut name_bytes = [0; 16]; // TODO: find more efficient way to do this
        for i in 0..16 {
            name_bytes[i] = bytes[i];
        }
        let name = str_from_jstr(name_bytes, env)?;

        let mut type_bytes = [0; 6]; // TODO: find more efficient way to do this
        for i in 0..6 {
            type_bytes[i] = bytes[i+16];
        }
        let dtype = DType::from_bytes(type_bytes);

        Some(Self { name, dtype })
    }

    pub fn initialize(&self, mut val: Expr, env: &mut Environment) -> Result<DType, TypeError> {
        let dtype = val.check(env)?;
        println!("{:?} {:?}", self.dtype, dtype);
        if self.dtype != dtype { return Err(TypeError::new("initialization value does not match declared type".into(), None)) }

        match val.interpret(env) {
            Some((bytes, ct_dtype)) => {
                let mut byte_lits = vec![];
                for byte in bytes {
                    byte_lits.push(Expr::Literal(Literal::Byte(byte)));
                }
                let constructor = move |_: Option<Box<Expr>>, _: &Environment, _: Option<Box<Expr>>|
                { Expr::Object(byte_lits.clone()) };
                if ct_dtype == dtype {
                    env.add_rt_msg(Msg::new(self.name.clone(), Rc::new(constructor.clone()), dtype.clone(), None));
                } else {
                    // add runtime msg TODO: defer code to a function
                    let constructor = move |_: Option<Box<Expr>>, _: &Environment, _: Option<Box<Expr>>|
                    { Expr::Object(vec![]) }; // TODO: return asm node
                    env.add_rt_msg(Msg::new(self.name.clone(), Rc::new(constructor), dtype.clone(), None));
                    env.add_rt_size(dtype.size);
                    println!("{:?}", env.rt_stack_type);
                }
                env.add_ct_msg(Msg::new(self.name.clone(), Rc::new(constructor), ct_dtype, None));
            },
            None => {
                // add runtime msg TODO: defer code to a function
                let constructor = move |_: Option<Box<Expr>>, _: &Environment, _: Option<Box<Expr>>|
                { Expr::Object(vec![]) }; // TODO: return asm node
                env.add_rt_msg(Msg::new(self.name.clone(), Rc::new(constructor), dtype.clone(), None));
                env.add_rt_size(dtype.size);
                println!("{:?}", env.rt_stack_type);
            },
        }

        Ok(dtype)
    }
}