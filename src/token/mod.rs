pub mod scanner;
pub mod literal;

use literal::Literal;

#[derive(Debug, Clone)]
pub struct Token {
    pub ttype: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub start: usize
}

impl Token {
    pub fn new(ttype: TokenType, lexeme: String, line: usize, start: usize) -> Self {
        Self {
            ttype,
            lexeme,
            line,
            start
        }
    }

    pub fn to_string(&self) -> String {
        match &self.ttype {
            TokenType::Identifier => format!("{:?} {} ln{}", self.ttype, self.lexeme, self.line),
            TokenType::Literal(lit) => format!("{:?} ln{}", lit, self.line),
            _ => format!("{:?} ln{}", self.ttype, self.line),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Reserved symbols
    Colon,
    Period,
    LeftBrace, RightBrace,
    LeftParen, RightParen,
    LeftSqBracket, RightSqBracket,
    Carrot,
    Pipe,
    Semicolon,
    // Reserved identifiers
    // Symbolic
    Equal,
    RightArrow,
    // Alphanumeric
    Underscore,
    Self_,
    Asm,

    // Literals
    Literal(Literal),
    // Index(u32),
    Identifier,
    // ScopedIdent,

    End
}