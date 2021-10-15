use super::{Token, TokenType};
use super::literal::Literal;
use std::{collections::HashMap, str::FromStr};

pub struct Scanner {
    source: String,

    start: usize,
    current: usize,
    line: usize,

    sym_keywords: HashMap<String, TokenType>,
    an_keywords: HashMap<String, TokenType>
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            line: 1,
            sym_keywords: {
                let mut map = HashMap::new();
                map.insert("=".to_string(), TokenType::Equal);
                map.insert("->".to_string(), TokenType::RightArrow);
                map
            },
            an_keywords: {
                let mut map = HashMap::new();
                map.insert("_".to_string(), TokenType::Underscore);
                map.insert("Self".to_string(), TokenType::Self_);
                map.insert("asm".to_string(), TokenType::Asm);
                map
            }
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, (usize, String)> {
        let mut tokens = Vec::new();
        while !self.is_at_end() {
            match self.scan_token()? {
                Some(token) => tokens.push(token),
                None => {}
            }
        }
        tokens.push(self.new_token(TokenType::End));

        Ok(tokens)
    }
    pub fn scan_tokens_err_ignore(&mut self) -> (Vec<Token>, usize) {
        let mut tokens = Vec::new();
        while !self.is_at_end() {
            match self.scan_token() {
                Ok(token_opt) => match token_opt {
                    Some(token) => tokens.push(token),
                    None => {}
                },
                Err(_) => {},
            }
        }
        tokens.push(self.new_token(TokenType::End));

        (tokens, self.start)
    }

    fn scan_token(&mut self) -> Result<Option<Token>, (usize, String)> {
        // in jovis you might be able to remove the whole result type by passing the context
        // of the while loop's block to this function, and instead of returning none it just
        // returns from that context, basically acting like a `continue` statement.
        let mut c = self.advance();

        let t = match c {
            ' '|'\r'|'\t' => { None }, // Ignore whitespace
            ':' => Some(self.new_token(TokenType::Colon)),
            '.' => Some(self.new_token(TokenType::Period)),
            '{' => Some(self.new_token(TokenType::LeftBrace)),
            '}' => Some(self.new_token(TokenType::RightBrace)),
            '(' => Some(self.new_token(TokenType::LeftParen)),
            ')' => Some(self.new_token(TokenType::RightParen)),
            '[' => Some(self.new_token(TokenType::LeftSqBracket)),
            ']' => Some(self.new_token(TokenType::RightSqBracket)),
            '^' => Some(self.new_token(TokenType::Carrot)),
            '|' => Some(self.new_token(TokenType::Pipe)),
            ';' => Some(self.new_token(TokenType::Semicolon)),
            '\n' => { self.line+=1; None },
            '\'' => self.scan_char()?,
            '"' => self.scan_string()?,
            
            '-' => Some(if Self::is_digit(self.peak()) {
                self.advance();
                self.scan_number()
            } else { self.scan_sym_ident() }),

            '#' => {
                while !self.is_at_end() {
                    let c = self.advance();
                    if c == '\n' { self.line += 1; break }
                }
                None
            },

            _ => Some(if Self::is_digit(c) {
                self.scan_number()
            } else if c == '_' {
                loop {
                    c = self.advance();
                    if c != '_' { break }
                }
                if Self::is_alpha(c) {
                    self.scan_an_ident()
                } else {
                    self.scan_sym_ident()
                }
            } else if Self::is_alpha(c) {
                self.scan_an_ident()
            } else {
                self.scan_sym_ident()
            })
        };

        self.start = self.current;

        Ok(t)
    }

    fn advance(&mut self) -> char {
        self.current += 1;

        self.source.chars().nth(self.current - 1).unwrap()
    }
    fn peak(&self) -> char {
        if self.is_at_end() { '\0' }
        else { self.source.chars().nth(self.current).unwrap() }
    }
    fn peak_next(&self) -> char {
        if self.current + 1 >= self.source.len() { '\0' }
        else { self.source.chars().nth(self.current + 1).unwrap() }
    }

    fn new_token(&self, ttype: TokenType) -> Token {
        let text = self.source.get(self.start..self.current).unwrap().to_string();

        Token::new(ttype, text, self.line, self.start)
    }

    fn scan_char(&mut self) -> Result<Option<Token>, (usize, String)> {
        let next_char = self.peak();
        if next_char == '\n' || next_char == '\0' {
            Err((self.line, "Unterminated character literal".to_string()))
        } else {
            let val = self.advance();
            if self.peak() == '\'' { 
                self.advance();
                Ok(Some(self.new_token(TokenType::Literal(Literal::Char(val)))))
            } else { Err((self.line, "Unterminated or oversized character literal".to_string())) } // TODO: keep scanning after one character to see if it's oversized or unterminated
        }
    }
    fn scan_string(&mut self) -> Result<Option<Token>, (usize, String)> {
        let mut next_char = self.peak();
        while next_char != '"' {
            if next_char == '\0' {
                return Err((self.line, "Unterminated string".to_string()));
            } else {
                if next_char == '\n' { self.line+= 1; }
                self.advance();
                next_char = self.peak();
            }
        }

        let val = self.source.get((self.start + 1)..self.current).unwrap().to_string();
        self.advance();
        Ok(Some(self.new_token(TokenType::Literal(Literal::String(val)))))
    }
    fn scan_number(&mut self) -> Token {
        while Self::is_digit(self.peak()) { self.advance(); }
        if self.peak() == '.' && Self::is_digit(self.peak_next()) {
            self.advance();
            while Self::is_digit(self.peak()) { self.advance(); }

            let text = self.source.get(self.start..self.current).unwrap();
            self.new_token(TokenType::Literal(Literal::Float(f32::from_str(text).unwrap())))
        } else if self.peak() == 'b' {
            let src_copy = self.source.clone();
            let text = src_copy.get(self.start..self.current).unwrap();
            self.advance();
            self.new_token(TokenType::Literal(Literal::Byte(u8::from_str(text).unwrap())))
        } else {
            let text = self.source.get(self.start..self.current).unwrap();
            self.new_token(TokenType::Literal(Literal::Integer(i32::from_str(text).unwrap())))
        }
    }
    fn scan_an_ident(&mut self) -> Token {
        while Self::is_alpha_numeric(self.peak()) { self.advance(); }

        let text = self.source.get(self.start..self.current).unwrap().to_string();
        let ttype = self.an_keywords.get(&text).unwrap_or(&TokenType::Identifier).clone();
        self.new_token(ttype)
    }
    fn scan_sym_ident(&mut self) -> Token {
        while Self::is_sym(self.peak())
        { self.advance(); }

        let text = self.source.get(self.start..self.current).unwrap().to_string();

        let ttype = self.sym_keywords.get(&text).unwrap_or(&TokenType::Identifier).clone();
        self.new_token(ttype)
    }

    fn is_sym(c: char) -> bool {
        match c {
            ' '|'\r'|'\t'|':'|'#'|'.'|'{'|'}'|'('|')'|'['|']'|'\n'|'\''|'"'|'\0'
                => false,
            _ => {
                if (c < 'a' || c > 'z')
                && (c < 'A' || c > 'Z')
                && (c < '0' || c > '9') {
                    true
                } else {
                    false
                }
            }
        }
    }

    fn is_at_end(&self) -> bool {
        self.current>=self.source.len()
    }

    fn is_alpha(c: char) -> bool {
        (c>='a' && c<='z') || (c>='A' && c<='Z') || c == '_'
    }

    fn is_digit(c: char) -> bool {
        c>='0' && c<='9'
    }

    fn is_alpha_numeric(c: char) -> bool {
        Self::is_alpha(c) || Self::is_digit(c)
    }
}