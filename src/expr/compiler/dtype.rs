use std::collections::HashMap;
use super::Environment;

#[derive(Clone, Debug)]
pub struct DType {
    pub size: usize,
    pub msgs: HashMap<String, fn(obj: Object, env: Environment, arg: Option<Object>) -> Object>
}

pub struct Object {
    pub dtype: DType,
    pub address: usize
}