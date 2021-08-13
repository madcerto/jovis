pub enum AsmTarget {
    X86Unix
}
#[derive(Clone)]
pub enum AsmLanguage {
    NASM
}

#[derive(PartialEq, Clone)]
pub enum NASMRegBase {
    A,
    B,
    C,
    D
}
pub enum NASMRegSize {
    L8,
    H8,
    L16,
    L32,
    L64
}
impl NASMRegBase {
    pub fn to_str(&self, size: NASMRegSize) -> String {
        let base = match self {
            Self::A => "a",
            Self::B => "b",
            Self::C => "c",
            Self::D => "d",
        };
        match size {
            NASMRegSize::L8 => format!("{}l", base),
            NASMRegSize::H8 => format!("{}h", base),
            NASMRegSize::L16 => format!("{}x", base),
            NASMRegSize::L32 => format!("e{}x", base),
            NASMRegSize::L64 => format!("r{}x", base),
        }
    }
}