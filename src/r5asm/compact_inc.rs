use super::asm_error::AsmError;

use super::{instruction::Instruction, machinecode::MachineCode, opcode::OpCode, r5asm_pest::InstructionRegisterName, register::Register, reverse_string};

pub fn get_machine_code_from_compact_inc(inc:&Instruction, inc_offset:usize, regs:&Register) -> Result<MachineCode, AsmError> {
    if !inc.get_op_code()?.is_compact_inc() {
        return Err(AsmError::ParameterError((file!(), line!()).into()));
    }

    let op_code = inc.get_op_code()?;
    let value = op_code.get_value();
    let value_str = format!("{value:0>2b}");
    match op_code {
        OpCode::Clwsp => {
            let funct3 = format!("{:0>3b}", op_code.get_funct3().unwrap());
            let imm_value_str = reverse_string(&format!("{:0>10b}", inc.imm_to_u32()?));
            let rd = inc.get_register_id_as_string(InstructionRegisterName::Rd, regs)?;
            let imm_5 = imm_value_str.chars().nth(5).unwrap();
            let imm_4_2 = imm_value_str[2..4].to_string();
            let machine_code_str = format!("{funct3}{imm_5}{rd}{imm_4_2}{imm_7_6}{value_str}", imm_7_6 = imm_value_str[6..7].to_string());
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)
        }
        OpCode::Cldsp => {
            let funct3 = format!("{:0>3b}", op_code.get_funct3().unwrap());
            let imm_value_str = reverse_string(&format!("{:0>10b}", inc.imm_to_u32()?));
            let rd = inc.get_register_id_as_string(InstructionRegisterName::Rd, regs)?;
            let imm_5 = imm_value_str.chars().nth(5).unwrap();
            let imm_4_3 = imm_value_str[3..4].to_string();
            let machine_code_str = format!("{funct3}{imm_5}{rd}{imm_4_3}{imm_8_6}{value_str}", imm_8_6 = imm_value_str[6..8].to_string());
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)
        }
        OpCode::Clqsp => {
            let funct3 = format!("{:0>3b}", op_code.get_funct3().unwrap());
            let imm_value_str = reverse_string(&format!("{:0>10b}", inc.imm_to_u32()?));
            let rd = inc.get_register_id_as_string(InstructionRegisterName::Rd, regs)?;
            let imm_5 = imm_value_str.chars().nth(5).unwrap();
            let imm_4_4 = imm_value_str[4..4].to_string();
            let machine_code_str = format!("{funct3}{imm_5}{rd}{imm_4_4}{imm_9_6}{value_str}", imm_9_6 = imm_value_str[6..9].to_string());
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)
        }
        OpCode::Cflwsp => {
            let funct3 = format!("{:0>3b}", op_code.get_funct3().unwrap());
            let imm_value_str = reverse_string(&format!("{:0>10b}", inc.imm_to_u32()?));
            let rd = inc.get_register_id_as_string(InstructionRegisterName::Rd, regs)?;
            let imm_5 = imm_value_str.chars().nth(5).unwrap();
            let imm_4_2 = imm_value_str[2..4].to_string();
            let machine_code_str = format!("{funct3}{imm_5}{rd}{imm_4_2}{imm_7_6}{value_str}", imm_7_6 = imm_value_str[6..7].to_string());
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)
        }
        OpCode::Cfldsp => {
            let funct3 = format!("{:0>3b}", op_code.get_funct3().unwrap());
            let imm_value_str = reverse_string(&format!("{:0>10b}", inc.imm_to_u32()?));
            let rd = inc.get_register_id_as_string(InstructionRegisterName::Rd, regs)?;
            let imm_5 = imm_value_str.chars().nth(5).unwrap();
            let imm_4_3 = imm_value_str[3..4].to_string();
            let machine_code_str = format!("{funct3}{imm_5}{rd}{imm_4_3}{imm_8_6}{value_str}", imm_8_6 = imm_value_str[6..8].to_string());
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)
        }
        OpCode::Cfswsp |
        OpCode::Cswsp => {
            let funct3 = format!("{:0>3b}", op_code.get_funct3().unwrap());
            let imm_value_str = reverse_string(&format!("{:0>10b}", inc.imm_to_u32()?));
            let rd = inc.get_register_id_as_string(InstructionRegisterName::Rd, regs)?;
            let imm_5_2 = imm_value_str[2..5].to_string();
            let machine_code_str = format!("{funct3}{imm_5_2}{imm_7_6}{rd}{value_str}", imm_7_6 = imm_value_str[6..7].to_string());
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)
        }
        OpCode::Cfsdsp |
        OpCode::Csdsp => {
            let funct3 = format!("{:0>3b}", op_code.get_funct3().unwrap());
            let imm_value_str = reverse_string(&format!("{:0>10b}", inc.imm_to_u32()?));
            let rd = inc.get_register_id_as_string(InstructionRegisterName::Rd, regs)?;
            let imm_5_3 = imm_value_str[3..5].to_string();
            let machine_code_str = format!("{funct3}{imm_5_3}{imm_8_6}{rd}{value_str}", imm_8_6 = imm_value_str[6..8].to_string());
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)
        }
        OpCode::Csqsp => {
            let funct3 = format!("{:0>3b}", op_code.get_funct3().unwrap());
            let imm_value_str = reverse_string(&format!("{:0>10b}", inc.imm_to_u32()?));
            let rd = inc.get_register_id_as_string(InstructionRegisterName::Rd, regs)?;
            let imm_5_4 = imm_value_str[4..5].to_string();
            let machine_code_str = format!("{funct3}{imm_5_4}{imm_9_6}{rd}{value_str}", imm_9_6 = imm_value_str[6..9].to_string());
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)
        }
        OpCode::Cflw |
        OpCode::Clw => {
            let funct3 = format!("{:0>3b}", op_code.get_funct3().unwrap());
            let imm_value_str = reverse_string(&format!("{:0>10b}", inc.imm_to_u32()?));
            let rd = inc.get_compact_register_id_as_string(InstructionRegisterName::Rd, regs)?;
            let r1 = inc.get_compact_register_id_as_string(InstructionRegisterName::Rs1, regs)?; 
            let imm_5_3 = imm_value_str[3..5].to_string();
            let imm2 = format!("{}{}", imm_value_str.chars().nth(2).unwrap(), imm_value_str.chars().nth(6).unwrap());
            let machine_code_str = format!("{funct3}{imm_5_3}{r1}{imm2}{rd}{value_str}");
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)
        }
        OpCode::Cfld |
        OpCode::Cld => {
            let funct3 = format!("{:0>3b}", op_code.get_funct3().unwrap());
            let imm_value_str = reverse_string(&format!("{:0>10b}", inc.imm_to_u32()?));
            let rd = inc.get_compact_register_id_as_string(InstructionRegisterName::Rd, regs)?;
            let r1 = inc.get_compact_register_id_as_string(InstructionRegisterName::Rs1, regs)?; 
            let imm_5_3 = imm_value_str[3..5].to_string();
            let imm2 = format!("{}{}", imm_value_str.chars().nth(7).unwrap(), imm_value_str.chars().nth(6).unwrap());
            let machine_code_str = format!("{funct3}{imm_5_3}{r1}{imm2}{rd}{value_str}");
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)
        }
        OpCode::Clq => {
            let funct3 = format!("{:0>3b}", op_code.get_funct3().unwrap());
            let imm_value_str = reverse_string(&format!("{:0>10b}", inc.imm_to_u32()?));
            let rd = inc.get_compact_register_id_as_string(InstructionRegisterName::Rd, regs)?;
            let r1 = inc.get_compact_register_id_as_string(InstructionRegisterName::Rs1, regs)?; 
            let imm3 = format!("{}{}{}", imm_value_str.chars().nth(5).unwrap(), imm_value_str.chars().nth(4).unwrap(), imm_value_str.chars().nth(8).unwrap());
            let imm2 = format!("{}{}", imm_value_str.chars().nth(7).unwrap(), imm_value_str.chars().nth(6).unwrap());
            let machine_code_str = format!("{funct3}{imm3}{r1}{imm2}{rd}{value_str}");
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)            
        }
        OpCode::Cfsw |
        OpCode::Csw => {
            let funct3 = format!("{:0>3b}", op_code.get_funct3().unwrap());
            let imm_value_str = reverse_string(&format!("{:0>10b}", inc.imm_to_u32()?));
            let rd = inc.get_compact_register_id_as_string(InstructionRegisterName::Rd, regs)?;
            let r1 = inc.get_compact_register_id_as_string(InstructionRegisterName::Rs1, regs)?; 
            let imm_5_3 = imm_value_str[3..5].to_string();
            let imm2 = format!("{}{}", imm_value_str.chars().nth(2).unwrap(), imm_value_str.chars().nth(6).unwrap());
            let machine_code_str = format!("{funct3}{imm_5_3}{r1}{imm2}{rd}{value_str}");
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)
        }
        OpCode::Cfsd |
        OpCode::Csd => {
            let funct3 = format!("{:0>3b}", op_code.get_funct3().unwrap());
            let imm_value_str = reverse_string(&format!("{:0>10b}", inc.imm_to_u32()?));
            let rd = inc.get_compact_register_id_as_string(InstructionRegisterName::Rd, regs)?;
            let r1 = inc.get_compact_register_id_as_string(InstructionRegisterName::Rs1, regs)?; 
            let imm_5_3 = imm_value_str[3..5].to_string();
            let imm2 = format!("{}{}", imm_value_str.chars().nth(7).unwrap(), imm_value_str.chars().nth(6).unwrap());
            let machine_code_str = format!("{funct3}{imm_5_3}{r1}{imm2}{rd}{value_str}");
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)
        }
        OpCode::Csq => {
            let funct3 = format!("{:0>3b}", op_code.get_funct3().unwrap());
            let imm_value_str = reverse_string(&format!("{:0>10b}", inc.imm_to_u32()?));
            let rd = inc.get_compact_register_id_as_string(InstructionRegisterName::Rd, regs)?;
            let r1 = inc.get_compact_register_id_as_string(InstructionRegisterName::Rs1, regs)?; 
            let imm3 = format!("{}{}{}", imm_value_str.chars().nth(5).unwrap(), imm_value_str.chars().nth(4).unwrap(), imm_value_str.chars().nth(8).unwrap());
            let imm2 = format!("{}{}", imm_value_str.chars().nth(7).unwrap(), imm_value_str.chars().nth(6).unwrap());
            let machine_code_str = format!("{funct3}{imm3}{r1}{imm2}{rd}{value_str}");
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)
        }
        OpCode::Cjal |
        OpCode::Cj => {
            let funct4 = format!("{:0>4b}", op_code.get_funct4().unwrap());
            let imm_value_str = reverse_string(&format!("{:0>11b}", inc.imm_to_u32()?));
            let imm11 = imm_value_str.chars().nth(11).unwrap();
            let imm4 = imm_value_str.chars().nth(4).unwrap();
            let imm_9_8 = format!("{}", imm_value_str[8..9].to_string());
            let imm10 = imm_value_str.chars().nth(10).unwrap();
            let imm6 = imm_value_str.chars().nth(6).unwrap();
            let imm7 = imm_value_str.chars().nth(7).unwrap();
            let imm_3_1 = format!("{}", imm_value_str[1..3].to_string());
            let imm5 = imm_value_str.chars().nth(5).unwrap();
            let shuffled_imm_value_str = format!("{imm11}{imm4}{imm_9_8}{imm10}{imm6}{imm7}{imm_3_1}{imm5}");
            let machine_code_str = format!("{funct4}{shuffled_imm_value_str}{value_str}");
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)
        }
        OpCode::Cjalr |
        OpCode::Cjr => {
            let funct3 = format!("{:0>3b}", op_code.get_funct3().unwrap());
            let rd = inc.get_register_id_as_string(InstructionRegisterName::Rd, regs)?;
            let r1 = "00000"; 
            let machine_code_str = format!("{funct3}{rd}{r1}{value_str}");
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)
        }
        OpCode::Cbnez |
        OpCode::Cbeqz => {
            let funct3 = format!("{:0>3b}", op_code.get_funct3().unwrap());
            let imm_value_str = reverse_string(&format!("{:0>10b}", inc.imm_to_u32()?));
            let rd = inc.get_compact_register_id_as_string(InstructionRegisterName::Rd, regs)?;
            let imm3 = format!("{}{}{}", imm_value_str.chars().nth(8).unwrap(), imm_value_str.chars().nth(4).unwrap(), imm_value_str.chars().nth(3).unwrap());
            let imm5 = format!("{}{}{}", imm_value_str[6..7].to_string(), imm_value_str[1..2].to_string(), imm_value_str.chars().nth(5).unwrap());
            let machine_code_str = format!("{funct3}{imm3}{rd}{imm5}{value_str}");
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)
        }
        OpCode::Cli => {
            let funct3 = format!("{:0>3b}", op_code.get_funct3().unwrap());
            let imm_value_str = reverse_string(&format!("{:0>10b}", inc.imm_to_u32()?));
            let rd = inc.get_compact_register_id_as_string(InstructionRegisterName::Rd, regs)?;
            let imm = format!("{}", imm_value_str.chars().nth(5).unwrap());
            let imm5 = format!("{}", imm_value_str[0..4].to_string());
            let machine_code_str = format!("{funct3}{imm}{rd}{imm5}{value_str}");
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)
        }
        OpCode::Clui => {
            let funct3 = format!("{:0>3b}", op_code.get_funct3().unwrap());
            let imm_value_str = reverse_string(&format!("{:0>20b}", inc.imm_to_u32()?));
            let rd = inc.get_compact_register_id_as_string(InstructionRegisterName::Rd, regs)?;
            let imm = format!("{}", imm_value_str.chars().nth(17).unwrap());
            let imm5 = format!("{}", imm_value_str[12..16].to_string());
            let machine_code_str = format!("{funct3}{imm}{rd}{imm5}{value_str}");
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)
        }
        OpCode::Caddiw |
        OpCode::Caddi => {
            let funct3 = format!("{:0>3b}", op_code.get_funct3().unwrap());
            let imm_value_str = reverse_string(&format!("{:0>10b}", inc.imm_to_u32()?));
            let rd = inc.get_compact_register_id_as_string(InstructionRegisterName::Rd, regs)?;
            let imm = format!("{}", imm_value_str.chars().nth(5).unwrap());
            let imm5 = format!("{}", imm_value_str[0..4].to_string());
            let machine_code_str = format!("{funct3}{imm}{rd}{imm5}{value_str}");
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)
        }
        OpCode::Caddi16sp => {
            let funct3 = format!("{:0>3b}", op_code.get_funct3().unwrap());
            let imm_value_str = reverse_string(&format!("{:0>10b}", inc.imm_to_u32()?));
            let rd = format!("{:0>5b}", 2);
            let imm = format!("{}", imm_value_str.chars().nth(9).unwrap());
            let imm5 = format!("{}{}{}{}", imm_value_str.chars().nth(4).unwrap(), imm_value_str.chars().nth(6).unwrap(), imm_value_str[7..8].to_string(), imm_value_str.chars().nth(5).unwrap());
            let machine_code_str = format!("{funct3}{imm}{rd}{imm5}{value_str}");
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)
        }
        OpCode::Caddi4spn => {
            let funct3 = format!("{:0>3b}", op_code.get_funct3().unwrap());
            let imm_value_str = reverse_string(&format!("{:0>10b}", inc.imm_to_u32()?));
            let rd = inc.get_compact_register_id_as_string(InstructionRegisterName::Rd, regs)?;
            let imm8 = format!("{}{}{}{}", imm_value_str[4..5].to_string(), imm_value_str[6..9].to_string(), imm_value_str.chars().nth(2).unwrap(), imm_value_str.chars().nth(3).unwrap());
            let machine_code_str = format!("{funct3}{imm8}{rd}{value_str}");
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)
        }
        OpCode::Cslli => {
            let funct3 = format!("{:0>3b}", op_code.get_funct3().unwrap());
            let imm_value_str = reverse_string(&format!("{:0>10b}", inc.imm_to_u32()?));
            let rd = inc.get_register_id_as_string(InstructionRegisterName::Rd, regs)?;
            let imm1 = format!("{}", imm_value_str.chars().nth(5).unwrap());
            let imm5 = format!("{}", imm_value_str[0..4].to_string());
            let machine_code_str = format!("{funct3}{imm1}{rd}{imm5}{value_str}");
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)
        }
        OpCode::Srli |
        OpCode::Srai |
        OpCode::Candi => {
            let funct3 = format!("{:0>3b}", op_code.get_funct3().unwrap());
            let imm_value_str = reverse_string(&format!("{:0>10b}", inc.imm_to_u32()?));
            let rd = inc.get_compact_register_id_as_string(InstructionRegisterName::Rd, regs)?;
            let funct2 = format!("{:0>2b}", op_code.get_funct2().unwrap());
            let imm1 = format!("{}", imm_value_str.chars().nth(5).unwrap());
            let imm5 = format!("{}", imm_value_str[0..4].to_string());
            let machine_code_str = format!("{funct3}{imm1}{funct2}{rd}{imm5}{value_str}");
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)
        }
        OpCode::Cmv |
        OpCode::Cadd => {
            let funct4 = format!("{:0>4b}", op_code.get_funct4().unwrap());
            let rd = inc.get_register_id_as_string(InstructionRegisterName::Rd, regs)?;
            let rs2 = inc.get_register_id_as_string(InstructionRegisterName::Rs1, regs)?;
            let machine_code_str = format!("{funct4}{rd}{rs2}{value_str}");
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)
        }
        OpCode::Cand |
        OpCode::Cor |
        OpCode::Cxor |
        OpCode::Csub |
        OpCode::Caddw |
        OpCode::Csubw => {
            let funct6 = format!("{:0>6b}", op_code.get_funct6().unwrap());
            let rd = inc.get_compact_register_id_as_string(InstructionRegisterName::Rd, regs)?;
            let rs2 = inc.get_compact_register_id_as_string(InstructionRegisterName::Rs1, regs)?;
            let funct2 = format!("{:0>2b}", op_code.get_funct2().unwrap());
            let machine_code_str = format!("{funct6}{rd}{rs2}{funct2}{value_str}");
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)
        }
        OpCode::Cnop => {
            let funct3 = format!("{:0>3b}", op_code.get_funct3().unwrap());
            let padding11 = format!("{:0>11b}", 0);
            let machine_code_str = format!("{funct3}{padding11}{value_str}");
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)
        }
        OpCode::Cebreak => {
            let funct4 = format!("{:0>4b}", op_code.get_funct4().unwrap());
            let padding10 = format!("{:0>10b}", 0);
            let machine_code_str = format!("{funct4}{padding10}{value_str}");
            let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
            Ok(r)
        }
        _ => {
            Err(AsmError::NoFound((file!(), line!()).into(), format!("required to process compact inc: {op_code:?}")))
        }
    }
}