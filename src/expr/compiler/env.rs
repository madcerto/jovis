use super::{DType, dtype::Msg, core_lib};

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    stack: Vec<Vec<u8>>,
    sp: usize,
    rt_stack_type: DType,
    ct_stack_type: DType
}

impl Environment {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(0),
            sp: 0,
            rt_stack_type: core_lib::export(),
            ct_stack_type: core_lib::export()
        }
    }

    pub fn get_rt_stack_type(&self) -> DType {
        self.rt_stack_type.clone()
    }
    pub fn get_ct_stack_type(&self) -> DType {
        self.ct_stack_type.clone()
    }

    pub fn add_ct_msg(&mut self, msg: Msg) {
        self.ct_stack_type.msgs.push(msg);
    }
    pub fn add_rt_msg(&mut self, msg: Msg) {
        self.rt_stack_type.msgs.push(msg);
    }
    pub fn _add_ct_size(&mut self, size: u32) {
        self.ct_stack_type.size += size;
    }
    pub fn add_rt_size(&mut self, size: u32) {
        self.rt_stack_type.size += size;
    }

    pub fn push(&mut self, bytes: Vec<u8>) -> usize {
        self.stack.push(bytes);
        let tmp = self.sp;
        self.sp += 1;
        tmp
    }
    pub fn get_stack(&self, addr: usize) -> Option<&Vec<u8>> {
        self.stack.get(addr)
    }

    pub fn print_stacks(&self) {
        println!("ct: {:?}\nrt: {:?}", self.ct_stack_type, self.rt_stack_type);
    }
}