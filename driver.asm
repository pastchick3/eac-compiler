; A driver program that prints the content of the `eax` register in signed decimal.
; Compile command: ml64 driver.asm main.asm /Fe main.exe /link /subsystem:console /defaultlib:kernel32.lib /entry:drive

extern GetStdHandle: proc
extern WriteFile: proc
extern ExitProcess: proc
extern main: proc

.data
    std_out dword -11
    buffer byte '-----------'
    len dword 11
    written dword 0

.code
    drive proc
        ; Save caller-saved registers.
        push RCX
        push RDX
        push R8
        push R9
        push R10
        push R11
        ; Allocate the stack frame.
        sub RSP, 512
        ; Set the stack frame pointer.
        mov RBP, RSP

        ; Call the `main` function.
        call main

        ; Clean the stack.
        add RSP, 512
        ; Restore caller-saved registers.
        pop R11
        pop R10
        pop R9
        pop R8
        pop RDX
        pop RCX

        ; Print the result.
        call print_dec
        ; Exit the program.
        xor rcx, rcx
        call ExitProcess
    drive endp

    print_dec proc
        ; Compute the decimal form of `eax`.
        mov ebx, 10
        xor ecx, ecx ; Set a flag showing `eax` is not a negative number.
        lea r8, buffer
        mov r9d, len
        cmp eax, 0
        jge WhileNotZero
        neg eax
        mov ecx, 1 ; Set a flag showing `eax` is a negative number.
        WhileNotZero:
            xor edx, edx
            div ebx
            add edx, 48 ; Convert the remainder to an ASCII digit.
            dec r9d
            mov [r8+r9], dl
            cmp eax, 0
            jg WhileNotZero
        sub r9d, ecx ; `r9d` will be the buffer offset.
        sub len, r9d ; `len` will be the buffer length.

        ; Print to the standard output.
        mov ecx, std_out
        call GetStdHandle
        mov rcx, rax
        lea rdx, [r8+r9]
        mov r8d, len
        lea r9, written
        call WriteFile

        ret
    print_dec endp
end
