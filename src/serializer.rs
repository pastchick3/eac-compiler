use crate::x64::{X64Function, X64Program};

const INDENT_SIZE: usize = 4;

pub fn run(asm: X64Program) -> String {
    let mut file = String::from(".code\n");
    let mut indent_level = 1;
    for X64Function { name, body } in asm {
        file += &format!("{}{} proc\n", indent(indent_level), name);
        indent_level += 1;
        for asm in body {
            file += &format!("{}{}\n", indent(indent_level), asm);
        }
        indent_level -= 1;
        file += &format!("{}{} endp\n\n", indent(indent_level), name);
    }
    file += "end\n";
    file
}

fn indent(indent_level: usize) -> String {
    String::from_utf8(vec![32; indent_level * INDENT_SIZE]).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::x64::{X64RegisterAllocator as X64R, X64};

    #[test]
    fn serialize() {
        let program = vec![X64Function {
            name: String::from("main"),
            body: vec![
                X64::MovNum(X64R::RSP, 0),
                X64::MovReg(X64R::RSP, X64R::RSP),
                X64::MovToStack(0, X64R::RSP),
                X64::MovFromStack(X64R::RSP, 0),
                X64::Call(String::from("Tag"), Vec::new()),
                X64::Neg(X64R::RSP),
                X64::CmpNum(X64R::RSP, 0),
                X64::CmpReg(X64R::RSP, X64R::RSP),
                X64::Jl(String::from("Tag")),
                X64::Jg(String::from("Tag")),
                X64::Jle(String::from("Tag")),
                X64::Jge(String::from("Tag")),
                X64::Je(String::from("Tag")),
                X64::Jne(String::from("Tag")),
                X64::Jump(String::from("Tag")),
                X64::Tag(String::from("Tag")),
                X64::Imul(X64R::RSP, X64R::RSP),
                X64::Idiv(X64R::RSP, X64R::RSP),
                X64::Add(X64R::RSP, X64R::RSP),
                X64::Sub(X64R::RSP, X64R::RSP),
                X64::SubNum(X64R::RSP, 0),
                X64::And(X64R::RSP, X64R::RSP),
                X64::Or(X64R::RSP, X64R::RSP),
                X64::Ret(None),
                X64::Push(X64R::RSP),
                X64::Pop(X64R::RSP),
            ],
        }];
        let file = run(program);
        let expected = ".code
    main proc
        mov RSP, 0
        mov RSP, RSP
        mov [RBP-0], RSP
        mov RSP, [RBP-0]
        call Tag
        neg RSP
        cmp RSP, 0
        cmp RSP, RSP
        jl Tag
        jg Tag
        jle Tag
        jge Tag
        je Tag
        jne Tag
        jump Tag
        Tag:
        imul RSP, RSP
        idiv RSP, RSP
        add RSP, RSP
        sub RSP, RSP
        sub RSP, 0
        and RSP, RSP
        or RSP, RSP
        ret
        push RSP
        pop RSP
    main endp

end
";
        assert_eq!(file, expected);
    }
}
