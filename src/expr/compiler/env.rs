use super::{DType, Object};

pub struct Environment {
    stack: Vec<u8>,
    sp: usize,
    stack_type: DType
}

impl Environment {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(0),
            sp: 0,
            stack_type: DType {size: 0, msgs: vec![]}
        }
    }

    pub fn define(&mut self, _name: String, dtype: DType, _val: Object) -> Object {
        let obj = Object { dtype: dtype.clone(), address: self.sp };
        self.stack.resize(dtype.size, 0);
        self.sp += dtype.size;
        // add variable to msgs
        obj
    }
    pub fn _assign(&mut self, obj: Object, val: Vec<u8>) {
        if val.len() != obj.dtype.size { panic!("mismatched object and value types") }
        for i in obj.address..(obj.address + obj.dtype.size) {
            self.stack[i] = val[i-obj.address]
        }
    }
    pub fn _get_byte(&mut self, address: usize) -> u8 {
        self.stack[address]
    }
}