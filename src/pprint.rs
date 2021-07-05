use super::expr::Expr;
use super::token::literal::Literal;

pub trait PPrint {
    fn prettify(&self) -> String;
    fn pprint(&self);
}

impl PPrint for Expr {
    fn prettify(&self) -> String {
        match self {
            Expr::Unary(operator, operand) =>
                format!("( {} {} )", operator.lexeme, operand.prettify()),
            Expr::Binary(left, op, right) =>
                format!("( {} {} {} )", op.lexeme, left.prettify(), right.prettify()),
            Expr::MsgEmission(left, right) =>
                format!("( {} {} {} )", ".", left.prettify(), right.prettify()),
            Expr::BinaryOpt(left, op, right) => match right {
                Some(right) => format!("( {} {} {} )", op.lexeme, left.prettify(), right.prettify()),
                None => format!("( {} {} )", op.lexeme, left.prettify())
            },
            Expr::Object(exprs) => {
                let mut str = "[ ".to_string();
                for expr in exprs {
                    str.push_str(format!("{} ", expr.prettify()).as_str())
                }
                str.push_str("]");
                str
            },
            Expr::Call(func, args) => {
                let mut str = format!("( {} ", func.prettify());
                for arg in args {
                    str.push_str(format!("{} ", arg.prettify()).as_str())
                }
                str.push_str(")");
                str
            },
            Expr::CodeBlock(capture_list, exprs) => {
                let mut str = "| ".to_string();
                for capture in capture_list {
                    str.push_str(format!("{} ", capture.prettify()).as_str())
                }
                str.push_str("| { ");
                for expr in exprs {
                    str.push_str(format!("{} ", expr.prettify()).as_str())
                }
                str.push_str("}");
                str
            },
            Expr::Identifier(name) => format!("{}", name.lexeme),
            Expr::Literal(inner) => format!("{}", inner.prettify()),
        }
    }

    fn pprint(&self) {
        println!("{}", self.prettify());
    }
}

impl PPrint for Literal {
    fn prettify(&self) -> String {
        match self {
            Literal::String(val) => format!("\"{}\"", val),
            Literal::Char(val) => format!("'{}'", val),
            Literal::Integer(val) => format!("{}", val),
            Literal::Float(val) => format!("{}", val),
            // Literal::Object(vals) => {
            //     let mut str = "[ ".to_string();
            //     for val in vals {
            //         str.push_str(format!("{} ", val.prettify()).as_str())
            //     }
            //     str.push_str("]");
            //     str
            // },
        }
    }

    fn pprint(&self) {
        println!("{}", self.prettify());
    }
}