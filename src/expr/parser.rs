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
        let mut expr = self.literal()?;
        loop { match self.peak().ttype {
            TokenType::LeftParen => expr = self.call(expr)?,
            TokenType::Semicolon
            | TokenType::Colon => expr = self.binary_opt(expr)?,
            TokenType::Equal
            | TokenType::RightArrow
            | TokenType::Carrot => expr = self.binary(expr)?,
            TokenType::Identifier => expr = self.msg_emission(expr)?,
            _ => break
        }}
        expr.pprint();
        Ok(expr)
    }
    fn literal(&mut self) -> Result<Expr, ParseError> {
        let tkn = self.peak();
        let expr = match tkn.ttype {
            TokenType::Literal(inner) => { self.advance(); Ok(Expr::Literal(inner)) },
            TokenType::Identifier
            | TokenType::Underscore
            | TokenType::Self_ => { self.advance(); Ok(Expr::MsgEmission(None, tkn)) },
            TokenType::T => { self.advance(); Ok(Expr::Unary(tkn, Box::new(self.expr()?))) }
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

                if let TokenType::LeftBrace = self.peak().ttype {
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
                    Ok(Expr::CodeBlock(capture_list, exprs))
                } else { Err(ParseError{
                    tkn: self.peak(),
                    msg: "Expected block after capture list".to_string()
                }) }
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
                Ok(Expr::CodeBlock(vec![], exprs))
            }
            _ => Err(ParseError{ tkn, msg: "Invalid expression-starting token".to_string() })
        };
        expr.clone()?.pprint();
        expr
    }
    fn call(&mut self, left: Expr) -> Result<Expr, ParseError> {
        self.advance();
        let mut exprs = Vec::new();
        while self.peak().ttype != TokenType::RightParen {
            if let TokenType::End = self.peak().ttype { return Err(ParseError{
                tkn: self.peak(),
                msg: "Unterminated argument list".to_string()
            }) }
            exprs.push(self.expr()?);
        }
        self.advance();
        let expr  = Expr::Call(Box::new(left), exprs);
        expr.pprint();
        Ok(expr)
    }
    fn binary_opt(&mut self, left: Expr) -> Result<Expr, ParseError> {
        self.advance();
        let right = match self.peak().ttype {
            TokenType::Literal(_)
            | TokenType::Identifier
            | TokenType::T
            | TokenType::LeftSqBracket
            | TokenType::Pipe
            | TokenType::LeftBrace => Some(Box::new(self.expr()?)),
            _ => None
        };
        let expr = Expr::BinaryOpt(Box::new(left), self.previous(), right);
        expr.pprint();
        Ok(expr)
    }
    fn binary(&mut self, left: Expr) -> Result<Expr, ParseError> {
        self.advance();
        let expr = Expr::Binary(Box::new(left), self.previous(), Box::new(self.expr()?));
        expr.pprint();
        Ok(expr)
    }
    fn msg_emission(&mut self, left: Expr) -> Result<Expr, ParseError> {
        Ok( Expr::MsgEmission(Some(Box::new(left)), self.advance()) )
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