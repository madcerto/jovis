use std::ffi::CString;
use crate::{expr::parser::Parser, linker::j_link, token::{literal::Literal, scanner::Scanner}};
use super::{Expr, TypeCheck, interpreter::Interpret, asm_type::{AsmLanguage, AsmTarget, NASMRegSize, NASMRegBase}, Environment, core_lib::*};

pub fn _generate_code(mut ast: Expr, out_file: String, _target: AsmTarget, mut env: Environment) {
    ast.check_new_env().unwrap();
    // turn ast into assembly
    let mut generator = CodeGenerator::new(AsmLanguage::NASM);
    generator.gen_nasm(ast, &mut env, Some(NASMRegBase::A));
    // TODO: Generate IR
    // TODO: Write IR to file
    // call linker on IR file
    unsafe {
        j_link(CString::new(out_file).unwrap().as_ptr());
    }
}

struct CodeGenerator {
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

    pub fn gen_nasm(&mut self, ast: Expr, env: &mut Environment, reg_opt: Option<NASMRegBase>) -> bool { // TODO
        match ast {
            Expr::Binary(_, _, right) => {
                let register = match reg_opt {
                    Some(register) => register,
                    None => self.available_regs[0].clone(),
                };
                let is_ptr = self.gen_nasm(*right, env, Some(register.clone()));
                if !is_ptr {
                    let mut code = format!("push {};", register.to_str(NASMRegSize::L32)).as_bytes().to_vec();
                    self.cur_code.asm.append(&mut code);
                }
                is_ptr
            },
            Expr::MsgEmission(_, _, _) => panic!("unexpected msg emission in code generator"),
            Expr::BinaryOpt(_, _, _) => todo!(), // leave as todo for a while because mostly unnecessary
            Expr::Asm(_asm_type, mut text_expr) => {
                // TODO: handle different assembly types
                let mut text = match text_expr.interpret(env) { // TODO: if string literal, get string directly
                    Some((text_bytes, text_type)) => if text_type == STRING {
                        if text_bytes.len() as u32 == STRING.size {
                            let mut text_slice: [u8; 16] = [0; 16]; // TODO: find more efficient way to do this
                            for i in 0..16 {
                                text_slice[i] = text_bytes[i];
                            }
                            str_from_jstr(text_slice, env).expect("could not get string from stack")
                        }
                        else { panic!("jstr is of incorrect size") }
                    } else { panic!("expected string") },
                    None => panic!("expected static expression")
                };

                // TODO: we're forced to run the scanner and parser twice, which might not be an
                // issue since they're usually pretty small, but still would probably be best
                // to have some sort of caching for this, maybe just give in and make a new data structure
                // handle embedded jovis expressions
                for (i,_) in text.match_indices("j#") {
                    let mut scanner  = Scanner::new(text.get((i+2)..).unwrap().to_string());
                    let (tokens, n) = scanner.scan_tokens_err_ignore();
                    let mut parser = Parser::new(tokens);
                    let mut expr = parser.parse();
                    expr.check(env).unwrap();

                    // get last semicolon before jovis expression, and push everything before that into cur_code
                    let mut line_start = i;
                    loop {
                        if line_start == 0
                        || text.get(line_start..(line_start+1)).unwrap() == ";"
                        { break }
                        line_start -= 1;
                    }
                    let preceeding_text = text.get(0..line_start).unwrap();
                    text.replace_range(0..line_start, "");
                    self.cur_code.asm.append(&mut preceeding_text.as_bytes().to_vec());
                    // generate code from checked expr
                    let register = self.get_available_reg(&reg_opt);
                    self.gen_nasm(expr, env, Some(register));
                    // replace expression in text with register
                    text.replace_range(i..n, register.to_str(NASMRegSize::L32).as_str());
                }

                // handle return expressions
                let mut is_ptr = false;
                for (i,_) in text.clone().match_indices("jret#") {
                    let (mut n, is_ptr_in) = match text.get((i+5)..) {
                        Some(text) => if let Some((i, _)) = text.match_indices("addr(").next() {
                            (i, true)
                        } else if let Some((i, _)) = text.match_indices("val(").next() {
                            (i, false)
                        } else { panic!("expected 'addr' or 'val'") },
                        None => panic!("value of i+5 seems to be too large"),
                    };
                    let mut operand = "".to_string();
                    loop {
                        let c = match text.chars().nth(n) {
                            Some(c) => c,
                            None => panic!("unterminated jret expression in asm"),
                        };
                        if c == ')' { break };
                        operand.push(c);
                        n += 1;
                    }
                    // generate code for return
                    let asm_ret_text;
                    match &reg_opt {
                        Some(register) => {
                            asm_ret_text = format!("mov {}, {};", register.to_str(NASMRegSize::L32), operand.trim());
                        },
                        None => asm_ret_text = "".to_string(),
                    }
                    // remove return from text
                    text.replace_range(i..(n+1), asm_ret_text.as_str());
                    // set is_ptr
                    is_ptr = is_ptr_in;
                }
                // add text to current code object
                self.cur_code.asm.append(&mut text.as_bytes().to_vec());
                return is_ptr
            },
            Expr::Object(exprs) => if let Some(register) = reg_opt{
                let val_reg = if self.available_regs[0] == register
                    { self.available_regs.remove(1) }
                    else { self.available_regs.remove(0) };
                let mut addr_save = format!("mov {}, rsp", register.clone().to_str(NASMRegSize::L64)).as_bytes().to_vec();
                self.cur_code.asm.append(&mut addr_save);
                for expr in exprs {
                    if !self.gen_nasm(expr, env, Some(val_reg.clone())) {
                        let mut code = format!("push {};", val_reg.clone().to_str(NASMRegSize::L32)).as_bytes().to_vec();
                        self.cur_code.asm.append(&mut code);
                    }
                }
                true
            } else { false },
            Expr::Fn(_capture_list, expr) => {
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
                let mut init = "push rbp; mov rbp, rsp;".as_bytes().to_vec();
                self.cur_code.asm.append(&mut init);
                // TODO: do something with capture list idk

                self.gen_nasm(*expr, env, reg_opt.clone());
                // add cleanup code
                let mut deinit = "mov rsp, rbp; pop rbp;".as_bytes().to_vec();
                self.cur_code.asm.append(&mut deinit);

                if let Some(register) = reg_opt {
                    let mut func_return = format!("mov {}, f{};", register.to_str(NASMRegSize::L64), self.code_vec.len()).as_bytes().to_vec();
                    prev_code.asm.append(&mut func_return);
                }
                self.code_vec.push(self.cur_code.clone());

                self.cur_code = prev_code;
                self.available_regs = prev_regs;
                false
            },
            Expr::CodeBlock(exprs) => {
                let mut is_ptr = false;
                for (i, expr) in exprs.clone().into_iter().enumerate() {
                    if i == (exprs.len()-1) { is_ptr = self.gen_nasm(expr, env, reg_opt.clone()); }
                        else { self.gen_nasm(expr, env, None); };
                }
                is_ptr
            },
            Expr::Type(_) => todo!(), // leave as todo for a while because mostly unnecessary
            Expr::Literal(lit) => if let Some(register) = reg_opt {
                match lit {
                    Literal::String(_s) => todo!(),
                    Literal::Char(c) => {
                        let mut code = format!("mov {}, 0x{:X};", register.to_str(NASMRegSize::L8), c as u8).as_bytes().to_vec();
                        self.cur_code.asm.append(&mut code);
                        false
                    },
                    Literal::Integer(i) => {
                        let mut code = format!("mov {}, 0x{:X};", register.to_str(NASMRegSize::L32), i as u32).as_bytes().to_vec();
                        self.cur_code.asm.append(&mut code);
                        false
                    },
                    Literal::Float(f) => {
                        let mut code = format!("mov {}, 0x{:X};", register.to_str(NASMRegSize::L32), f as u32).as_bytes().to_vec();
                        self.cur_code.asm.append(&mut code);
                        false
                    },
                    Literal::Byte(b) => {
                        let mut code = format!("mov {}, 0x{:X};", register.to_str(NASMRegSize::L8), b).as_bytes().to_vec();
                        self.cur_code.asm.append(&mut code);
                        false
                    },
                }
            } else { false },
        }
    }

    fn get_available_reg(&mut self, ret_reg: &Option<NASMRegBase>) -> NASMRegBase {
        match ret_reg {
            Some(register) => if &self.available_regs[0] == register
            { self.available_regs.remove(1) }
            else { self.available_regs.remove(0) },
            None => { self.available_regs.remove(0) },
        }
    }
}