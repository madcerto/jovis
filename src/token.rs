use super::literal::Literal;

#[derive(Debug, Clone)]
pub struct Token {
    pub ttype: TokenType,
    pub lexeme: String,
    line: usize
}

impl Token {
    pub fn new(ttype: TokenType, lexeme: String, line: usize) -> Self {
        Self {
            ttype,
            lexeme,
            line
        }
    }

    pub fn to_string(&self) -> String {
        format!("{:?} {}", self.ttype, self.lexeme)
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
    T,

    // Literals
    Literal(Literal),
    // Index(u32),
    Identifier,
    // ScopedIdent,

    End
}