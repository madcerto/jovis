mod token;
use token::{Token, TokenType};
use core::str::Chars;
use std::any::Any;

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,

    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&self) -> Result<Vec<Token>, ()> {
        while !self.is_at_end() {
            self.scan_token();
            self.start = self.current;
        }

        Ok(Vec::new())
    }

    fn scan_token(&self) {
        //
    }

    fn advance(&mut self) -> char {
        self.current += 1;

        self.source.chars().nth(self.current - 1).unwrap()
    }

    fn is_at_end(&self) -> bool {
        self.current>=self.source.len()
    }
}