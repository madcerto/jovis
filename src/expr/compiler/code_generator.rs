use std::ffi::CString;
use crate::{expr::parser::Parser, linker::j_link, token::{literal::Literal, scanner::Scanner}};
use super::{Expr, asm_type::{AsmLanguage, AsmTarget, NASMRegSize, NASMRegBase}, Environment};

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

    pub fn generate_code(&mut self, ast: Expr, _out_file: String, _target: AsmTarget, env: &mut Environment) {
        // turn ast into assembly
        self.gen_nasm(ast, env, Some(NASMRegBase::A));
        // temp: print out generated assembly
        self.end_cur_code();
        for (i, code) in self.code_vec.iter().enumerate() {
            let mut code_str = String::new();
            for b in code.clone().asm {
                let mut c = b as char;
                if c == '\n' { c = '\n' }
                code_str.push(c);
            }
            println!("f{}:\n{}", i, code_str);
        }
        // TODO: Generate IR
        // TODO: Write IR to file
        // call linker on IR file
        // unsafe {
        //     j_link(CString::new(out_file).unwrap().as_ptr());
        // }
    }

    pub fn gen_nasm(&mut self, ast: Expr, env: &mut Environment, reg_opt: Option<NASMRegBase>) -> Option<NASMRegSize> { // TODO
        match ast {
            Expr::Binary(_, _, right) => {
                let register = match reg_opt {
                    Some(register) => register,
                    None => self.available_regs[0].clone(),
                };
                let is_ptr = self.gen_nasm(*right, env, Some(register.clone()));
                if let Some(size) = is_ptr.clone() {
                    let mut code;
                    if size != NASMRegSize::L64 {
                        let mut code_str = format!("sub rsp, {}\n", size.to_num());
                        code_str.push_str(format!("mov [rsp], {} {}\n", size.to_name(), register.to_str(size.clone())).as_str());

                        code = code_str.as_bytes().to_vec();
                    } else {
                        code = format!("push {}\n", register.to_str(NASMRegSize::L64)).as_bytes().to_vec();
                    }
                    self.cur_code.asm.append(&mut code);
                }
                is_ptr
            },
            Expr::MsgEmission(_, _, _) => panic!("unexpected msg emission in checked ast"),
            Expr::BinaryOpt(_, _, _) => todo!(), // leave as todo for a while because mostly unnecessary
            Expr::Asm(_asm_type, text_expr) => { // 2 future TODO
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
                        (i+3, Some(NASMRegSize::L64))
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
                    let register = self.get_available_reg(&reg_opt);
                    self.gen_nasm(expr, env, Some(register.clone()));
                    // replace expression in text with register
                    text.replace_range((i-line_start)..(i-line_start+n), register.to_str(NASMRegSize::L64).as_str());
                    self.available_regs.push(register);
                }

                // add text to current code object
                self.cur_code.asm.append(&mut text.as_bytes().to_vec());
                return is_ptr
            },
            Expr::Object(exprs) => if let Some(register) = &reg_opt {
                let val_reg = self.pop_available_reg(&reg_opt);
                self.available_regs.retain(|x| x != register );
                let mut addr_save = format!("mov {}, rsp\n", register.to_str(NASMRegSize::L64)).as_bytes().to_vec();
                self.cur_code.asm.append(&mut addr_save);
                for expr in exprs {
                    if let Some(size) = self.gen_nasm(expr.clone(), env, Some(val_reg.clone())) {
                        if let Expr::Binary(_,_,_) = expr {/* do nothing */}
                        // TODO: if size less than address size, do something different
                        else {
                            let mut code = format!("push {}\n", val_reg.to_str(NASMRegSize::L64)).as_bytes().to_vec();
                            self.cur_code.asm.append(&mut code);
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
            Expr::CodeBlock(mut exprs) => { // TODO new stack frame
                // TODO: once type checker has nested environments, store stack frame
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
            Expr::Type(_) => None, // leave as todo for a while because mostly unnecessary
            Expr::Literal(lit) => if let Some(register) = reg_opt { // TODO make target specific
                match lit {
                    Literal::String(s) => {
                        // TODO: make stack register target-specific
                        let mut text = String::new();
                        // in the future store the characters in .rodata

                        // store stack pointer
                        let char_ptr_reg = self.get_available_reg(&Some(register.clone()));
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
                                    text.push_str(format!("mov [{}-{}], dword 0x", char_ptr_reg.to_str(NASMRegSize::L64), 4).as_str());
                                    text.push_str(format!("{:X}{:X}{:X}{:X}\n", last_chunk[0],last_chunk[1],last_chunk[2],last_chunk[3]).as_str());
                                }
                                if len % 2 < len {
                                    text.push_str(format!("mov [{}-{}], word 0x", char_ptr_reg.to_str(NASMRegSize::L64), (offset+2)).as_str());
                                    text.push_str(format!("{:X}{:X}\n", last_chunk[offset],last_chunk[offset + 1]).as_str());
                                    offset += 2;
                                }
                                if offset != len {
                                    text.push_str(format!("mov [{}-{}], byte 0x", char_ptr_reg.to_str(NASMRegSize::L64), (offset+1)).as_str());
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

    fn end_cur_code(&mut self) {
        self.code_vec.push(self.cur_code.clone());
        self.cur_code = Code::new(self.cur_code.lang.clone());
    }

    fn get_available_reg(&mut self, ret_reg: &Option<NASMRegBase>) -> NASMRegBase {
        match ret_reg {
            Some(register) => if &self.available_regs[0] == register
            { self.available_regs.remove(1) }
            else { self.available_regs.remove(0) },
            None => { self.available_regs.remove(0) },
        }
    }
    fn pop_available_reg(&mut self, ret_reg: &Option<NASMRegBase>) -> NASMRegBase {
        let reg = self.get_available_reg(ret_reg);
        self.available_regs.retain(|x| x != &reg );
        reg
    }
}