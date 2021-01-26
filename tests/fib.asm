.code
    fib proc
        push RBX
        push RSI
        push RDI
        push R12
        push R13
        push R14
        push R15
        mov R15, 2
        mov R14, 1
        cmp RCX, R15
        jle VR2
        mov R14, 0
        VR2:
        cmp R14, 0
        je VR2Start
        mov R13, 1
        mov R12, RCX
        sub R12, R13
        mov RAX, R12
        pop R15
        pop R14
        pop R13
        pop R12
        pop RDI
        pop RSI
        pop RBX
        ret
        jmp VR2End
        VR2Start:
        mov R11, 1
        mov R10, RCX
        sub R10, R11
        push RCX
        push RDX
        push R8
        push R9
        push R10
        push R11
        sub RSP, 512
        mov RBP, RSP
        mov 0[RBP], R10
        mov RCX, R10
        call fib
        add RSP, 512
        pop R11
        pop R10
        pop R9
        pop R8
        pop RDX
        pop RCX
        mov R9, RAX
        mov R8, 2
        mov RDI, RCX
        sub RDI, R8
        push RCX
        push RDX
        push R8
        push R9
        push R10
        push R11
        sub RSP, 512
        mov RBP, RSP
        mov 0[RBP], RDI
        mov RCX, RDI
        call fib
        add RSP, 512
        pop R11
        pop R10
        pop R9
        pop R8
        pop RDX
        pop RCX
        mov RSI, RAX
        mov RDX, R9
        add RDX, RSI
        mov RAX, RDX
        pop R15
        pop R14
        pop R13
        pop R12
        pop RDI
        pop RSI
        pop RBX
        ret
        VR2End:
        pop R15
        pop R14
        pop R13
        pop R12
        pop RDI
        pop RSI
        pop RBX
        ret
    fib endp

    main proc
        push RBX
        push RSI
        push RDI
        push R12
        push R13
        push R14
        push R15
        mov R15, 10
        push RCX
        push RDX
        push R8
        push R9
        push R10
        push R11
        sub RSP, 512
        mov RBP, RSP
        mov 0[RBP], R15
        mov RCX, R15
        call fib
        add RSP, 512
        pop R11
        pop R10
        pop R9
        pop R8
        pop RDX
        pop RCX
        mov R14, RAX
        mov RAX, R14
        pop R15
        pop R14
        pop R13
        pop R12
        pop RDI
        pop RSI
        pop RBX
        ret
        pop R15
        pop R14
        pop R13
        pop R12
        pop RDI
        pop RSI
        pop RBX
        ret
    main endp

end
