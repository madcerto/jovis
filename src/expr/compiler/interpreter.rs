use super::{Expr, Environment, DType, Object, dtype::Msg};
use crate::token::{TokenType, literal::Literal};

pub trait Interpret {
    fn interpret(self, env: &mut Environment) -> Option<(Vec<u8>, DType)>;
    fn interpret_new_env(self) -> Option<(Vec<u8>, DType)>;
}

impl Interpret for Expr {
    fn interpret(self, env: &mut Environment) -> Option<(Vec<u8>, DType)> {
        match self {
            Expr::Unary(_, _) => todo!(),
            Expr::Binary(left, op, right) => match op.ttype {
                TokenType::Equal => {
                    let name  = match *left {
                        Expr::BinaryOpt(left, op, _) => match op.ttype {
                            TokenType::Colon => match *left {
                                Expr::MsgEmission(None, name) => name.lexeme,
                                _ => panic!("expected identifier")
                            },
                            _ => panic!("expected declaration")
                        },
                        _ => panic!("expected declaration")
                    };
                    let val = right.interpret(env);
                    // env.define(name, val.clone(), Object { dtype: DType { size: 0, msgs: HashMap::new() }, address: 0 });
                    env.define(name, DType { size: 0, msgs: vec![] }, Object { dtype: DType { size: 0, msgs: vec![] }, address: 0 });
                    val
                },
                _ => panic!("unexpected binary operator")
            },
            Expr::MsgEmission(_, _) => todo!(),
            Expr::BinaryOpt(_, _, _) => todo!(),
            Expr::Object(exprs) => {
                let mut bytes = vec![];
                let mut msgs = vec![];
                let mut size = 0;
                for expr in exprs {
                    match expr {
                        Expr::Binary(left, op, right) => if let TokenType::Equal = op.ttype {
                            let name  = match *left {
                                Expr::BinaryOpt(left, op, _) => match op.ttype {
                                    TokenType::Colon => match *left {
                                        Expr::MsgEmission(None, name) => name.lexeme,
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
                                    for byte in bytes.clone() { byte_lits.push(Expr::Literal(Literal::Byte(byte))) }
                                    fn constructor(_: usize,_:Environment,_:Option<Expr>) -> Expr { Expr::Object(vec![]) } // TODO
                                    msgs.push(Msg::new(name, constructor, dtype.clone() ));
                                    size += dtype.size;
                                },
                                None => return None,
                            }
                        }
                        _ => {}
                    }
                }
                Some((bytes,  DType { size, msgs }))
            },
            Expr::Call(_, _) => todo!(),
            Expr::CodeBlock(_, exprs) => {
                let mut last = vec![];
                let mut last_type = DType { size: 0, msgs: vec![] };
                for expr in exprs {
                    let interpreted = expr.interpret(env);
                    match interpreted {
                        Some((bytes, dtype)) => {
                            last = bytes;
                            last_type = dtype;
                        },
                        None => return None
                    }
                }
                Some((last, last_type))
            },
            // Expr::Identifier(name) => env.get(name.lexeme),
            Expr::Literal(inner) => match inner.clone() {
                Literal::String(val) => {
                    let mut bytes = vec![];
                    val.chars().for_each(|c|{ bytes.push(c as u8) });
                    Some((bytes, DType::from_literal(inner)))
                },
                Literal::Char(c) => Some((vec![c as u8], DType::from_literal(inner))),
                Literal::Integer(i) => {
                    let mut vec = vec![];
                    vec.extend_from_slice(&i.to_le_bytes());
                    Some((vec, DType::from_literal(inner)))
                },
                Literal::Float(f) => {
                    let mut vec = vec![];
                    vec.extend_from_slice(&f.to_le_bytes());
                    Some((vec, DType::from_literal(inner)))
                },
                Literal::Byte(b) => Some((vec![b], DType::from_literal(inner))),
            },
        }
    }

    fn interpret_new_env(self) -> Option<(Vec<u8>, DType)> {
        self.interpret(&mut Environment::new())
    }
}