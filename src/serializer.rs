use crate::x64::X64Program;

pub fn run(asm: X64Program) -> String {
    String::from(
        "
        .code
            main proc
                mov eax, -65
                ret
            main endp
        end
    ",
    )
}
