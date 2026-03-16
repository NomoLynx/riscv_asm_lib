# RiscV Assembly Language Macro Instructions

this file contains macro instruction for RiscV asm. Please make sure the asm code only contains basic instructions, not previlege ones or other types. check BaseIntegerInstructions for those instructions. if more instruction required to be added, reference exit.

| CLang | ASM |
| --- | --- |
| exit | [exit the code](#exit-the-code) |
| exit_reg | [exit code in reg](#exit-code-in-reg) |
| push64 | [push int64 to memory](#push-int64-to-memory) |
| pop64 | [pop int64](#pop-int64) |
| pushf | [push int32 to memory](#push-int32-to-memory) |
| popf | [pop int32](#pop-int32) |
| push32 | [push int32 to memory](#push-int32-to-memory) |
| pop32 | [pop int32](#pop-int32) |
| push | [push int32 to memory](#push-int32-to-memory) |
| pop | [pop int32](#pop-int32) |
| pushf32 | [push float32](#push-float32) |
| popf32 | [pop float32](#pop-float32) |
| push8 | [push int8 to memory](#push-int8-to-memory) |
| pop8 | [pop int8](#pop-int8) |
| push16 | [push int16 to memory](#push-int16-to-memory) |
| pop16 | [pop int16](#pop-int16) |
| pushf64 | [push float64](#push-float64) |
| popf64 | [pop float64](#pop-float64) |
| li.f32_near | [load f32 imm near](#load-f32-imm-near) |
| li.f32 | [load f32 imm](#load-f32-imm) |
| li.f64_near | [load f64 imm near](#load-f64-imm-near) |
| li.f64 | [load f64 imm](#load-f64-imm) |
| muli | [multi imm](#multi-imm) |
| divi | [divi imm](#divi-imm) |
| tt_panic | [assert value](#assert-value) |
| pushby | [push by](#push-by) |
| popby | [pop by](#pop-by) |
| inc | [inc](#inc) |

## exit the code

exit the current process, $1 is to be replaced with exit code.

```asm
    addi    a0, x0, $1   # Use a0 hold return code
    addi    a7, x0, 93  # Service command code 93 terminates
    ecall               # Call linux to terminate the program

```

## exit code in reg

exit the current process, $1 is the register holding the exit code

```asm
    add    a0, x0, $1   # Use a0 hold return code
    addi    a7, x0, 93  # Service command code 93 terminates
    ecall               # Call linux to terminate the program

```

## push int64 to memory

push int64 value in a register to memory.

```asm
    addi sp, sp, -8
    sd $1, sp, 0

```

## pop int64

pop int64 from memory

```asm
    ld $1, sp, 0
    addi sp, sp, 8

```

## push int32 to memory

push int32 value in a register to memory.

```asm
    addi sp, sp, -4
    sw $1, sp, 0

```

## pop int32

pop int32 from memory

```asm
    lw $1, sp, 0
    addi sp, sp, 4

```

## push int8 to memory

push int8 (byte) value in a register to memory.

```asm
    addi sp, sp, -4
    sb $1, sp, 0

```

## pop int8

pop int8 (byte) from memory

```asm
    lb $1, sp, 0
    addi sp, sp, 4

```

## push int16 to memory

push int16 value in a register to memory.

```asm
    addi sp, sp, -4
    sh $1, sp, 0

```

## pop int16

pop int16 from memory

```asm
    lh $1, sp, 0
    addi sp, sp, 4

```

## push float32

push float32 to memory

```asm
    addi sp, sp, -4
    fsw $1, sp, 0
```

## pop float32

pop float32 from memory

```asm
    addi sp, sp, 4
    flw $1, sp, 0
```

## push float64

push float64 to memory

```asm
    addi sp, sp, -8
    fsd $1, sp, 0
```

## pop float64

pop float64 from memory

```asm
    addi sp, sp, 8
    fld $1, sp, 0
```

## load f32 imm near

load f32 immeidate value to float register (64bit), this case shows the value is near value. the sample code is li.f32 ft0, t0, 3.14. It shows 3.14 is moved to ft0 via t0.

```asm
    addi $2, x0, $3
    fmv.s.x $1, $2
```

## load f32 imm

load f32 immediate value to float register (64bit). The sample code is li.f32 ft0, t0, 3.14. It shows 3.14 is moved to ft0 via t0.

```asm
    lui $2, $4          # load high portion
    addi $2, $2, $3     # add low portion
    fmv.s.x $1, $2
```

## load f64 imm near

load f64 immeidate value to float register (64bit), this case shows the value is near value. the sample code is li.f64 ft0, t0, 3.14. It shows 3.14 is moved to ft0 via t0.

```asm
    addi $2, x0, $3
    fmv.d.x $1, $2
```

## load f64 imm

load f64 immediate value to float register (64bit). The sample code is li.f64 ft0, t0, 3.14. It shows 3.14 is moved to ft0 via t0.

```asm
    lui $2, $4
    addi $2, $2, $3
    fmv.d.x $1, $2
```

## multi imm

multiple imm. the sample code is muli t0, t1, 32

```asm
    li $2, $3
    mul $1, $1, $2
```

## divi imm

divide imm. the sample code is divi t0, t1, 3

```asm
    li $2, $3
    div $1, $1, $2
```

## assert value

trigger exception / exit from code

```asm
    li t0, $1
    exit t0
```

## push by

perform push stack operation by using a register provided. The register content will be decreased after each operation

```asm
    addi $1, $1, -4
    sw $2, $1, 0

```

## pop by

pop int32 from memory by using a non-sp register. The register content will be increased after each operation

```asm
    lw $2, $1, 0
    addi $1, $1, 4

```

## inc

increase the register value by a const, this constant is between -2047 to 2048. 

```asm
    addi $1, $1, $2
```
