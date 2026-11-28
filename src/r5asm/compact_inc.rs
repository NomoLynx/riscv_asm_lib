use super::{asm_error::AsmError, instruction::Instruction, machinecode::MachineCode,
    opcode::OpCode, r5asm_pest::InstructionRegisterName, register::Register, reverse_string};

const CI_BITS: &[(u8, u8)] = &[(5, 12), (4, 6), (3, 5), (2, 4), (1, 3), (0, 2)];
const LW_BITS: &[(u8, u8)] = &[(5, 12), (4, 11), (3, 10), (2, 6), (6, 5)];
const LD_BITS: &[(u8, u8)] = &[(5, 12), (4, 11), (3, 10), (7, 6), (6, 5)];
const LWSP_BITS: &[(u8, u8)] = &[(5, 12), (4, 6), (3, 5), (2, 4), (7, 3), (6, 2)];
const LDSP_BITS: &[(u8, u8)] = &[(5, 12), (4, 6), (3, 5), (8, 4), (7, 3), (6, 2)];
const SWSP_BITS: &[(u8, u8)] = &[(5, 12), (4, 11), (3, 10), (2, 9), (7, 8), (6, 7)];
const SDSP_BITS: &[(u8, u8)] = &[(5, 12), (4, 11), (3, 10), (8, 9), (7, 8), (6, 7)];
const CJ_BITS: &[(u8, u8)] = &[(11, 12), (4, 11), (9, 10), (8, 9), (10, 8),
    (6, 7), (7, 6), (3, 5), (2, 4), (1, 3), (5, 2)];
const CB_BITS: &[(u8, u8)] = &[(8, 12), (4, 11), (3, 10), (7, 6), (6, 5), (2, 4), (1, 3), (5, 2)];
const ADDI16SP_BITS: &[(u8, u8)] = &[(9, 12), (4, 6), (6, 5), (8, 4), (7, 3), (5, 2)];
const ADDI4SPN_BITS: &[(u8, u8)] = &[(5, 12), (4, 11), (9, 10), (8, 9), (7, 8), (6, 7), (2, 6), (3, 5)];

fn invalid(inc:&Instruction, message:&str) -> AsmError {
    AsmError::GeneralError((file!(), line!()).into(), format!("{}: {message}", inc.name))
}

fn require(inc:&Instruction, valid:bool, message:&str) -> Result<(), AsmError> {
    if valid { Ok(()) } else { Err(invalid(inc, message)) }
}

fn register(inc:&Instruction, name:Option<&String>, regs:&Register, floating:bool) -> Result<u16, AsmError> {
    let name = name.ok_or_else(|| invalid(inc, "missing register operand"))?;
    let valid = if floating { Register::is_float_register_name(name) }
        else { Register::is_integer_register_name(name) };
    require(inc, valid, "wrong register class")?;
    let id = regs.get_register_value(Some(name))?;
    require(inc, (0..=31).contains(&id), "register ID must be in 0..31")?;
    Ok(id as u16)
}

fn prime(inc:&Instruction, id:u16) -> Result<u16, AsmError> {
    require(inc, (8..=15).contains(&id), "compact register ID must be in 8..15")?;
    Ok(id - 8)
}

fn immediate(inc:&Instruction, min:i64, max:i64, scale:i64) -> Result<i64, AsmError> {
    require(inc, inc.get_imm().is_some(), "missing immediate operand")?;
    let value = inc.imm_to_i64()?;
    require(inc, (min..=max).contains(&value) && value % scale == 0, "immediate is out of range or misaligned")?;
    Ok(value)
}

fn stack_base(inc:&Instruction, regs:&Register) -> Result<(), AsmError> {
    if inc.r1_name.is_some() {
        require(inc, register(inc, inc.r1_name.as_ref(), regs, false)? == 2, "stack base must be sp/x2")?;
    }
    Ok(())
}

fn scatter(value:i64, bits:&[(u8, u8)]) -> u16 {
    let value = value as u16;
    bits.iter().fold(0, |word, &(source, target)| word | (((value >> source) & 1) << target))
}

pub (super) fn lui_immediate(value:i64) -> Option<i64> {
    let field = if (0..=0xfffff).contains(&value) && value & 0x80000 != 0 {
        value - 0x100000
    } else {
        value
    };
    (field != 0 && (-32..=31).contains(&field)).then_some(field)
}

pub fn get_machine_code_from_compact_inc(inc:&Instruction, inc_offset:usize, regs:&Register, build_target:u8) -> Result<MachineCode, AsmError> {
    let op = inc.get_op_code()?;
    require(inc, op.is_compact_inc(), "not a compressed opcode")?;
    if matches!(op, OpCode::Clq | OpCode::Clqsp | OpCode::Csq | OpCode::Csqsp) {
        return encode_legacy_quad(inc, inc_offset, regs);
    }
    require(inc, matches!(build_target, 4 | 8), "current C encoding requires RV32 or RV64")?;
    require(inc, !matches!(op, OpCode::Cld | OpCode::Cldsp | OpCode::Csd | OpCode::Csdsp
        | OpCode::Caddiw | OpCode::Caddw | OpCode::Csubw) || build_target == 8, "instruction is RV64-only")?;
    require(inc, !matches!(op, OpCode::Cflw | OpCode::Cflwsp | OpCode::Cfsw | OpCode::Cfswsp | OpCode::Cjal)
        || build_target == 4, "instruction is RV32-only")?;
    let opcode = op.get_value() as u16;
    let f3 = || op.get_funct3().map(|value| (value as u16) << 13)
        .ok_or_else(|| invalid(inc, "missing funct3 metadata"));
    let f4 = || op.get_funct4().map(|value| (value as u16) << 12)
        .ok_or_else(|| invalid(inc, "missing funct4 metadata"));
    let f2 = || op.get_funct2().map(|value| value as u16)
        .ok_or_else(|| invalid(inc, "missing funct2 metadata"));
    let word = match op {
        OpCode::Clw | OpCode::Cld | OpCode::Cflw | OpCode::Cfld
        | OpCode::Csw | OpCode::Csd | OpCode::Cfsw | OpCode::Cfsd => {
            let floating = matches!(op, OpCode::Cflw | OpCode::Cfld | OpCode::Cfsw | OpCode::Cfsd);
            let double = matches!(op, OpCode::Cld | OpCode::Cfld | OpCode::Csd | OpCode::Cfsd);
            let data = prime(inc, register(inc, inc.r0_name.as_ref(), regs, floating)?)?;
            let base = prime(inc, register(inc, inc.r1_name.as_ref(), regs, false)?)?;
            let scale = if double { 8 } else { 4 };
            let imm = immediate(inc, 0, 31 * scale, scale)?;
            f3()? | scatter(imm, if double { LD_BITS } else { LW_BITS }) | (base << 7) | (data << 2) | opcode
        }
        OpCode::Clwsp | OpCode::Cldsp | OpCode::Cflwsp | OpCode::Cfldsp => {
            let floating = matches!(op, OpCode::Cflwsp | OpCode::Cfldsp);
            let double = matches!(op, OpCode::Cldsp | OpCode::Cfldsp);
            let data = register(inc, inc.r0_name.as_ref(), regs, floating)?;
            require(inc, floating || data != 0, "integer stack-load destination cannot be x0")?;
            stack_base(inc, regs)?;
            let scale = if double { 8 } else { 4 };
            let imm = immediate(inc, 0, 63 * scale, scale)?;
            f3()? | scatter(imm, if double { LDSP_BITS } else { LWSP_BITS }) | (data << 7) | opcode
        }
        OpCode::Cswsp | OpCode::Csdsp | OpCode::Cfswsp | OpCode::Cfsdsp => {
            let floating = matches!(op, OpCode::Cfswsp | OpCode::Cfsdsp);
            let double = matches!(op, OpCode::Csdsp | OpCode::Cfsdsp);
            let data = register(inc, inc.r0_name.as_ref(), regs, floating)?;
            stack_base(inc, regs)?;
            let scale = if double { 8 } else { 4 };
            let imm = immediate(inc, 0, 63 * scale, scale)?;
            f3()? | scatter(imm, if double { SDSP_BITS } else { SWSP_BITS }) | (data << 2) | opcode
        }
        OpCode::Cj | OpCode::Cjal =>
            f3()? | scatter(immediate(inc, -2048, 2046, 2)?, CJ_BITS) | opcode,
        OpCode::Cjr | OpCode::Cjalr => {
            let source = register(inc, inc.r0_name.as_ref(), regs, false)?;
            require(inc, source != 0, "jump source cannot be x0")?;
            f4()? | (source << 7) | opcode
        }
        OpCode::Cbeqz | OpCode::Cbnez => {
            let source = prime(inc, register(inc, inc.r0_name.as_ref(), regs, false)?)?;
            f3()? | scatter(immediate(inc, -256, 254, 2)?, CB_BITS) | (source << 7) | opcode
        }
        OpCode::Caddi | OpCode::Caddiw | OpCode::Cli => {
            let rd = register(inc, inc.r0_name.as_ref(), regs, false)?;
            require(inc, op != OpCode::Caddiw || rd != 0, "C.ADDIW destination cannot be x0")?;
            f3()? | scatter(immediate(inc, -32, 31, 1)?, CI_BITS) | (rd << 7) | opcode
        }
        OpCode::Clui => {
            let rd = register(inc, inc.r0_name.as_ref(), regs, false)?;
            require(inc, rd != 2, "C.LUI destination cannot be sp/x2")?;
            require(inc, inc.get_imm().is_some(), "missing immediate operand")?;
            let imm = lui_immediate(inc.imm_to_i64()?)
                .ok_or_else(|| invalid(inc, "C.LUI requires a nonzero signed six-bit field"))?;
            f3()? | scatter(imm, CI_BITS) | (rd << 7) | opcode
        }
        OpCode::Caddi16sp => {
            require(inc, register(inc, inc.r0_name.as_ref(), regs, false)? == 2, "C.ADDI16SP destination must be sp/x2")?;
            let imm = immediate(inc, -512, 496, 16)?;
            require(inc, imm != 0, "C.ADDI16SP immediate cannot be zero")?;
            f3()? | scatter(imm, ADDI16SP_BITS) | (2 << 7) | opcode
        }
        OpCode::Caddi4spn => {
            let rd = prime(inc, register(inc, inc.r0_name.as_ref(), regs, false)?)?;
            f3()? | scatter(immediate(inc, 4, 1020, 4)?, ADDI4SPN_BITS) | (rd << 2) | opcode
        }
        OpCode::Cslli | OpCode::Csrli | OpCode::Csrai => {
            let rd = register(inc, inc.r0_name.as_ref(), regs, false)?;
            let imm = immediate(inc, 0, i64::from(build_target) * 8 - 1, 1)?;
            let fields = if op == OpCode::Cslli { rd << 7 }
                else { (prime(inc, rd)? << 7) | (f2()? << 10) };
            f3()? | scatter(imm, CI_BITS) | fields | opcode
        }
        OpCode::Candi => {
            let rd = prime(inc, register(inc, inc.r0_name.as_ref(), regs, false)?)?;
            f3()? | scatter(immediate(inc, -32, 31, 1)?, CI_BITS) | (f2()? << 10) | (rd << 7) | opcode
        }
        OpCode::Cmv | OpCode::Cadd => {
            let rd = register(inc, inc.r0_name.as_ref(), regs, false)?;
            let source = register(inc, inc.r1_name.as_ref(), regs, false)?;
            require(inc, source != 0, "register source cannot be x0")?;
            f4()? | (rd << 7) | (source << 2) | opcode
        }
        OpCode::Cand | OpCode::Cor | OpCode::Cxor | OpCode::Csub | OpCode::Caddw | OpCode::Csubw => {
            let rd = prime(inc, register(inc, inc.r0_name.as_ref(), regs, false)?)?;
            let source = prime(inc, register(inc, inc.r1_name.as_ref(), regs, false)?)?;
            let f6 = op.get_funct6().ok_or_else(|| invalid(inc, "missing funct6 metadata"))? as u16;
            (f6 << 10) | (rd << 7) | (f2()? << 5) | (source << 2) | opcode
        }
        OpCode::Cnop => {
            let imm = if inc.get_imm().is_some() { immediate(inc, -32, 31, 1)? } else { 0 };
            f3()? | scatter(imm, CI_BITS) | opcode
        }
        OpCode::Cebreak => f4()? | opcode,
        _ => return Err(AsmError::NoFound((file!(), line!()).into(), format!("unsupported compact opcode: {op:?}"))),
    };
    Ok(MachineCode::new_u16(word, inc_offset))
}

// Preserve the pre-existing RV128-oriented paths outside the current C audit.
fn encode_legacy_quad(inc:&Instruction, inc_offset:usize, regs:&Register) -> Result<MachineCode, AsmError> {
    let op = inc.get_op_code()?;
    let value_str = format!("{:0>2b}", op.get_value());
    let funct3 = format!("{:0>3b}", op.get_funct3().unwrap());
    let imm_value_str = reverse_string(&format!("{:0>10b}", inc.imm_to_u32()?));
    let machine_code_str = match op {
        OpCode::Clqsp => {
            let rd = inc.get_register_id_as_string(InstructionRegisterName::Rd, regs)?;
            let imm_5 = imm_value_str.chars().nth(5).unwrap();
            let imm_4_4 = imm_value_str[4..4].to_string();
            format!("{funct3}{imm_5}{rd}{imm_4_4}{imm_9_6}{value_str}", imm_9_6 = imm_value_str[6..9].to_string())
        }
        OpCode::Csqsp => {
            let rd = inc.get_register_id_as_string(InstructionRegisterName::Rd, regs)?;
            let imm_5_4 = imm_value_str[4..5].to_string();
            format!("{funct3}{imm_5_4}{imm_9_6}{rd}{value_str}", imm_9_6 = imm_value_str[6..9].to_string())
        }
        OpCode::Clq | OpCode::Csq => {
            let rd = inc.get_compact_register_id_as_string(InstructionRegisterName::Rd, regs)?;
            let r1 = inc.get_compact_register_id_as_string(InstructionRegisterName::Rs1, regs)?;
            let imm3 = format!("{}{}{}", imm_value_str.chars().nth(5).unwrap(), imm_value_str.chars().nth(4).unwrap(), imm_value_str.chars().nth(8).unwrap());
            let imm2 = format!("{}{}", imm_value_str.chars().nth(7).unwrap(), imm_value_str.chars().nth(6).unwrap());
            format!("{funct3}{imm3}{r1}{imm2}{rd}{value_str}")
        }
        _ => return Err(invalid(inc, "not a legacy quad opcode")),
    };
    MachineCode::from_string(&machine_code_str, inc_offset)
}
