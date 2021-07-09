use crate::token::{Token, TokenType};
use super::{DType, Expr};

pub trait TypeCheck {
    fn check(&mut self) -> DType;
}

impl TypeCheck for Expr {
    fn check(&mut self) -> DType {
        match self {
            Expr::Unary(_, _) => todo!(),
            Expr::Binary(left, op, right) => match op.ttype {
                TokenType::Equal => {
                    let _name  = match *left.clone() {
                        Expr::BinaryOpt(left, Token { ttype: TokenType::Colon, lexeme:_,line:_ }, _) => match *left {
                            Expr::MsgEmission(None, name) => name.lexeme,
                            _ => panic!("expected identifier")
                        },
                        _ => panic!("expected declaration")
                    };
                    // TODO: add name to stack type's msgs
                    right.check()
                },
                _ => panic!("expected identifier")
            },
            Expr::MsgEmission(_, _) => todo!(),
            Expr::BinaryOpt(_, _, _) => todo!(),
            Expr::Object(exprs) => {
                let mut size = 0;
                for expr in exprs {
                    // if initialization, add name to object's msgs
                    size += expr.check().size;
                }
                DType { size, msgs: vec![] }
            },
            Expr::Call(_, _) => todo!(),
            Expr::CodeBlock(_, exprs) => {
                let mut last = &mut Expr::Object(vec![]);
                for expr in exprs {
                    last = expr;
                }
                last.check()
            },
            Expr::Literal(inner) => DType::from_literal(inner.clone()),
        }
    }
}