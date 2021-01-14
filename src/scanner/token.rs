#[derive(Debug, Clone)]
pub struct Token {
    ttype: TokenType,
    lexeme: String,
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

#[derive(Debug, Clone)]
pub enum TokenType {
    // Reserved symbols
    Colon,
    Hash,
    LeftBrace, RightBrace,
    LeftParen, RightParen,
    LeftSqBracket, RightSqBracket,

    // Reserved identifiers
    // Symbolic
    Equal,
    Pipe,
    // Alphanumeric
    Underscore,
    Self_,

    // Literals
    Literal(Literal),
    Identifier,
    ScopedIdent
}

#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Char(char),
    Integer(i32),
    Float(f32)
}