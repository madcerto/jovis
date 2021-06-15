use super::literal::Literal;
use super::expr::Expr;
use super::env::Environment;
use super::token::TokenType;

pub trait Interpreter {
    fn interpret(self, env: &mut Environment) -> Literal;
}

impl Interpreter for Expr {
    fn interpret(self, env: &mut Environment) -> Literal {
        match self {
            Expr::Unary(_, _) => todo!(),
            Expr::Binary(left, op, right) => match op.ttype {
                TokenType::Equal => {
                    let name  = match *left {
                        Expr::BinaryOpt(left, op, _) => match op.ttype {
                            TokenType::Colon => match *left {
                                Expr::Identifier(name) => name.lexeme,
                                _ => panic!("expected identifier")
                            },
                            _ => panic!("expected declaration")
                        },
                        _ => panic!("expected declaration")
                    };
                    let val = right.interpret(env);
                    env.assign(name, val.clone());
                    val
                },
                _ => panic!("unexpected binary operator")
            },
            Expr::ScopeRes(_, _) => todo!(),
            Expr::BinaryOpt(_, _, _) => todo!(),
            Expr::Object(exprs) => {
                let mut vals = vec![];
                for expr in exprs {
                    vals.push(expr.interpret(env));
                }
                Literal::Object(vals)
            },
            Expr::Call(_, _) => todo!(),
            Expr::CodeBlock(_, exprs) => {
                let mut last = Literal::Object(vec![]);
                for expr in exprs {
                    last = expr.interpret(env);
                }
                last
            },
            Expr::Identifier(name) => env.get(name.lexeme),
            Expr::Literal(inner) => inner,
        }
    }
}