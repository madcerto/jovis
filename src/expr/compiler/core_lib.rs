use std::rc::Rc;
use crate::token::literal::Literal;

use super::{DType, Environment, dtype::Msg, Expr};

pub const VOID: DType = DType {
    size: 0,
    msgs: vec![],
    size_unknown: false,
    msgs_unknown: false
};

// byte types
pub const B8: DType = DType {
    size: 1,
    msgs: vec![],
    size_unknown: false,
    msgs_unknown: false
};
// const B16: DType = DType {
//     size: 2,
//     msgs: vec![],
    // size_unknown: false,
    // msgs_unknown: false
// };
// const B32: DType = DType {
//     size: 4,
//     msgs: vec![],
    // size_unknown: false,
    // msgs_unknown: false
// };
// const B64: DType = DType {
//     size: 8,
//     msgs: vec![],
    // size_unknown: false,
    // msgs_unknown: false
// };

// "primitives"
pub const STRING: DType = DType {
    size: 16,
    msgs: vec![],
    size_unknown: false,
    msgs_unknown: false
};
pub fn str_from_jstr(bytes: [u8; 16], env: &mut Environment) -> Option<String> {
    let mut addr: [u8; 8] = [0; 8]; // TODO: find more efficient way to do this
    for i in 0..8 {
        addr[i] = bytes[i];
    }
    let mut size: [u8; 8] = [0; 8]; // TODO: find more efficient way to do this
    for i in 0..8 {
        size[i] = bytes[i+8];
    }
    let addr = usize::from_ne_bytes(addr);
    let size = usize::from_ne_bytes(size);
    let val_bytes = env.get_stack(addr);
    match val_bytes {
        Some(val_bytes) => {
            if val_bytes.len() != size { return None }
            let mut str = String::default();
            for byte in val_bytes {
                str.push(*byte as char);
            }
            Some(str)
        },
        None => None
    }
}
pub const CHAR: DType = DType {
    size: 1,
    msgs: vec![],
    size_unknown: false,
    msgs_unknown: false
};
pub const I32: DType = DType {
    size: 4,
    msgs: vec![],
    size_unknown: false,
    msgs_unknown: false
};
// pub const U32: DType = DType {
//     size: 4,
//     msgs: vec![],
    // size_unknown: false,
    // msgs_unknown: false
// };
// pub const U64: DType = DType {
//     size: 8,
//     msgs: vec![],
    // size_unknown: false,
    // msgs_unknown: false
// };
pub const F32: DType = DType {
    size: 4,
    msgs: vec![],
    size_unknown: false,
    msgs_unknown: false
};
// pub const BOOL: DType = DType {
//     size: 1,
//     msgs: vec![],
//     size_unknown: false,
//     msgs_unknown: false
// };
pub const TYPE: DType = DType {
    size: 6, // stores u32 size, which will temporarily be the only thing a udt represents, and bools for unknowns
    msgs: vec![],
    size_unknown: false,
    msgs_unknown: false
};
pub const FN: DType = DType {
    size: 8, // u64 of address
    msgs: vec![],
    size_unknown: false,
    msgs_unknown: false
};
pub const DECL: DType = DType {
    size: 22, // name as string then type
    msgs: vec![],
    size_unknown: false,
    msgs_unknown: false
};

pub fn export() -> DType {
    DType {
        size: 0,
        msgs: vec![
            {
                let mut byte_lits = vec![];
                for byte in I32.to_bytes() {
                    byte_lits.push(Expr::Literal(Literal::Byte(byte)));
                }
                let constructor = move |_: Option<Box<Expr>>, _: &Environment, _: Option<Box<Expr>>|
                { Expr::Object(byte_lits.clone()) };
                Msg::new("I32".into(), Rc::new(constructor), TYPE, None)
            }
        ],
        size_unknown: false,
        msgs_unknown: false
    }
}