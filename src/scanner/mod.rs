mod token;
use token::{Literal, Token, TokenType};
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
                map.insert("|".to_string(), TokenType::Pipe);
                map
            },
            an_keywords: {
                let mut map = HashMap::new();
                map.insert("_".to_string(), TokenType::Underscore);
                map.insert("Self".to_string(), TokenType::Self_);
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
            self.start = self.current;
        }

        Ok(tokens)
    }

    fn scan_token(&mut self) -> Result<Option<Token>, (usize, String)> {
        let c = self.advance();

        Ok(match c {
            ' '|'\r'|'\t' => { None }, // Ignore whitespace
            ':' => Some(self.new_token(TokenType::Colon)),
            '#' => Some(self.new_token(TokenType::Hash)),
            '{' => Some(self.new_token(TokenType::LeftBrace)),
            '}' => Some(self.new_token(TokenType::RightBrace)),
            '(' => Some(self.new_token(TokenType::LeftParen)),
            ')' => Some(self.new_token(TokenType::RightParen)),
            '[' => Some(self.new_token(TokenType::LeftSqBracket)),
            ']' => Some(self.new_token(TokenType::RightSqBracket)),
            '\n' => { self.line+=1; None },
            '\'' => self.scan_char()?,
            '"' => self.scan_string()?,
            '.' => if Self::is_alpha_numeric(self.peak()) {
                Some(self.scan_an_ident(true))
            } else {
                self.scan_sym_ident(true)
            },
            _ => if Self::is_digit(c) {
                Some(self.scan_number())
            } else if Self::is_alpha(c) {
                Some(self.scan_an_ident(false))
            } else {
                self.scan_sym_ident(false)
            }
        })
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

    fn new_token(&mut self, ttype: TokenType) -> Token {
        let text = self.source.get(self.start..self.current).unwrap().to_string();

        Token::new(ttype, text, self.line)
    }

    fn scan_char(&mut self) -> Result<Option<Token>, (usize, String)> {
        let next_char = self.peak();
        if next_char == '\n' || next_char == '\0' {
            Err((self.line, "Unterminated character literal".to_string()))
        } else {
            let val = self.advance();
            if self.peak() == '\'' { // Handle character literals that are too long
                self.advance();
                Ok(Some(self.new_token(TokenType::Literal(Literal::Char(val)))))
            } else { Err((self.line, "Unterminated or oversized character literal".to_string())) }
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
        let mut next_char = self.peak();
        while Self::is_digit(next_char) { self.advance(); next_char = self.peak(); }
        if next_char == '.' && Self::is_digit(self.peak_next()) {
            self.advance();
            next_char = self.peak();
            while Self::is_digit(next_char) { self.advance(); next_char = self.peak(); }

            let val = self.source.get(self.start..self.current).unwrap();
            self.new_token(TokenType::Literal(Literal::Float(f32::from_str(val).unwrap())))
        } else {
            let val = self.source.get(self.start..self.current).unwrap();
            self.new_token(TokenType::Literal(Literal::Integer(i32::from_str(val).unwrap())))
        }
    }

    fn scan_an_ident(&mut self, scoped: bool) -> Token {
        let mut next_char = self.peak();
        while Self::is_alpha_numeric(next_char) { self.advance(); next_char = self.peak(); }

        let text = self.source.get(self.start..self.current).unwrap().to_string();
        let ttype = self.an_keywords.get(&text).unwrap_or({
            &if scoped { TokenType::ScopedIdent }
            else { TokenType::Identifier }
        }).clone();
        self.new_token(ttype)
    }

    fn scan_sym_ident(&mut self, scoped: bool) -> Option<Token> {
        let mut next_char = self.peak();
        while Self::is_sym(next_char)
        { self.advance(); next_char = self.peak(); }

        let text = self.source.get(self.start..self.current).unwrap().to_string();

        if text == "//" {
            while next_char != '\n' && next_char != '\0' { self.advance(); next_char = self.peak(); }
            None
        } else {
            let ttype = self.sym_keywords.get(&text).unwrap_or({
                &if scoped { TokenType::ScopedIdent }
                else { TokenType::Identifier }
            }).clone();
            Some(self.new_token(ttype))
        }
    }

    fn is_sym(c: char) -> bool {
        match c {
            ' '|'\r'|'\t'|':'|'#'|'.'|'{'|'}'|'('|')'|'['|']'|'\n'|'\''|'"'|'\0'
                => false,
            _ => {
                if Self::is_alpha_numeric(c) {
                    false
                } else {
                    true
                }
            }
        }
    }
}