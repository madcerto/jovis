use super::expr::Expr;
use super::token::literal::Literal;

pub trait PPrint {
    fn prettify(&self) -> String;
    fn pprint(&self);
}

impl PPrint for Expr {
    fn prettify(&self) -> String {
        match self {
            Expr::Binary(left, op, right) =>
                format!("( {} {} {} )", op.lexeme, left.prettify(), right.prettify()),
            Expr::MsgEmission(self_expr, msg_name, arg_opt) => {
                let mut str;
                match self_expr {
                    Some(self_expr) => str = format!("{}.{}", self_expr.prettify(), msg_name.lexeme),
                    None => str = format!("{}", msg_name.lexeme),
                }
                if let Some(arg) = arg_opt {
                    str.push_str(format!(": {}", arg.prettify()).as_str())
                }
                str
            },
            Expr::BinaryOpt(left, op, right) => match right {
                Some(right) => format!("( {} {} {} )", op.lexeme, left.prettify(), right.prettify()),
                None => format!("( {} {} )", op.lexeme, left.prettify())
            },
            Expr::Asm(_, _) => todo!(),
            Expr::Object(exprs) => {
                let mut str = "[\n".to_string();
                for expr in exprs {
                    str.push_str(format!("{}\n", expr.prettify()).as_str())
                }
                str.push_str("]");
                str
            },
            Expr::CodeBlock(exprs) => {
                let mut str: String = "{\n".into();
                for expr in exprs {
                    str.push_str(format!("{}\n", expr.prettify()).as_str())
                }
                str.push_str("}");
                str
            },
            Expr::Fn(capture_list, expr) => {
                let mut str = "| ".to_string();
                for capture in capture_list {
                    str.push_str(format!("{} ", capture.prettify()).as_str())
                }
                str.push_str("| ");
                str.push_str(format!("{}", expr.prettify()).as_str());
                str
            },
            Expr::Type(exprs) => {
                let mut str = "t( ".to_string();
                for expr in exprs {
                    str.push_str(format!("{} ", expr.prettify()).as_str())
                }
                str.push_str(")");
                str
            },
            // Expr::Identifier(name) => format!("{}", name.lexeme),
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
            Literal::Byte(val) => format!("{}b", val),
        }
    }

    fn pprint(&self) {
        println!("{}", self.prettify());
    }
}