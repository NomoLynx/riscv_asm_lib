use super::vector_inc::{VMaskOp, VRedOp, ValueOp, ValueSrc, VectorInc};
use super::vector_pest::{base_vector_mnemonic, parse_value_form, parse_vload_kind, parse_vstore_kind, parse_vm_from_option, parse_vwidth};
use crate::r5asm::asm_error::AsmError;
use crate::r5asm::instruction::Instruction;
use crate::r5asm::machinecode::MachineCode;
use crate::r5asm::r5asm_pest::InstructionRegisterName;
use crate::r5asm::register::Register;

fn parse_reg_id(inc: &Instruction, regs: &Register, reg: InstructionRegisterName) -> Result<u8, AsmError> {
    let bits = inc.get_register_id_as_string(reg, regs)?;
    u8::from_str_radix(&bits, 2)
        .map_err(|_| AsmError::ConversionFailed((file!(), line!()).into(), format!("cannot convert register bits '{bits}' to u8")))
}

fn value_op_from_abstract_name(name: &str) -> Result<ValueOp, AsmError> {
    match name.to_lowercase().as_str() {
        "add" => Ok(ValueOp::Add),
        "sub" => Ok(ValueOp::Sub),
        "mul" => Ok(ValueOp::Mul),
        "div" => Ok(ValueOp::Div),
        "divu" => Ok(ValueOp::Divu),
        "rem" => Ok(ValueOp::Rem),
        "remu" => Ok(ValueOp::Remu),
        "min" => Ok(ValueOp::Min),
        "minu" => Ok(ValueOp::Minu),
        "max" => Ok(ValueOp::Max),
        "maxu" => Ok(ValueOp::Maxu),
        "and" => Ok(ValueOp::And),
        "or" => Ok(ValueOp::Or),
        "xor" => Ok(ValueOp::Xor),
        "sll" => Ok(ValueOp::Sll),
        "srl" => Ok(ValueOp::Srl),
        "sra" => Ok(ValueOp::Sra),
        "seq" => Ok(ValueOp::Seq),
        "slt" => Ok(ValueOp::Slt),
        "sle" => Ok(ValueOp::Sle),
        "sltu" => Ok(ValueOp::Sltu),
        "sgtu" => Ok(ValueOp::Sgtu),
        "madd" => Ok(ValueOp::Madd),
        "nmsub" => Ok(ValueOp::Nmsub),
        "macc" => Ok(ValueOp::Macc),
        "nmacc" => Ok(ValueOp::Nmacc),
        "merge" => Ok(ValueOp::Merge),
        "move" => Ok(ValueOp::Move),
        _ => Err(AsmError::NoFound((file!(), line!()).into(), format!("unsupported abstract vector op: {name}"))),
    }
}

fn reduction_op_from_mnemonic(name: &str) -> Result<VRedOp, AsmError> {
    match name {
        "vredsum" => Ok(VRedOp::Sum),
        "vredmin" => Ok(VRedOp::Min),
        "vredmax" => Ok(VRedOp::Max),
        _ => Err(AsmError::NoFound((file!(), line!()).into(), format!("unsupported vector reduction op: {name}"))),
    }
}

fn mask_op_from_mnemonic(name: &str) -> Result<VMaskOp, AsmError> {
    match name {
        "vand" => Ok(VMaskOp::And),
        "vor" => Ok(VMaskOp::Or),
        "vxor" => Ok(VMaskOp::Xor),
        "vnot" => Ok(VMaskOp::Not),
        _ => Err(AsmError::NoFound((file!(), line!()).into(), format!("unsupported vector mask op: {name}"))),
    }
}

fn to_machine_code(vector_inc: VectorInc, offset: usize) -> MachineCode {
    let bytes = vector_inc.to_le_bytes();
    let raw = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    MachineCode::new(raw, offset)
}

pub fn emit_vector_instruction(inc: &Instruction, regs: &Register, offset: usize) -> Result<Vec<MachineCode>, AsmError> {
    let name = inc.get_name().to_lowercase();
    let base = base_vector_mnemonic(&name);
    let vm = parse_vm_from_option(inc.option.as_deref());

    // Reduction instructions: vred*.vs vd, vs2, vs1[, v0.t]
    if name.ends_with(".vs") {
        let op = reduction_op_from_mnemonic(base)?;
        let vd = parse_reg_id(inc, regs, InstructionRegisterName::Rd)?;
        let vs2 = parse_reg_id(inc, regs, InstructionRegisterName::Rs1)?;
        let vs1 = parse_reg_id(inc, regs, InstructionRegisterName::Rs2)?;
        let encoded = VectorInc::encode_vreduction(op, vd, vs2, vs1, vm);
        return Ok(vec![to_machine_code(encoded, offset)]);
    }

    // Mask instructions: vand.mm / vor.mm / vxor.mm / vnot.m
    if name.ends_with(".mm") || name.ends_with(".m") {
        let op = mask_op_from_mnemonic(base)?;
        let vd = parse_reg_id(inc, regs, InstructionRegisterName::Rd)?;
        let vs1 = parse_reg_id(inc, regs, InstructionRegisterName::Rs1)?;
        let vs2 = if name.starts_with("vnot") {
            None
        } else {
            Some(parse_reg_id(inc, regs, InstructionRegisterName::Rs2)?)
        };
        let encoded = VectorInc::encode_vmask(op, vd, vs1, vs2);
        return Ok(vec![to_machine_code(encoded, offset)]);
    }

    // Load instructions
    if base.starts_with("vl") {
        let kind = parse_vload_kind(base)
            .ok_or(AsmError::NoFound((file!(), line!()).into(), format!("cannot infer vload kind from '{base}'")))?;
        let width = parse_vwidth(base)
            .ok_or(AsmError::NoFound((file!(), line!()).into(), format!("cannot infer vload width from '{base}'")))?;
        let vd = parse_reg_id(inc, regs, InstructionRegisterName::Rd)?;
        let base_reg = parse_reg_id(inc, regs, InstructionRegisterName::Rs1)?;
        let index_or_stride = if inc.has_reg(InstructionRegisterName::Rs2) {
            Some(parse_reg_id(inc, regs, InstructionRegisterName::Rs2)?)
        } else {
            None
        };
        let encoded = VectorInc::encode_vload(kind, width, vd, base_reg, index_or_stride);
        return Ok(vec![to_machine_code(encoded, offset)]);
    }

    // Store instructions
    if base.starts_with("vs") {
        let kind = parse_vstore_kind(base)
            .ok_or(AsmError::NoFound((file!(), line!()).into(), format!("cannot infer vstore kind from '{base}'")))?;
        let width = parse_vwidth(base)
            .ok_or(AsmError::NoFound((file!(), line!()).into(), format!("cannot infer vstore width from '{base}'")))?;
        let vs3 = parse_reg_id(inc, regs, InstructionRegisterName::Rd)?;
        let base_reg = parse_reg_id(inc, regs, InstructionRegisterName::Rs1)?;
        let index_or_stride = if inc.has_reg(InstructionRegisterName::Rs2) {
            Some(parse_reg_id(inc, regs, InstructionRegisterName::Rs2)?)
        } else {
            None
        };
        let encoded = VectorInc::encode_vstore(kind, width, vs3, base_reg, index_or_stride);
        return Ok(vec![to_machine_code(encoded, offset)]);
    }

    // Value instructions: *.vv, *.vx, *.vi
    let form = parse_value_form(&name)
        .ok_or(AsmError::NoFound((file!(), line!()).into(), format!("cannot infer vector value form from '{name}'")))?;
    let abstract_name = if base == "vmv" {
        "move"
    } else if base == "vmerge" {
        "merge"
    } else if base.starts_with("vmseq") {
        "seq"
    } else if base.starts_with("vmsltu") {
        "sltu"
    } else if base.starts_with("vmslt") {
        "slt"
    } else if base.starts_with("vmsle") {
        "sle"
    } else if let Some(trimmed) = base.strip_prefix('v') {
        trimmed
    } else {
        base
    };

    let op = value_op_from_abstract_name(abstract_name)?;
    let vd = parse_reg_id(inc, regs, InstructionRegisterName::Rd)?;
    let vs2 = parse_reg_id(inc, regs, InstructionRegisterName::Rs1)?;

    let src = match form {
        super::vector_inc::ValueForm::VV => ValueSrc::Vs1(parse_reg_id(inc, regs, InstructionRegisterName::Rs2)?),
        super::vector_inc::ValueForm::VX => ValueSrc::Rs1(parse_reg_id(inc, regs, InstructionRegisterName::Rs2)?),
        super::vector_inc::ValueForm::VI => {
            let imm = inc.get_imm_value_from_imm_string()
                .ok_or(AsmError::ConversionFailed((file!(), line!()).into(), format!("cannot parse vector immediate for '{name}'")))? as i8;
            ValueSrc::Imm(imm)
        }
    };

    let encoded = VectorInc::encode_value(op, form, vd, vs2, src, vm);
    Ok(vec![to_machine_code(encoded, offset)])
}
