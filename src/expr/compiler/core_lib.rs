use super::{DType, Environment};

pub const VOID: DType = DType {
    size: 0,
    msgs: vec![]
};

// byte types
pub const B8: DType = DType {
    size: 1,
    msgs: vec![]
};
// const B16: DType = DType {
//     size: 8,
//     msgs: vec![]
// };
// const B32: DType = DType {
//     size: 8,
//     msgs: vec![]
// };
// const B64: DType = DType {
//     size: 8,
//     msgs: vec![]
// };

// "primitives"
pub const STRING: DType = DType {
    size: 16,
    msgs: vec![]
};
pub fn str_from_jstr(bytes: [u8; 16], env: &mut Environment) -> Option<String> {
    let mut addr: [u8; 8] = [0; 8]; // TODO: find more efficient way to do this
    for i in 0..8 {
        addr[i] = bytes[i];
    }
    let mut size: [u8; 8] = [0; 8]; // TODO: find more efficient way to do this
    for i in 8..16 {
        size[i] = bytes[i];
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
    msgs: vec![]
};
pub const I32: DType = DType {
    size: 4,
    msgs: vec![]
};
pub const F32: DType = DType {
    size: 4,
    msgs: vec![]
};
pub const TYPE: DType = DType {
    size: 4,
    msgs: vec![]
};
pub const FN: DType = DType {
    size: 8,
    msgs: vec![]
};