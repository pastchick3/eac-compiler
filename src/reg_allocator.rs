use crate::x64::{Register, X64Function, X64Program, X64RegisterAllocator, X64};

pub fn alloc(asm: X64Program) -> X64Program {
    asm.into_iter()
        .map(
            |X64Function {
                 name,
                 param_cnt,
                 body,
             }| X64Function {
                name,
                param_cnt,
                body: alloc_body(param_cnt, body),
            },
        )
        .collect()
}

fn alloc_body(param_cnt: usize, body: Vec<X64>) -> Vec<X64> {
    let mut allocator = X64RegisterAllocator::new(param_cnt);
    let mut assemblies = allocator.prolog();
    for asm in body {
        let asms = match asm {
            X64::MovNum(vreg, num) => {
                let (mut asms, reg) = allocator.alloc(vreg);
                asms.push(X64::MovNum(reg, num));
                asms
            }
            X64::MovReg(left, right) => {
                let (mut left_asms, left) = allocator.alloc(left);
                let (right_asms, right) = allocator.alloc(right);
                left_asms.extend(right_asms);
                left_asms.push(X64::MovReg(left, right));
                left_asms
            }
            X64::Call(func, args, ret) => {
                let mut asms = allocator.call_prolog(args);
                asms.push(X64::Call(func, Vec::new(), Register::Virtual(0)));
                asms.extend(allocator.call_epilog());
                let (a_s, ret) = allocator.alloc(ret);
                asms.extend(a_s);
                asms.push(X64::MovReg(ret, X64RegisterAllocator::RAX));
                asms
            }
            X64::Neg(vreg) => {
                let (mut asms, reg) = allocator.alloc(vreg);
                asms.push(X64::Neg(reg));
                asms
            }
            X64::CmpNum(vreg, num) => {
                let (mut asms, reg) = allocator.alloc(vreg);
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
            X64::Ret(Some(vreg)) => {
                let mut asms = allocator.ret(vreg);
                asms.extend(allocator.epilog());
                asms
            }
            asm => vec![asm],
        };
        assemblies.extend(asms);
    }
    assemblies.extend(allocator.epilog());
    assemblies
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asm::X64Builder;
    use crate::parser;
    use crate::ssa;
    use crate::x64::X64RegisterAllocator as X64R;

    #[test]
    fn calling_convention() {
        let ast = parser::parse(
            "
            int f(int a, int b, int c, int d, int e) {
                return a;
            }

            int main() {
                int a;
                int b;
                int c;
                int d;
                int e;
                return f(a, b, c, d, e) + 1;
            }
        ",
        );
        let (ssa, prog_leaves) = ssa::construct(ast);
        let cfg = ssa::destruct(ssa, prog_leaves);
        let asm = X64Builder::new().build(cfg);
        let asm = alloc(asm);
        let expected = vec![
            X64Function {
                name: String::from("f"),
                param_cnt: 5,
                body: vec![
                    X64::Push(X64R::RBX),
                    X64::Push(X64R::RSI),
                    X64::Push(X64R::RDI),
                    X64::Push(X64R::R12),
                    X64::Push(X64R::R13),
                    X64::Push(X64R::R14),
                    X64::Push(X64R::R15),
                    X64::MovReg(X64R::RAX, X64R::RCX),
                    X64::Pop(X64R::R15),
                    X64::Pop(X64R::R14),
                    X64::Pop(X64R::R13),
                    X64::Pop(X64R::R12),
                    X64::Pop(X64R::RDI),
                    X64::Pop(X64R::RSI),
                    X64::Pop(X64R::RBX),
                    X64::Ret(None),
                    X64::Pop(X64R::R15),
                    X64::Pop(X64R::R14),
                    X64::Pop(X64R::R13),
                    X64::Pop(X64R::R12),
                    X64::Pop(X64R::RDI),
                    X64::Pop(X64R::RSI),
                    X64::Pop(X64R::RBX),
                    X64::Ret(None),
                ],
            },
            X64Function {
                name: String::from("main"),
                param_cnt: 0,
                body: vec![
                    X64::Push(X64R::RBX),
                    X64::Push(X64R::RSI),
                    X64::Push(X64R::RDI),
                    X64::Push(X64R::R12),
                    X64::Push(X64R::R13),
                    X64::Push(X64R::R14),
                    X64::Push(X64R::R15),
                    X64::Push(X64R::RCX),
                    X64::Push(X64R::RDX),
                    X64::Push(X64R::R8),
                    X64::Push(X64R::R9),
                    X64::Push(X64R::R10),
                    X64::Push(X64R::R11),
                    X64::SubNum(X64R::RSP, X64R::FRAME_SIZE),
                    X64::MovReg(X64R::RBP, X64R::RSP),
                    X64::MovToStack(0 * X64R::INT_SIZE, X64R::R15),
                    X64::MovReg(X64R::RCX, X64R::R15),
                    X64::MovToStack(1 * X64R::INT_SIZE, X64R::R14),
                    X64::MovReg(X64R::RDX, X64R::R14),
                    X64::MovToStack(2 * X64R::INT_SIZE, X64R::R13),
                    X64::MovReg(X64R::R8, X64R::R13),
                    X64::MovToStack(3 * X64R::INT_SIZE, X64R::R12),
                    X64::MovReg(X64R::R9, X64R::R12),
                    X64::MovToStack(4 * X64R::INT_SIZE, X64R::R11),
                    X64::Call(String::from("f"), Vec::new(), Register::Virtual(0)),
                    X64::AddNum(X64R::RSP, X64R::FRAME_SIZE),
                    X64::Pop(X64R::R11),
                    X64::Pop(X64R::R10),
                    X64::Pop(X64R::R9),
                    X64::Pop(X64R::R8),
                    X64::Pop(X64R::RDX),
                    X64::Pop(X64R::RCX),
                    X64::MovReg(X64R::R10, X64R::RAX),
                    X64::MovNum(X64R::R9, 1),
                    X64::MovReg(X64R::R8, X64R::R10),
                    X64::Add(X64R::R8, X64R::R9),
                    X64::MovReg(X64R::RAX, X64R::R8),
                    X64::Pop(X64R::R15),
                    X64::Pop(X64R::R14),
                    X64::Pop(X64R::R13),
                    X64::Pop(X64R::R12),
                    X64::Pop(X64R::RDI),
                    X64::Pop(X64R::RSI),
                    X64::Pop(X64R::RBX),
                    X64::Ret(None),
                    X64::Pop(X64R::R15),
                    X64::Pop(X64R::R14),
                    X64::Pop(X64R::R13),
                    X64::Pop(X64R::R12),
                    X64::Pop(X64R::RDI),
                    X64::Pop(X64R::RSI),
                    X64::Pop(X64R::RBX),
                    X64::Ret(None),
                ],
            },
        ];
        assert_eq!(asm, expected);
    }

    #[test]
    fn register_spilling() {
        let ast = parser::parse(
            "
            int main() {
                1+2+3+4+5+6+7;
                1;
            }
        ",
        );
        let (ssa, prog_leaves) = ssa::construct(ast);
        let cfg = ssa::destruct(ssa, prog_leaves);
        let asm = X64Builder::new().build(cfg);
        let asm = alloc(asm);
        if let X64::MovToStack(_, reg) = &asm[0].body[26] {
            let expected = vec![X64Function {
                name: String::from("main"),
                param_cnt: 0,
                body: vec![
                    X64::Push(X64R::RBX),
                    X64::Push(X64R::RSI),
                    X64::Push(X64R::RDI),
                    X64::Push(X64R::R12),
                    X64::Push(X64R::R13),
                    X64::Push(X64R::R14),
                    X64::Push(X64R::R15),
                    X64::MovNum(X64R::R15, 1),
                    X64::MovNum(X64R::R14, 2),
                    X64::MovReg(X64R::R13, X64R::R15),
                    X64::Add(X64R::R13, X64R::R14),
                    X64::MovNum(X64R::R12, 3),
                    X64::MovReg(X64R::R11, X64R::R13),
                    X64::Add(X64R::R11, X64R::R12),
                    X64::MovNum(X64R::R10, 4),
                    X64::MovReg(X64R::R9, X64R::R11),
                    X64::Add(X64R::R9, X64R::R10),
                    X64::MovNum(X64R::R8, 5),
                    X64::MovReg(X64R::RDI, X64R::R9),
                    X64::Add(X64R::RDI, X64R::R8),
                    X64::MovNum(X64R::RSI, 6),
                    X64::MovReg(X64R::RDX, X64R::RDI),
                    X64::Add(X64R::RDX, X64R::RSI),
                    X64::MovNum(X64R::RCX, 7),
                    X64::MovReg(X64R::RBX, X64R::RDX),
                    X64::Add(X64R::RBX, X64R::RCX),
                    X64::MovToStack(0, *reg),
                    X64::MovNum(*reg, 1),
                    X64::Pop(X64R::R15),
                    X64::Pop(X64R::R14),
                    X64::Pop(X64R::R13),
                    X64::Pop(X64R::R12),
                    X64::Pop(X64R::RDI),
                    X64::Pop(X64R::RSI),
                    X64::Pop(X64R::RBX),
                    X64::Ret(None),
                ],
            }];
            assert_eq!(asm, expected);
        } else {
            panic!()
        }
    }
}
