use std::{collections::HashMap, ffi::CString, io::Write};
use crate::{expr::parser::Parser, linker::j_link, token::{literal::Literal, scanner::Scanner}};
use super::{Expr, Environment};

pub mod asm_type;
use asm_type::{AsmLanguage, AsmTarget, NASMRegSize, NASMRegBase};

pub struct CodeGenerator {
    code_vec: Vec<Code>,
    cur_code: Code,
    available_regs: Vec<NASMRegBase>
}
#[derive(Clone)]
struct Code {
    asm: Vec<u8>,
    lang: AsmLanguage
}
impl Code {
    fn new(lang: AsmLanguage) -> Self { Self { asm: vec![], lang } }
}

impl CodeGenerator {
    pub fn new(lang: AsmLanguage) -> Self {
        Self {
            code_vec: vec![],
            cur_code: Code::new(lang),
            available_regs: vec![
                NASMRegBase::A,
                NASMRegBase::B,
                NASMRegBase::C,
                NASMRegBase::D,
            ]
        }
    }

    pub fn generate_ir(mut self, ast: Expr, out_path: String, target: AsmTarget, env: &mut Environment) {
        // generate code
        self.generate_code(ast, target, env);
        // temp: print out generated assembly
        for (i, code) in self.code_vec.iter().enumerate() {
            let mut code_str = String::new();
            for b in code.clone().asm {
                let c = b as char;
                code_str.push(c);
            }
            println!("f{}:\n{}", i, code_str);
        }
        // TODO: Generate IR
        // TODO: Write IR to file
        // temp: manually write ir file
        self.manual_ir_write(&out_path);
        // call linker on IR file
        unsafe {
            let cstr = CString::new(out_path).unwrap();
            j_link(cstr.as_ptr());
        }
    }
    
    pub fn generate_code(&mut self, ast: Expr, _target: AsmTarget, env: &mut Environment) {
        // turn ast into assembly
        self.gen_nasm(ast, env, Some(&NASMRegBase::A));
    }

    pub fn gen_nasm(&mut self, ast: Expr, env: &mut Environment, reg_opt: Option<&NASMRegBase>) -> Option<NASMRegSize> { // TODO
        match ast {
            Expr::Binary(_, _, right) => {
                let register = match reg_opt {
                    Some(register) => register.clone(),
                    None => self.available_regs[0].clone(),
                };
                let is_ptr = self.gen_nasm(*right, env, Some(&register));
                if let Some(size) = is_ptr.clone() {
                    let mut code;
                    if size != NASMRegSize::L64 {
                        let mut code_str = format!("sub rsp, {}\n", size.to_num());
                        code_str.push_str(format!("mov {} [rsp], {}\n", size.to_name(), register.to_str(size.clone())).as_str());

                        code = code_str.as_bytes().to_vec();
                    } else {
                        code = format!("push {}\n", register.to_str(NASMRegSize::L64)).as_bytes().to_vec();
                    }
                    self.cur_code.asm.append(&mut code);
                }
                is_ptr
            },
            Expr::MsgEmission(_, name, _) => panic!("unexpected msg emission in checked ast: {}", name.lexeme),
            Expr::BinaryOpt(_, _, _) => todo!(), // leave as todo for a while because mostly unnecessary
            Expr::Asm(_asm_type, _ret_type, text_expr) => { // 2 future TODO
                // TODO: handle different assembly types
                let mut text = match *text_expr {
                    Expr::Literal(Literal::String(string)) => string,
                    _ => panic!("checked asm node does not have string literal as text")
                };

                // handle return expressions
                let mut is_ptr = Some(NASMRegSize::L64);
                for (i,_) in text.clone().match_indices("jret#") {
                    let ret_text = text.get((i+5)..).expect("value of i+5 seems to be too large");
                    let (mut n, is_ptr_in) = if let Some((i, _)) = ret_text.match_indices("addr(").next() {
                        (i+5, None)
                    } else if let Some((i, _)) = ret_text.match_indices("val(").next() {
                        (i+4, Some(NASMRegSize::L64))
                    } else { panic!("expected 'addr' or 'val'") };

                    let mut operand = "".to_string();
                    loop {
                        let c = match ret_text.chars().nth(n) {
                            Some(c) => c,
                            None => panic!("unterminated jret expression in asm"), // TODO: should be error for user
                        };
                        if c == ')' { break };
                        operand.push(c);
                        n += 1;
                    }
                    // generate code for return
                    let asm_ret_text;
                    match &reg_opt {
                        Some(register) => {
                            asm_ret_text = format!("mov {}, {}\n", register.to_str(NASMRegSize::L64), operand.trim());
                        },
                        None => asm_ret_text = "".to_string(),
                    }
                    // remove return from text
                    text.replace_range(i..(i+5+n+1), asm_ret_text.as_str());
                    // set is_ptr
                    is_ptr = is_ptr_in;
                }

                // TODO: we're forced to run the scanner and parser twice, which might not be an
                // issue since expressions are usually pretty small, but still would probably be best
                // to have some sort of caching for this, maybe just give in and make a new data structure.
                // handle embedded jovis expressions
                for (i,_) in text.clone().match_indices("j#") {
                    let mut scanner  = Scanner::new(text.get((i+2)..).unwrap().to_string());
                    let (tokens, n) = scanner.scan_tokens_err_ignore();
                    let mut parser = Parser::new(tokens);
                    let expr = parser.parse();

                    // get last semicolon before jovis expression, and push everything before that into cur_code
                    let mut line_start = i;
                    loop {
                        if line_start == 0
                        || text.chars().nth(line_start).unwrap() == '\n'
                        { break }
                        line_start -= 1;
                    }
                    let preceeding_text = text.get(0..line_start).unwrap().to_string();
                    text.replace_range(0..line_start, "");
                    self.cur_code.asm.append(&mut preceeding_text.as_bytes().to_vec());
                    // generate code from checked expr
                    let register = self.get_available_reg(reg_opt);
                    self.gen_nasm(expr, env, Some(&register));
                    // replace expression in text with register
                    text.replace_range((i-line_start)..(i-line_start+n), register.to_str(NASMRegSize::L64).as_str());
                    self.available_regs.push(register);
                }

                // handle registers
                let mut regs: HashMap<Option<String>, NASMRegBase> = HashMap::new();
                let mut offset = 0;
                for (mut i, _) in text.clone().match_indices("jreg") {
                    i -= offset;
                    let reg_size = match text.get((i+4)..(i+6)).unwrap() {
                        "1#" => NASMRegSize::L8,
                        "2#" => NASMRegSize::L16,
                        "4#" => NASMRegSize::L32,
                        "8#" => NASMRegSize::L64,
                        c => panic!("{}", c)
                    };
                    let mut chars = text.chars();
                    if chars.nth(i+6).map(|c| c.is_whitespace() || c == ',') != Some(true) {
                        let mut j = 6;
                        while chars.next().map(|c| c.is_whitespace()) != Some(true)
                            { j += 1 }
                        let alias = text.get((i+6)..(i+j)).unwrap().to_owned(); // TODO: err handling
                        let reg_base = if let Some(reg) = regs.get(&Some(alias.clone())) { reg.clone() }
                        else {
                            let reg_base = self.pop_available_reg(reg_opt);
                            regs.insert(Some(alias), reg_base.clone());
                            reg_base
                        };
                        let reg_str = reg_base.to_str(reg_size);
                        offset += j-reg_str.len();
                        text.replace_range(i..(i+j), reg_str.as_str());
                    } else {
                        let reg_base = self.pop_available_reg(reg_opt);
                        regs.insert(None, reg_base.clone());
                        let reg_str =reg_base.to_str(reg_size);
                        offset += 6-reg_str.len();
                        text.replace_range(i..(i+6), reg_str.as_str());
                    }
                }
                for (_, reg) in regs
                    { self.available_regs.push(reg) }
                
                // replace semicolons with newlines
                text = text.replace(';', "\n");

                // add text to current code object
                self.cur_code.asm.append(&mut text.as_bytes().to_vec());
                return is_ptr
            },
            Expr::Object(exprs) => if let Some(register) = reg_opt { // 1 future TODO
                // TODO: put values together, handle endianness, and push them on together
                let val_reg = self.pop_available_reg(reg_opt);
                self.available_regs.retain(|x| x != register );
                let mut addr_save = format!("mov {}, rsp\n", register.to_str(NASMRegSize::L64)).as_bytes().to_vec();
                self.cur_code.asm.append(&mut addr_save);
                for expr in exprs {
                    if let Some(size) = self.gen_nasm(expr.clone(), env, Some(&val_reg)) {
                        if let Expr::Binary(_,_,_) = expr {/* do nothing */}
                        else if size != NASMRegSize::L64 {
                            let mut code_str = format!("sub rsp, {}\n", size.to_num());
                            code_str.push_str(format!("mov {} [rsp], {}\n", size.to_name(), val_reg.to_str(size.clone())).as_str());
    
                            self.cur_code.asm.append(&mut code_str.as_bytes().to_vec());
                        } else {
                            let code_str = format!("push {}\n", val_reg.to_str(NASMRegSize::L64));
                            self.cur_code.asm.append(&mut code_str.as_bytes().to_vec());
                        }
                    }
                }
                self.available_regs.push(val_reg);
                self.available_regs.push(register.clone());
                None
            } else { Some(NASMRegSize::L64) },
            Expr::Fn(_capture_list, expr) => { // TODO capture list, 1 future TODO
                let mut prev_code = self.cur_code.clone();
                let prev_regs = self.available_regs.clone();
                self.available_regs = vec![
                    NASMRegBase::A,
                    NASMRegBase::B,
                    NASMRegBase::C,
                    NASMRegBase::D,
                ];
                
                self.cur_code = Code::new(self.cur_code.lang.clone());
                // add initialization code
                // TODO: make relative to target (64-bit or 32-bit system). currently assumes 64-bit
                let mut init = "push rbp\n mov rbp, rsp\n".as_bytes().to_vec();
                self.cur_code.asm.append(&mut init);
                // TODO: do something with capture list idk

                self.gen_nasm(*expr, env, reg_opt.clone());
                // add cleanup code
                let mut deinit = "mov rsp, rbp\n pop rbp\n ret\n".as_bytes().to_vec();
                self.cur_code.asm.append(&mut deinit);

                if let Some(register) = reg_opt {
                    let mut func_return = format!("mov {}, f{}\n", register.to_str(NASMRegSize::L64), self.code_vec.len()).as_bytes().to_vec();
                    prev_code.asm.append(&mut func_return);
                }
                self.code_vec.push(self.cur_code.clone());

                self.cur_code = prev_code;
                self.available_regs = prev_regs;
                Some(NASMRegSize::L64)
            },
            Expr::CodeBlock(mut exprs) => { // TODO waiting on type checker for new stack frame
                // TODO: once type checker has nested environments, store stack frame here
                match exprs.pop() {
                    Some(last_expr) => {
                        for expr in exprs.clone().into_iter() {
                            self.gen_nasm(expr, env, None);
                        }
                        // TODO: restore stack frame
                        self.gen_nasm(last_expr, env, reg_opt)
                    },
                    None => Some(NASMRegSize::L64),
                }
            },
            Expr::Type(_) => None, // leave as None for a while because mostly unnecessary
            Expr::Literal(lit) => if let Some(register) = reg_opt { // TODO make target specific
                match lit {
                    Literal::String(s) => {
                        // TODO: make stack register target-specific
                        let mut text = String::new();
                        // in the future store the characters in .rodata

                        // store stack pointer
                        let char_ptr_reg = self.get_available_reg(Some(register));
                        text.push_str(format!("mov {}, rsp\n", char_ptr_reg.to_str(NASMRegSize::L64)).as_str());
                        // push each character onto the stack
                        // TODO: make chunk size dependant on target
                        let mut chunks = s.as_bytes().chunks(8);
                        let last_chunk_opt = chunks.nth_back(0);
                        // TODO: flip bytes based on endianness
                        for qword in chunks {
                            text.push_str("push 0x");
                            for c in qword {
                                text.push_str(format!("{:X}", c).as_str());
                            }
                            text.push('\n');
                        }
                        if let Some(last_chunk) = last_chunk_opt {
                            let len = last_chunk.len();
                            let mut offset = 0;
                            if len != 8 {
                                text.push_str(format!("sub rsp, {}\n", len).as_str());
                                if len % 4 < len {
                                    offset += 4;
                                    text.push_str(format!("mov dword [{}-{}], 0x", char_ptr_reg.to_str(NASMRegSize::L64), 4).as_str());
                                    text.push_str(format!("{:X}{:X}{:X}{:X}\n", last_chunk[0],last_chunk[1],last_chunk[2],last_chunk[3]).as_str());
                                }
                                if len % 2 < len {
                                    text.push_str(format!("mov word [{}-{}], 0x", char_ptr_reg.to_str(NASMRegSize::L64), (offset+2)).as_str());
                                    text.push_str(format!("{:X}{:X}\n", last_chunk[offset],last_chunk[offset + 1]).as_str());
                                    offset += 2;
                                }
                                if offset != len {
                                    text.push_str(format!("mov byte [{}-{}], 0x", char_ptr_reg.to_str(NASMRegSize::L64), (offset+1)).as_str());
                                    text.push_str(format!("{:X}\n", last_chunk[offset]).as_str());
                                }
                            } else {
                                text.push_str("push 0x");
                                for c in last_chunk {
                                    text.push_str(format!("{:X}", c).as_str());
                                }
                                text.push('\n');
                            }
                        }
                        // store stack pointer in return register
                        text.push_str(format!("mov {}, rsp\n", register.to_str(NASMRegSize::L64)).as_str());
                        // push first stored pointer onto stack
                        text.push_str(format!("push {}\n", char_ptr_reg.to_str(NASMRegSize::L64)).as_str());
                        self.available_regs.push(char_ptr_reg);
                        // push length onto stack
                        text.push_str(format!("push {}\n", s.len()).as_str());

                        self.cur_code.asm.append(&mut text.as_bytes().to_vec());
                        None
                    },
                    Literal::Char(c) => {
                        let mut code = format!("mov {}, 0x{:X}\n", register.to_str(NASMRegSize::L8), c as u8).as_bytes().to_vec();
                        self.cur_code.asm.append(&mut code);
                        Some(NASMRegSize::L8)
                    },
                    Literal::Integer(i) => {
                        let mut code = format!("mov {}, 0x{:X}\n", register.to_str(NASMRegSize::L32), i as u32).as_bytes().to_vec();
                        self.cur_code.asm.append(&mut code);
                        Some(NASMRegSize::L32)
                    },
                    Literal::Float(f) => {
                        let mut code = format!("mov {}, 0x{:X}\n", register.to_str(NASMRegSize::L32), f as u32).as_bytes().to_vec();
                        self.cur_code.asm.append(&mut code);
                        Some(NASMRegSize::L32)
                    },
                    Literal::Byte(b) => {
                        let mut code = format!("mov {}, 0x{:X}\n", register.to_str(NASMRegSize::L8), b).as_bytes().to_vec();
                        self.cur_code.asm.append(&mut code);
                        Some(NASMRegSize::L8)
                    },
                }
            } else { Some(NASMRegSize::L64) },
        }
    }

    fn get_available_reg(&mut self, ret_reg: Option<&NASMRegBase>) -> NASMRegBase {
        match ret_reg {
            Some(register) => if &self.available_regs[0] == register
            { self.available_regs.remove(1) }
            else { self.available_regs.remove(0) },
            None => { self.available_regs.remove(0) },
        }
    }
    fn pop_available_reg(&mut self, ret_reg: Option<&NASMRegBase>) -> NASMRegBase {
        let reg = self.get_available_reg(ret_reg);
        self.available_regs.retain(|x| x != &reg );
        reg
    }

    fn manual_ir_write(self, out_path: &String) {
        let mut buf: Vec<u8> = vec![];
        // write header
        let fn_no = self.code_vec.len();
        let addr_size: u8;
        #[cfg(target_pointer_width = "64")]
        { addr_size = 8; }
        #[cfg(target_pointer_width = "32")]
        { addr_size = 4; }
        buf.extend_from_slice(&(0 as usize).to_ne_bytes()); // data ptr
        buf.extend_from_slice(&(0 as usize).to_ne_bytes()); // data size
        buf.extend_from_slice(&((6*addr_size as usize) as usize).to_ne_bytes()); // code ptr
        buf.extend_from_slice(&(fn_no as usize).to_ne_bytes()); // fn no
        buf.extend_from_slice(&(0 as usize).to_ne_bytes()); // dep ptr
        buf.extend_from_slice(&(0 as usize).to_ne_bytes()); // dep no

        // writen fns
        for code in self.code_vec {
            let size = code.asm.len();
            let mut text = code.asm;
            buf.extend_from_slice(&size.to_ne_bytes());
            buf.append(&mut text);
        }
        let mut file = std::fs::File::create(out_path).unwrap();
        file.write_all(&mut buf).unwrap();
    }
}