.global main

.section .data

hex_format: .asciz "%#x"

.section .text

.macro trap
    movq    $62, %rax
    movq    %r12, %rdi
    movq    $5, %rsi
    syscall
.endm

main:
    push    %rbp
    movq    %rsp, %rbp

    # get PID
    movq    $39, %rax
    syscall
    movq    %rax, %r12

    # debugger writes to rsi while we're stopped

    trap

    # print contents of rsi
    leaq    hex_format(%rip), %rdi
    movq    $0, %rax
    call    printf@plt
    movq    $0, %rdi
    call    fflush@plt

    trap

    popq    %rbp
    movq    $0, %rax
    ret
