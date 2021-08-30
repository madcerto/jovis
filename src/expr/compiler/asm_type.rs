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
#[derive(Clone, PartialEq)]
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
impl NASMRegSize {
    pub fn to_num(&self) -> usize {
        match self {
            NASMRegSize::L8 => 1,
            NASMRegSize::H8 => 1,
            NASMRegSize::L16 => 2,
            NASMRegSize::L32 => 4,
            NASMRegSize::L64 => 8,
        }
    }
    pub fn to_name(&self) -> &str {
        match self {
            NASMRegSize::L8 => "byte",
            NASMRegSize::H8 => "byte",
            NASMRegSize::L16 => "word",
            NASMRegSize::L32 => "dword",
            NASMRegSize::L64 => "qword",
        }
    }
}