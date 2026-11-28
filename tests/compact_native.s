.text
_start:
    addi x2, x2, -512
    # Keep arena initialization stores uncompressed with a non-prime base.
    addi x31, x2, 0
    sd x0, 248(x31)
    sd x0, 376(x31)
    sd x0, 496(x31)
    sd x0, 504(x31)
    li x9, 0x11223344
    sw x9, 252(x2)
    li x31, -1
    lw x31, 252(x2)
    bne x31, x9, _fail
    sd x9, 504(x2)
    li x31, -1
    ld x31, 504(x2)
    bne x31, x9, _fail
    addi x8, x2, 256
    sw x9, 124(x8)
    li x15, -1
    lw x15, 124(x8)
    bne x15, x9, _fail
    addi x31, x2, 0
    sd x0, 504(x31)
    sd x9, 248(x8)
    li x15, -1
    ld x15, 248(x8)
    bne x15, x9, _fail
    fmv.d.x f0, x0
    fld f0, 504(x2)
    li x31, -1
    fmv.x.d x31, f0
    bne x31, x9, _fail
    fsd f0, 496(x2)
    li x31, -1
    ld x31, 496(x2)
    bne x31, x9, _fail
    fmv.d.x f8, x0
    fld f8, 248(x8)
    addi x31, x2, 0
    sd x0, 496(x31)
    fsd f8, 240(x8)
    li x31, -1
    ld x31, 496(x2)
    bne x31, x9, _fail

    li x8, -1
    srli x8, x8, 63
    li x9, 1
    bne x8, x9, _fail
    slli x8, x8, 63
    srai x8, x8, 63
    li x9, -1
    bne x8, x9, _fail
    andi x8, x8, -32
    li x9, -32
    bne x8, x9, _fail
    lui x31, 0xfffff
    li x9, -4096
    bne x31, x9, _fail
    li x31, 1
    slli x31, x31, 63
    li x9, 0x8000000000000000
    bne x31, x9, _fail

    li x8, 9
    li x9, 4
    add x31, x0, x8
    bne x31, x8, _fail
    add x8, x8, x9
    li x15, 13
    bne x8, x15, _fail
    sub x8, x8, x9
    li x15, 9
    bne x8, x15, _fail
    xor x8, x8, x9
    li x15, 13
    bne x8, x15, _fail
    li x8, 9
    or x8, x8, x9
    bne x8, x15, _fail
    and x8, x8, x9
    bne x8, x9, _fail

    li x8, 0x180000000
    addiw x8, x8, 0
    li x15, -2147483648
    bne x8, x15, _fail
    li x8, 0x7fffffff
    li x9, 1
    addw x8, x8, x9
    bne x8, x15, _fail
    subw x8, x8, x9
    li x15, 0x7fffffff
    bne x8, x15, _fail

    ori x7, x0, 0
    la x5, _callee
    jalr x1, x5, 0
_after_call:
    la x6, _after_call
    bne x1, x6, _fail
    li x9, 1
    bne x7, x9, _fail
    addi x0, x0, 0
    addi x2, x2, 512
    li a0, 42
    li a7, 93
    ecall
    jal x0, 0

_callee:
    addi x7, x7, 1
    jalr x0, x1, 0
_fail:
    li a0, 17
    li a7, 93
    ecall
    jal x0, 0
