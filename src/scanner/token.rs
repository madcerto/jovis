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
    Semicolon,
    Newline,
    Colon,
    Comma,
    Hash,
    Dot,
    Pipe,
    LeftBrace, RightBrace,
    LeftParen, RightParen,

    // Reserved identifiers
    Equal,
    SlashSlash,
    MinusGreater,
    At,
    Ampersand,
    DollarSign,

    // Literals
    Literal(Literal),
    identifier
}

#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Integer(i32),
    Float(f32)
}