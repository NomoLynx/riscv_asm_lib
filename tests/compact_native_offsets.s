# Literal displacements select the optimized two-byte control instructions.
# Build this separate sensitivity fixture with compact optimization on.
.text
_start:
    ori x7, x0, 0
    li x8, 0
    beq x8, x0, 6
    ori x7, x0, 1
    bne x7, x0, _fail
    li x8, 1
    bne x0, x8, 6
    ori x7, x0, 2
    bne x7, x0, _fail
    jal x0, 6
    ori x7, x0, 4
    bne x7, x0, _fail
    li x8, 3
    addi x8, x8, -1
    bne x8, x0, -2
    bne x8, x0, _fail
    li a0, 42
    li a7, 93
    ecall
    jal x0, 0
_fail:
    li a0, 17
    li a7, 93
    ecall
    jal x0, 0
