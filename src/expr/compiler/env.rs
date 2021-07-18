use super::{DType, dtype::Msg};

pub struct Environment {
    stack: Vec<Vec<u8>>,
    sp: usize,
    pub rt_stack_type: DType,
    pub ct_stack_type: DType
}

impl Environment {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(0),
            sp: 0,
            rt_stack_type: DType {size: 0, msgs: vec![]},
            ct_stack_type: DType {size: 0, msgs: vec![]}
        }
    }

    pub fn add_ct_msg(&mut self, msg: Msg) {
        self.ct_stack_type.msgs.push(msg);
    }
    pub fn get_stack(&self, addr: usize) {

    }







    // pub fn declare_data(&mut self, name: String, dtype: DType) {
    //     fn constructor(self_address: usize, env: Environment, arg: Option<Expr>) -> Expr {
    //         // TODO: generate an asm node
    //         Expr::Object(vec![])
    //     }
    //     self.rt_stack_type.msgs.push(Msg::new(name, constructor, dtype));
    // }
    // pub fn define_data(&mut self, name: String, dtype: DType, mut value: Vec<u8>) {
    //     self.stack.append(&mut value);

    //     fn ctime_constructor(self_address: usize, env: Environment, arg: Option<Expr>) -> Expr {
    //         let mut literals = vec![];
    //         let value = env.stack;
    //         for b in value {
    //             literals.push(Expr::Literal(Literal::Byte(b)));
    //         }
    //         Expr::Object(literals)
    //     }
    //     fn constructor(self_address: usize, env: Environment, arg: Option<Expr>) -> Expr {
    //         // TODO: generate an asm node
    //         Expr::Object(vec![])
    //     }
    //     self.rt_stack_type.msgs.push(Msg::new(name.clone(), constructor, dtype.clone()));
    //     self.ct_stack_type.msgs.push(Msg::new(name, ctime_constructor, dtype));
    // }
    // pub fn define(&mut self, _name: String, dtype: DType, _val: Object) -> Object {
    //     let obj = Object { dtype: dtype.clone(), address: self.sp };
    //     self.stack.resize(dtype.size, 0);
    //     self.sp += dtype.size;
    //     // add variable to msgs
    //     obj
    // }
    // pub fn _assign(&mut self, obj: Object, val: Vec<u8>) {
    //     if val.len() != obj.dtype.size { panic!("mismatched object and value types") }
    //     for i in obj.address..(obj.address + obj.dtype.size) {
    //         self.stack[i] = val[i-obj.address]
    //     }
    // }
    // pub fn _get_byte(&mut self, address: usize) -> u8 {
    //     self.stack[address]
    // }
}