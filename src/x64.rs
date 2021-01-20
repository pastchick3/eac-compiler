use crate::ir::SSAVar;
use std::collections::HashMap;

pub type Register = usize;

#[derive(Debug, PartialEq)]
pub struct RegisterBuilder {
    count: usize,
    var_map: HashMap<SSAVar, Register>,
}

impl RegisterBuilder {
    pub fn new() -> Self {
        RegisterBuilder {
            count: 0,
            var_map: HashMap::new(),
        }
    }

    pub fn from_var(&mut self, var: SSAVar) -> Register {
        let reg = *self.var_map.entry(var).or_insert(self.count);
        self.count += 1;
        reg
    }

    pub fn create_temp(&mut self) -> Register {
        let reg = self.count;
        self.count += 1;
        reg
    }

    pub fn clear(&mut self) {
        self.count = 0;
    }
}

#[derive(Debug, PartialEq)]
pub enum X64 {
    MovNum(Register, i32),
    MovReg(Register, Register),
    Call(String, Vec<Register>),
    Neg(Register),
    CmpNum(Register, i32),
    CmpReg(Register, Register),
    Jl(String),
    Jg(String),
    Jle(String),
    Jge(String),
    Je(String),
    Jne(String),
    Jump(String),
    Tag(String),
    Imul(Register, Register),
    Idiv(Register, Register),
    Add(Register, Register),
    Sub(Register, Register),
    And(Register, Register),
    Or(Register, Register),
    Ret(Option<Register>),
}

#[derive(Debug, PartialEq)]
pub struct X64Function {
    pub name: String,
    pub body: Vec<X64>,
}

pub type X64Program = Vec<X64Function>;
