use std::ffi::CString;
use crate::{linker::j_link, token::literal::Literal};
use super::{Expr, TypeCheck, asm_type::{AsmLanguage, AsmTarget, NASMRegSize, NASMRegBase}, Environment};

pub fn _generate_code(mut ast: Expr, out_file: String, _target: AsmTarget, env: Environment) {
    ast.check_new_env().unwrap();
    // turn ast into assembly
    let mut generator = CodeGenerator::new(AsmLanguage::NASM);
    generator.gen_nasm(ast, &env, Some(NASMRegBase::A));
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

    pub fn gen_nasm(&mut self, ast: Expr, env: &Environment, reg_opt: Option<NASMRegBase>) -> bool { // TODO
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
            Expr::Asm(_asm_type, _code_expr) => {
                true
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
}