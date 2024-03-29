use std::rc::Rc;
use crate::token::literal::Literal;
use super::{Expr, TypeCheck, core_lib::*, dtype::{DType, Msg}, env::Environment, fill_slice_with_vec, interpreter::Interpret, type_checker::TypeError};

pub struct Decl {
    pub name: String,
    pub dtype: DType
}
impl Decl {
    pub fn from_bytes(bytes: [u8; 22], env: &mut Environment) -> Option<Self> {
        let mut name_bytes = [0; 16]; // TODO: find more efficient way to do this
        for i in 0..16 {
            name_bytes[i] = bytes[i];
        }
        let name = str_from_jstr(name_bytes, env)?;

        let mut type_bytes = [0; 6]; // TODO: find more efficient way to do this
        for i in 0..6 {
            type_bytes[i] = bytes[i+16];
        }
        let dtype = DType::from_bytes(type_bytes);

        Some(Self { name, dtype })
    }
    pub fn from_expr(expr: &mut Expr, env: &mut Environment) -> Option<Self> {
        let mut decl_slice = [0; 22];
        let expr_bytes = expr.interpret(env)?.0;
        fill_slice_with_vec(&mut decl_slice, expr_bytes);
        Decl::from_bytes(decl_slice, env)
    }

    pub fn initialize(&self, val: &mut Expr, env: &mut Environment) -> Result<DType, TypeError> {
        let dtype = val.check(env)?;
        let final_dtype = self.dtype.union(&dtype)
            .ok_or(TypeError::new("initialization value does not match declared type".into(), None))?;
        // println!("{:?} {:?} {:?}", self.dtype, dtype, final_dtype);

        match val.interpret(env) {
            Some((bytes, ct_dtype)) => {
                let mut byte_lits = vec![];
                for byte in bytes {
                    byte_lits.push(Expr::Literal(Literal::Byte(byte)));
                }
                let constructor = move |_: Option<Box<Expr>>, _: &Environment, _: Option<Box<Expr>>|
                { Expr::Object(byte_lits.clone()) };
                if ct_dtype == final_dtype {
                    env.add_rt_msg(Msg::new(self.name.clone(), Rc::new(constructor.clone()), final_dtype.clone(), None));
                } else {
                    // add runtime msg TODO: defer code to a function
                    let constructor = move |_: Option<Box<Expr>>, _: &Environment, _: Option<Box<Expr>>|
                    { Expr::Object(vec![]) }; // TODO: return asm node
                    env.add_rt_msg(Msg::new(self.name.clone(), Rc::new(constructor), final_dtype.clone(), None));
                    env.add_rt_size(final_dtype.size);
                    println!("rt stack: {:?}", env.get_rt_stack_type());
                }
                env.add_ct_msg(Msg::new(self.name.clone(), Rc::new(constructor), ct_dtype, None));
            },
            None => {
                // add runtime msg TODO: defer code to a function
                let constructor = move |_: Option<Box<Expr>>, _: &Environment, _: Option<Box<Expr>>|
                { Expr::Object(vec![]) }; // TODO: return asm node
                env.add_rt_msg(Msg::new(self.name.clone(), Rc::new(constructor), final_dtype.clone(), None));
                env.add_rt_size(final_dtype.size);
                println!("rt stack: {:?}", env.get_rt_stack_type());
            },
        }

        Ok(dtype)
    }
    pub fn ct_initialize(&self, mut val: Expr, env: &mut Environment) -> Option<(Vec<u8>, DType)> {
        let dtype = val.check(env).ok()?;
        if self.dtype != dtype { return None }

        let (bytes, ct_dtype) = val.interpret(env)?;
        let mut byte_lits = vec![];
        for byte in bytes.clone() {
            byte_lits.push(Expr::Literal(Literal::Byte(byte)));
        }
        let constructor = move |_: Option<Box<Expr>>, _: &Environment, _: Option<Box<Expr>>|
        { Expr::Object(byte_lits.clone()) };
        env.add_ct_msg(Msg::new(self.name.clone(), Rc::new(constructor), ct_dtype, None));

        Some((bytes, dtype))
    }
}