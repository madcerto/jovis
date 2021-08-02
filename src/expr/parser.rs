use super::Expr;
use crate::token::{Token, TokenType};
use crate::pprint::PPrint;

pub struct Parser {
    source: Vec<Token>,
    next: usize
}

impl Parser {
    pub fn new(source: Vec<Token>) -> Self {
        Self {
            source,
            next: 0
        }
    }

    pub fn parse(&mut self) -> Expr {
        match self.expr() {
            Ok(expr) => expr,
            Err(err) => match err.tkn.ttype {
                TokenType::End => panic!("error at end: {}", err.msg),
                _ => panic!("error at token {}: {}", err.tkn.to_string(), err.msg)
            }
        }
    }

    fn expr(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.in_expr()?;
        loop { match self.peak().ttype {
            TokenType::Equal => expr = self.binary(expr, false)?,
            _ => break
        }}
        Ok(expr)
    }
    fn in_expr(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.literal()?;
        loop { match self.peak().ttype {
            TokenType::Semicolon => expr = self.binary_opt(expr)?,
            TokenType::RightArrow
            | TokenType::Carrot => expr = self.binary(expr, true)?,
            TokenType::Period => expr = self.msg_emission(expr)?,
            _ => break
        }}
        Ok(expr)
    }
    fn literal(&mut self) -> Result<Expr, ParseError> {
        let tkn = self.peak();
        match tkn.ttype {
            TokenType::Literal(inner) => { self.advance(); Ok(Expr::Literal(inner)) },
            TokenType::Identifier
            | TokenType::Underscore
            | TokenType::Self_ => {
                self.advance();
                let mut arg = None;

                if let TokenType::Colon = self.peak().ttype {
                    self.advance();
                    arg = Some(Box::new(self.in_expr()?));
                }

                Ok(Expr::MsgEmission(None, tkn, arg))
            },
            TokenType::LeftSqBracket => {
                self.advance();
                let mut exprs = Vec::new();
                while self.peak().ttype != TokenType::RightSqBracket {
                    if let TokenType::End = self.peak().ttype { return Err(ParseError{
                        tkn: self.peak(),
                        msg: "Unterminated object literal".to_string()
                    }) }
                    exprs.push(self.expr()?);
                }
                self.advance();
                Ok(Expr::Object(exprs))
            },
            TokenType::Pipe => {
                self.advance();
                let mut capture_list = Vec::new();
                while self.peak().ttype != TokenType::Pipe {
                    if let TokenType::End = self.peak().ttype { return Err(ParseError{
                        tkn: self.peak(),
                        msg: "Unterminated capture list".to_string()
                    })} else { capture_list.push(self.expr()?); }
                }
                self.advance();

                Ok(Expr::Fn(capture_list, Box::new(self.expr()?)))
            },
            TokenType::LeftBrace => {
                self.advance();
                let mut exprs = Vec::new();
                while self.peak().ttype != TokenType::RightBrace {
                    if let TokenType::End = self.peak().ttype { return Err(ParseError{
                        tkn: self.peak(),
                        msg: "Unterminated code block".to_string()
                    }) }
                    exprs.push(self.expr()?);
                }
                self.advance();
                Ok(Expr::CodeBlock(exprs))
            }
            TokenType::LeftParen => self.dtype(),
            _ => Err(ParseError{ tkn, msg: "Invalid expression-starting token".to_string() })
        }
    }
    fn dtype(&mut self) -> Result<Expr, ParseError> {
        self.advance();
        let mut exprs = Vec::new();
        while self.peak().ttype != TokenType::RightParen {
            if self.peak().ttype == TokenType::End { return Err(ParseError{
                tkn: self.peak(),
                msg: "Unterminated type definition".to_string()
            }) }
            exprs.push(self.expr()?);
        }
        self.advance();
        Ok(Expr::Type(exprs))
    }
    fn binary_opt(&mut self, left: Expr) -> Result<Expr, ParseError> {
        let op = self.advance();
        let right = match self.peak().ttype {
            TokenType::Literal(_)
            | TokenType::Identifier
            | TokenType::Underscore
            | TokenType::Self_
            | TokenType::LeftSqBracket
            | TokenType::Pipe
            | TokenType::LeftBrace
            | TokenType::LeftParen => Some(Box::new(self.in_expr()?)),
            // {
            //     let mut expr = self.literal()?;
            //     loop { match self.peak().ttype {
            //         TokenType::Semicolon => expr = self.binary_opt(expr)?,
            //         TokenType::RightArrow
            //         | TokenType::Carrot => expr = self.binary(expr)?,
            //         TokenType::Period => expr = self.msg_emission(expr)?,
            //         _ => break
            //     } println!("{}", expr.prettify());}
            //     expr.pprint();
            //     Some(Box::new(expr))
            // },
            _ => None
        };
        Ok(Expr::BinaryOpt(Box::new(left), op, right))
    }
    fn binary(&mut self, left: Expr, in_expr: bool) -> Result<Expr, ParseError> {
        self.advance();
        let expr;
        if in_expr { expr = Expr::Binary(Box::new(left), self.previous(), Box::new(self.in_expr()?)) }
        else { expr = Expr::Binary(Box::new(left), self.previous(), Box::new(self.expr()?)) }
        Ok(expr) // TODO: remove use of expr variable
    }
    fn msg_emission(&mut self, left: Expr) -> Result<Expr, ParseError> {
        self.advance();
        let msg_name = self.advance();
        let mut arg = None;

        if let TokenType::Colon = self.peak().ttype {
            self.advance();
            arg = Some(Box::new(self.in_expr()?));
        }
        Ok( Expr::MsgEmission(Some(Box::new(left)), msg_name, arg) )
    }

    fn is_at_end(&self) -> bool {
        match self.peak().ttype {
            TokenType::End => {}
            _ => {}
        }
        false
    }
    fn peak(&self) -> Token {
        self.source[self.next].clone()
    }
    fn previous(&self) -> Token {
        self.source[self.next - 1].clone()
    }
    fn advance(&mut self) -> Token {
        if !self.is_at_end() { self.next += 1; }
        self.previous()
    }
}

#[derive(Debug, Clone)]
pub struct ParseError {
    tkn: Token,
    msg: String
}