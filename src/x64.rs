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
        self.var_map.clear();
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

#[derive(Debug)]
enum VRegStatus {
    Reg(Register),
    Stack(usize),
}

#[derive(Debug)]
pub struct X64RegisterAllocator {
    vreg_map: HashMap<Register, VRegStatus>,
    last: Register,
    stack: Vec<bool>,
    x64regs: Vec<Register>,
}

impl X64RegisterAllocator {
    pub const INT_SIZE: usize = 4;
    pub const FRAME_SIZE: usize = Self::INT_SIZE * 128;
    pub const RAX: Register = Register::X64(X64Register::RAX);
    pub const RBX: Register = Register::X64(X64Register::RBX);
    pub const RCX: Register = Register::X64(X64Register::RCX);
    pub const RDX: Register = Register::X64(X64Register::RDX);
    pub const RBP: Register = Register::X64(X64Register::RBP);
    pub const RSI: Register = Register::X64(X64Register::RSI);
    pub const RDI: Register = Register::X64(X64Register::RDI);
    pub const RSP: Register = Register::X64(X64Register::RSP);
    pub const R8: Register = Register::X64(X64Register::R8);
    pub const R9: Register = Register::X64(X64Register::R9);
    pub const R10: Register = Register::X64(X64Register::R10);
    pub const R11: Register = Register::X64(X64Register::R11);
    pub const R12: Register = Register::X64(X64Register::R12);
    pub const R13: Register = Register::X64(X64Register::R13);
    pub const R14: Register = Register::X64(X64Register::R14);
    pub const R15: Register = Register::X64(X64Register::R15);

    pub fn new(params: usize) -> Self {
        let mut allocator = X64RegisterAllocator {
            vreg_map: HashMap::new(),
            last: Self::RBP,
            stack: Vec::new(),
            x64regs: vec![
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
        };
        allocator.stack = vec![false; params];
        for i in 0..params {
            let vreg = Register::Virtual(i);
            match i {
                0 => {
                    let rcx= allocator.x64regs.remove(1);
                    allocator.vreg_map.insert(vreg, VRegStatus::Reg(rcx));
                }
                1 => {
                    let rdx= allocator.x64regs.remove(1);
                    allocator.vreg_map.insert(vreg, VRegStatus::Reg(rdx));
                }
                2 => {
                    let r8= allocator.x64regs.remove(3);
                    allocator.vreg_map.insert(vreg, VRegStatus::Reg(r8));
                }
                3 => {
                    let rcx= allocator.x64regs.remove(3);
                    allocator.vreg_map.insert(vreg, VRegStatus::Reg(rcx));
                }
                i => {
                    allocator.vreg_map.insert(vreg, VRegStatus::Stack(i*Self::INT_SIZE));
                }
            }
        }
        allocator
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
            X64::Pop(Self::R15),
            X64::Pop(Self::R14),
            X64::Pop(Self::R13),
            X64::Pop(Self::R12),
            X64::Pop(Self::RDI),
            X64::Pop(Self::RSI),
            X64::Pop(Self::RBX),
            X64::Ret(None),
        ]
    }

    pub fn call_prolog(&mut self, args: Vec<Register>, ret:Register) -> (Vec<X64>, Register) {
        let mut assemblies = vec![
            X64::Push(Self::RCX),
            X64::Push(Self::RDX),
            X64::Push(Self::R8),
            X64::Push(Self::R9),
            X64::Push(Self::R10),
            X64::Push(Self::R11),
            X64::SubNum(Self::RSP, Self::FRAME_SIZE),
            X64::MovReg(Self::RBP, Self::RSP),
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
        let (asms, ret) = self.alloc(ret);
        assemblies.extend(asms);
        (assemblies, ret)
    }

    pub fn call_epilog(&self) -> Vec<X64> {
        vec![
            X64::AddNum(Self::RSP, Self::FRAME_SIZE),
            X64::Pop(Self::R11),
            X64::Pop(Self::R10),
            X64::Pop(Self::R9),
            X64::Pop(Self::R8),
            X64::Pop(Self::RDX),
            X64::Pop(Self::RCX),
        ]
    }

    pub fn ret(&mut self, vreg: Register) -> Vec<X64> {
        let (mut asms, reg) = self.alloc(vreg);
        asms.push(X64::MovReg(Self::RAX, reg));
        asms
    }

    pub fn alloc(&mut self, vreg: Register) -> (Vec<X64>, Register) {
        if let reg @ Register::X64(_) = vreg {
            return (Vec::new(), reg)
        }
        match self.vreg_map.remove(&vreg) {
            Some(VRegStatus::Reg(reg)) => {
                self.vreg_map.insert(vreg, VRegStatus::Reg(reg));
                (Vec::new(), reg)
            }
            Some(VRegStatus::Stack(offset)) => {
                let (mut asms, reg) = self.ensure_reg();
                asms.push(X64::MovFromStack(reg, offset));
                self.vreg_map.insert(vreg, VRegStatus::Reg(reg));
                (asms, reg)
            }
            None => {
                let (asms, reg) = self.ensure_reg();
                self.vreg_map.insert(vreg, VRegStatus::Reg(reg));
                (asms, reg)
            }
        }
    }

    fn ensure_reg(&mut self) -> (Vec<X64>, Register) {
        match self.x64regs.pop() {
            Some(reg) => (Vec::new(), reg),
            None => {
                let offset = self.alloc_stack();
                for (vreg, status) in self.vreg_map.iter_mut() {
                    if *vreg != self.last {
                        if let VRegStatus::Reg(reg) = *status {
                            self.last = *vreg;
                            let asms = vec![X64::MovToStack(offset, reg)];
                            *status = VRegStatus::Stack(offset);
                            return (asms, reg);
                        }
                    }
                }
                unreachable!()
            }
        }
    }

    fn alloc_stack(&mut self) -> usize {
        for (i, freed) in self.stack.iter_mut().enumerate() {
            if *freed {
                *freed = false;
                return i * Self::INT_SIZE;
            }
        }
        self.stack.push(false);
        let offset = (self.stack.len() - 1) * Self::INT_SIZE;
        offset
    }
}

#[derive(Debug, PartialEq)]
pub enum X64 {
    MovNum(Register, i32),
    MovReg(Register, Register),
    MovToStack(usize, Register),
    MovFromStack(Register, usize),
    Call(String, Vec<Register>, Register),
    Neg(Register),
    CmpNum(Register, i32),
    CmpReg(Register, Register),
    Jl(String),
    Jg(String),
    Jle(String),
    Jge(String),
    Je(String),
    Jne(String),
    Jmp(String),
    Tag(String),
    Imul(Register, Register),
    Idiv(Register, Register),
    Add(Register, Register),
    AddNum(Register, usize),
    Sub(Register, Register),
    SubNum(Register, usize),
    And(Register, Register),
    Or(Register, Register),
    Ret(Option<Register>),
    Push(Register),
    Pop(Register),
}

impl Display for X64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            X64::MovNum(reg, num) => write!(f, "mov {}, {}", reg, num),
            X64::MovReg(left, right) => write!(f, "mov {}, {}", left, right),
            X64::MovToStack(offset, reg) => write!(f, "mov {}[RBP], {}", offset, reg),
            X64::MovFromStack(reg, offset) => write!(f, "mov {}, {}[RBP]", reg, offset),
            X64::Call(name, _, _) => write!(f, "call {}", name),
            X64::Neg(reg) => write!(f, "neg {}", reg),
            X64::CmpNum(reg, num) => write!(f, "cmp {}, {}", reg, num),
            X64::CmpReg(left, right) => write!(f, "cmp {}, {}", left, right),
            X64::Jl(tag) => write!(f, "jl {}", tag),
            X64::Jg(tag) => write!(f, "jg {}", tag),
            X64::Jle(tag) => write!(f, "jle {}", tag),
            X64::Jge(tag) => write!(f, "jge {}", tag),
            X64::Je(tag) => write!(f, "je {}", tag),
            X64::Jne(tag) => write!(f, "jne {}", tag),
            X64::Jmp(tag) => write!(f, "jmp {}", tag),
            X64::Tag(tag) => write!(f, "{}:", tag),
            X64::Imul(left, right) => write!(f, "imul {}, {}", left, right),
            X64::Idiv(left, right) => write!(f, "idiv {}, {}", left, right),
            X64::Add(left, right) => write!(f, "add {}, {}", left, right),
            X64::AddNum(reg, offset) => write!(f, "add {}, {}", reg, offset),
            X64::Sub(left, right) => write!(f, "sub {}, {}", left, right),
            X64::SubNum(reg, offset) => write!(f, "sub {}, {}", reg, offset),
            X64::And(left, right) => write!(f, "and {}, {}", left, right),
            X64::Or(left, right) => write!(f, "or {}, {}", left, right),
            X64::Ret(_) => write!(f, "ret"),
            X64::Push(reg) => write!(f, "push {}", reg),
            X64::Pop(reg) => write!(f, "pop {}", reg),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct X64Function {
    pub name: String,
    pub params: usize,
    pub body: Vec<X64>,
}

pub type X64Program = Vec<X64Function>;
