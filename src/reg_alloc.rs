use crate::x64::{X64Function, X64Program, X64RegisterAllocator, X64};

pub fn alloc(asm: X64Program) -> X64Program {
    asm.into_iter()
        .map(|X64Function { name, body }| X64Function {
            name,
            body: alloc_body(body),
        })
        .collect()
}

fn alloc_body(body: Vec<X64>) -> Vec<X64> {
    let mut allocator = X64RegisterAllocator::new();
    let mut asms = allocator.prolog();
    for asm in body {
        asms.extend(match asm {
            X64::MovNum(reg, num) => {
                allocator.free(reg);
                let (mut asms, reg) = allocator.alloc(reg);
                asms.push(X64::MovNum(reg, num));
                asms
            }
            X64::MovReg(left, right) => {
                allocator.free(left);
                let (mut left_asms, left) = allocator.alloc(left);
                let (right_asms, right) = allocator.alloc(right);
                left_asms.extend(right_asms);
                left_asms.push(X64::MovReg(left, right));
                left_asms
            }
            X64::Call(func, args) => {
                let mut asms = allocator.call_prolog(args);
                asms.push(X64::Call(func, Vec::new()));
                asms.extend(allocator.call_epilog());
                asms
            }
            X64::Neg(reg) => {
                let (mut asms, reg) = allocator.alloc(reg);
                asms.push(X64::Neg(reg));
                asms
            }
            X64::CmpNum(reg, num) => {
                let (mut asms, reg) = allocator.alloc(reg);
                asms.push(X64::CmpNum(reg, num));
                asms
            }
            X64::CmpReg(left, right) => {
                let (mut left_asms, left) = allocator.alloc(left);
                let (right_asms, right) = allocator.alloc(right);
                left_asms.extend(right_asms);
                left_asms.push(X64::CmpReg(left, right));
                left_asms
            }
            X64::Imul(left, right) => {
                let (mut left_asms, left) = allocator.alloc(left);
                let (right_asms, right) = allocator.alloc(right);
                left_asms.extend(right_asms);
                left_asms.push(X64::Imul(left, right));
                left_asms
            }
            X64::Idiv(left, right) => {
                let (mut left_asms, left) = allocator.alloc(left);
                let (right_asms, right) = allocator.alloc(right);
                left_asms.extend(right_asms);
                left_asms.push(X64::Idiv(left, right));
                left_asms
            }
            X64::Add(left, right) => {
                let (mut left_asms, left) = allocator.alloc(left);
                let (right_asms, right) = allocator.alloc(right);
                left_asms.extend(right_asms);
                left_asms.push(X64::Add(left, right));
                left_asms
            }
            X64::Sub(left, right) => {
                let (mut left_asms, left) = allocator.alloc(left);
                let (right_asms, right) = allocator.alloc(right);
                left_asms.extend(right_asms);
                left_asms.push(X64::Sub(left, right));
                left_asms
            }
            X64::And(left, right) => {
                let (mut left_asms, left) = allocator.alloc(left);
                let (right_asms, right) = allocator.alloc(right);
                left_asms.extend(right_asms);
                left_asms.push(X64::And(left, right));
                left_asms
            }
            X64::Or(left, right) => {
                let (mut left_asms, left) = allocator.alloc(left);
                let (right_asms, right) = allocator.alloc(right);
                left_asms.extend(right_asms);
                left_asms.push(X64::Or(left, right));
                left_asms
            }
            X64::Ret(Some(reg)) => {
                let (mut asms, reg) = allocator.alloc(reg);
                asms.push(X64::MovReg(X64RegisterAllocator::RAX, reg));
                asms.push(X64::Ret(None));
                asms
            }
            _ => Vec::new(),
        });
    }
    asms.extend(allocator.epilog());
    asms
}
