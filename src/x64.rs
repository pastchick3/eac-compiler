use crate::ir::SSAVar;
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Register {
    Virtual(VRegister),
    X64(X64Register),
}

impl Display for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Register::Virtual(i) => write!(f, "VR{}", i),
            Register::X64(reg) => write!(f, "{:?}", reg),
        }
    }
}

pub type VRegister = usize;

pub struct VRegisterAllocator {
    count: usize,
    var_map: HashMap<SSAVar, VRegister>,
}

impl VRegisterAllocator {
    pub fn new() -> Self {
        VRegisterAllocator {
            count: 0,
            var_map: HashMap::new(),
        }
    }

    pub fn from_var(&mut self, var: SSAVar) -> Register {
        match self.var_map.get(&var) {
            Some(reg) => Register::Virtual(*reg),
            None => {
                let reg = self.count;
                self.count += 1;
                self.var_map.insert(var, reg);
                Register::Virtual(reg)
            }
        }
    }

    pub fn create_temp(&mut self) -> Register {
        let reg = self.count;
        self.count += 1;
        Register::Virtual(reg)
    }

    pub fn clear(&mut self) {
        self.count = 0;
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum X64Register {
    RAX,
    RBX,
    RCX,
    RDX,
    RBP,
    RSI,
    RDI,
    RSP,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

enum VRegStatus {
    Reg(Register),
    Stack(usize),
}

pub struct X64RegisterAllocator {
    vreg_map: HashMap<Register, VRegStatus>,
    last: Register,
    stack: Vec<bool>,
    x64regs: Vec<Register>,
}

impl X64RegisterAllocator {
    const INT_SIZE: usize = 4;
    pub const RAX: Register = Register::X64(X64Register::RAX);
    const RBX: Register = Register::X64(X64Register::RBX);
    const RCX: Register = Register::X64(X64Register::RCX);
    const RDX: Register = Register::X64(X64Register::RDX);
    const RBP: Register = Register::X64(X64Register::RBP);
    const RSI: Register = Register::X64(X64Register::RSI);
    const RDI: Register = Register::X64(X64Register::RDI);
    const RSP: Register = Register::X64(X64Register::RSP);
    const R8: Register = Register::X64(X64Register::R8);
    const R9: Register = Register::X64(X64Register::R9);
    const R10: Register = Register::X64(X64Register::R10);
    const R11: Register = Register::X64(X64Register::R11);
    const R12: Register = Register::X64(X64Register::R12);
    const R13: Register = Register::X64(X64Register::R13);
    const R14: Register = Register::X64(X64Register::R14);
    const R15: Register = Register::X64(X64Register::R15);

    pub fn new() -> Self {
        X64RegisterAllocator {
            vreg_map: HashMap::new(),
            last: Self::RBP,
            stack: Vec::new(),
            x64regs: vec![
                Self::RAX,
                Self::RBX,
                Self::RCX,
                Self::RDX,
                Self::RSI,
                Self::RDI,
                Self::R8,
                Self::R9,
                Self::R10,
                Self::R11,
                Self::R12,
                Self::R13,
                Self::R14,
                Self::R15,
            ],
        }
    }

    pub fn prolog(&self) -> Vec<X64> {
        vec![
            X64::Push(Self::RBX),
            X64::Push(Self::RSI),
            X64::Push(Self::RDI),
            X64::Push(Self::R12),
            X64::Push(Self::R13),
            X64::Push(Self::R14),
            X64::Push(Self::R15),
        ]
    }

    pub fn epilog(&self) -> Vec<X64> {
        vec![
            X64::Push(Self::R15),
            X64::Push(Self::R14),
            X64::Push(Self::R13),
            X64::Push(Self::R12),
            X64::Push(Self::RDI),
            X64::Push(Self::RSI),
            X64::Push(Self::RBX),
        ]
    }

    pub fn alloc(&mut self, vreg: Register) -> (Vec<X64>, Register) {
        match self.vreg_map.remove(&vreg) {
            Some(VRegStatus::Reg(reg)) => (Vec::new(), reg),
            Some(VRegStatus::Stack(offset)) => {
                let (mut asms, reg) = self.ensure_reg();
                asms.push(X64::MovFromStack(reg, offset));
                (asms, reg)
            }
            None => {
                let (asms, reg) = self.ensure_reg();
                self.vreg_map.insert(vreg, VRegStatus::Reg(reg));
                (asms, reg)
            }
        }
    }

    pub fn call_prolog(&mut self, args: Vec<Register>) -> Vec<X64> {
        let mut assemblies = vec![
            X64::Push(Self::RAX),
            X64::Push(Self::RCX),
            X64::Push(Self::RDX),
            X64::Push(Self::R8),
            X64::Push(Self::R9),
            X64::Push(Self::R10),
            X64::Push(Self::R11),
            X64::MovReg(Self::RBP, Self::RSP),
            X64::SubNum(Self::RSP, args.len() * Self::INT_SIZE),
        ];
        for (i, arg) in args.into_iter().enumerate() {
            let (asms, reg) = self.alloc(arg);
            assemblies.extend(asms);
            assemblies.push(X64::MovToStack(i * Self::INT_SIZE, reg));
            match i {
                0 => assemblies.push(X64::MovReg(Self::RCX, reg)),
                1 => assemblies.push(X64::MovReg(Self::RDX, reg)),
                2 => assemblies.push(X64::MovReg(Self::R8, reg)),
                3 => assemblies.push(X64::MovReg(Self::R9, reg)),
                _ => {}
            }
        }
        assemblies
    }

    pub fn call_epilog(&self) -> Vec<X64> {
        vec![
            X64::MovReg(Self::RSP, Self::RBP),
            X64::Pop(Self::R11),
            X64::Pop(Self::R10),
            X64::Pop(Self::R9),
            X64::Pop(Self::R8),
            X64::Pop(Self::RDX),
            X64::Pop(Self::RCX),
            X64::Pop(Self::RAX),
        ]
    }

    fn ensure_reg(&mut self) -> (Vec<X64>, Register) {
        match self.x64regs.pop() {
            Some(reg) => (Vec::new(), reg),
            None => {
                let (mut asms, offset) = self.alloc_stack();
                for (vreg, status) in self.vreg_map.iter_mut() {
                    if *vreg != self.last {
                        if let VRegStatus::Reg(reg) = *status {
                            self.last = *vreg;
                            asms.push(X64::MovToStack(offset, reg));
                            *status = VRegStatus::Stack(offset);
                            return (asms, reg);
                        }
                    }
                }
                unreachable!()
            }
        }
    }

    fn alloc_stack(&mut self) -> (Vec<X64>, usize) {
        for (i, freed) in self.stack.iter_mut().enumerate() {
            if *freed {
                *freed = false;
                return (Vec::new(), i * Self::INT_SIZE);
            }
        }
        self.stack.push(false);
        let asm = X64::SubNum(Self::RBP, Self::INT_SIZE);
        let offset = (self.stack.len() - 1) * Self::INT_SIZE;
        (vec![asm], offset)
    }

    pub fn free(&mut self, vreg: Register) {
        match self.vreg_map.remove(&vreg) {
            Some(VRegStatus::Reg(reg)) => self.x64regs.push(reg),
            Some(VRegStatus::Stack(offset)) => {
                self.stack[offset / Self::INT_SIZE] = true;
            }
            None => {}
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum X64 {
    MovNum(Register, i32),
    MovReg(Register, Register),
    MovToStack(usize, Register),
    MovFromStack(Register, usize),
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
    SubNum(Register, usize),
    And(Register, Register),
    Or(Register, Register),
    Ret(Option<Register>),
    Push(Register),
    Pop(Register),
}

#[derive(Debug, PartialEq)]
pub struct X64Function {
    pub name: String,
    pub body: Vec<X64>,
}

pub type X64Program = Vec<X64Function>;
