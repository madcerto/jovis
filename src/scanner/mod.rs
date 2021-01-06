mod token;
use token::{IdentType, Literal, Token, TokenType};
use std::{collections::HashMap, str::FromStr};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,

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
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            sym_keywords: {
                let mut map = HashMap::new();
                map.insert("=".to_string(), TokenType::Equal);
                map.insert("@".to_string(), TokenType::Stat);
                map.insert("&".to_string(), TokenType::Mut);
                map
            },
            an_keywords: {
                let mut map = HashMap::new();
                map.insert("stat".to_string(), TokenType::Stat);
                map.insert("mut".to_string(), TokenType::Mut);
                map.insert("_".to_string(), TokenType::Underscore);
                map
            }
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, ()> {
        while !self.is_at_end() {
            self.scan_token()?;
            self.start = self.current;
        }

        Ok(self.tokens.clone())
    }

    fn scan_token(&mut self) -> Result<(), ()> {
        let c = self.advance();

        match c {
            ' '|'\r'|'\t' => { Ok(()) }, // Ignore whitespace
            ':' => { self.add_token(TokenType::Colon); Ok(()) },
            '#' => { self.add_token(TokenType::Hash); Ok(()) },
            '{' => { self.add_token(TokenType::LeftBrace); Ok(()) },
            '}' => { self.add_token(TokenType::RightBrace); Ok(()) },
            '(' => { self.add_token(TokenType::LeftParen); Ok(()) },
            ')' => { self.add_token(TokenType::RightParen); Ok(()) },
            '[' => { self.add_token(TokenType::LeftSqBracket); Ok(()) },
            ']' => { self.add_token(TokenType::RightSqBracket); Ok(()) },
            '\n' => { self.line+=1; Ok(()) },
            '\'' => { self.scan_char() },
            '"' => { self.scan_string() },
            '.' => {
                if Self::is_alpha_numeric(self.peak()) {
                    self.scan_an_ident(true);
                } else {
                    self.scan_sym_ident(true);
                }
                Ok(())
            }
            _ => {
                if Self::is_digit(c) {
                    self.scan_number();
                } else if Self::is_alpha(c) {
                    self.scan_an_ident(false);
                } else {
                    self.scan_sym_ident(false);
                }
                Ok(())
            }
        }
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

    fn add_token(&mut self, ttype: TokenType) {
        let text = self.source.get(self.start..self.current).unwrap().to_string();

        self.tokens.push(Token::new(ttype, text, self.line));
    }

    fn scan_char(&mut self) -> Result<(), ()> {
        let next_char = self.peak();
        if next_char == '\n' || next_char == '\0' {
            // TODO: add error message when error reporting is done
            Err(())
        } else {
            let val = self.advance();
            if self.peak() == '\'' {
                self.advance();
                self.add_token(TokenType::Literal(Literal::Char(val)));
                Ok(())
            } else { /*TODO*/ Err(()) }
        }
    }

    fn scan_string(&mut self) -> Result<(), ()> {
        let mut next_char = self.peak();
        while next_char != '"' {
            if next_char == '\0' {
                return Err(());
            } else {
                if next_char == '\n' { self.line+= 1; }
                self.advance();
                next_char = self.peak();
            }
        }

        let val = self.source.get((self.start + 1)..self.current).unwrap().to_string();
        self.advance();
        self.add_token(TokenType::Literal(Literal::String(val)));

        Ok(())
    }

    fn scan_number(&mut self) {
        let mut next_char = self.peak();
        while Self::is_digit(next_char) { self.advance(); next_char = self.peak(); }
        if next_char == '.' && Self::is_digit(self.peak_next()) {
            self.advance();
            while Self::is_digit(next_char) { self.advance(); }

            let val = self.source.get(self.start..self.current).unwrap();
            self.add_token(TokenType::Literal(Literal::Float(f32::from_str(val).unwrap())))
        } else {
            let val = self.source.get(self.start..self.current).unwrap();
            self.add_token(TokenType::Literal(Literal::Integer(i32::from_str(val).unwrap())))
        }
    }

    fn scan_an_ident(&mut self, scoped: bool) {
        let mut next_char = self.peak();
        while Self::is_alpha_numeric(next_char) { self.advance(); next_char = self.peak(); }

        let text = self.source.get(self.start..self.current).unwrap().to_string();
        let ttype = self.an_keywords.get(&text).unwrap_or({
            &if scoped { TokenType::ScopedIdent(IdentType::AlphaNumeric) }
            else { TokenType::Identifier(IdentType::AlphaNumeric) }
        }).clone();
        self.add_token(ttype);
    }

    fn scan_sym_ident(&mut self, scoped: bool) {
        let mut next_char = self.peak();
        while Self::is_sym(next_char) && next_char != '\0' {
            self.advance(); next_char = self.peak(); }

        let text = self.source.get(self.start..self.current).unwrap().to_string();
        let ttype = self.sym_keywords.get(&text).unwrap_or({
            &if scoped { TokenType::ScopedIdent(IdentType::Symbolic) }
            else { TokenType::Identifier(IdentType::Symbolic) }
        }).clone();
        self.add_token(ttype);
    }

    fn is_sym(c: char) -> bool {
        match c {
            ' '|'\r'|'\t' => false, // Ignore whitespace
            ':' => false,
            '#' => false,
            '.' => false,
            '{' => false,
            '}' => false,
            '(' => false,
            ')' => false,
            '[' => false,
            ']' => false,
            '\n' => false,
            '\'' => false,
            '"' => false,
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