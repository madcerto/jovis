use std::collections::HashMap;
use super::literal::Literal;

pub struct Environment {
    stack: HashMap<String, Literal>
}

impl Environment {
    pub fn new() -> Self {
        Self {
            stack: HashMap::new()
        }
    }

    pub fn assign(&mut self, name: String, val: Literal) {
        self.stack.insert(name, val);
    }
    pub fn get(&mut self, name: String) -> Literal {
        self.stack.get(&name).unwrap().clone()
    }
}