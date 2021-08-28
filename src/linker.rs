#[allow(dead_code)]
extern "C" {
    pub fn j_link(entry_file: *const cty::c_char) -> cty::c_int;
}