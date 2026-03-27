use rust_macro_internal::*;

use super::super::asm_error::AsmError;

use super::super::r5asm_pest::InstructionTypes;

#[csv2enum_lookup("src/r5asm/opcode/opcode.csv", opcode, 
    "src/r5asm/opcode/opcode_col.csv")]
#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {

}

impl OpCode {
    
    pub fn need_offset_compute_address(&self) -> bool {
        match self {
            Self::Auipc => true, 
            Self::Beq | 
            Self::Bne |
            Self::Blt | 
            Self::Bge |
            Self::Bltu |
            Self::Bgeu => true,
            _ => false,
        }
    }

    pub (crate) fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_uppercase().as_str() {
            "LUI" => Some(Self::Lui),
            "AUIPC" => Some(Self::Auipc),
            "ADDI" => Some(Self::Addi),
            "SLTI" => Some(Self::Slti),
            "SLTIU" => Some(Self::Sltiu),
            "XORI" => Some(Self::Xori),
            "ORI" => Some(Self::Ori),
            "ANDI" => Some(Self::Andi),
            "SLLI" => Some(Self::Slli),
            "SRLI" => Some(Self::Srli),
            "SRAI" => Some(Self::Srai),
            "ADD" => Some(Self::Add),
            "SUB" => Some(Self::Sub),
            "SLL" => Some(Self::Sll),
            "SLT" => Some(Self::Slt),
            "SLTU" => Some(Self::Sltu),
            "XOR" => Some(Self::Xor),
            "SRL" => Some(Self::Srl),
            "SRA" => Some(Self::Sra),
            "OR" => Some(Self::Or),
            "AND" => Some(Self::And),
            "FENCE" => Some(Self::Fence),
            "FENCEI" => Some(Self::Fencei),
            "CSRRW" => Some(Self::Csrrw),
            "CSRRS" => Some(Self::Csrrs),
            "CSRRC" => Some(Self::Csrrc),
            "CSRRWI" => Some(Self::Csrrwi),
            "CSRRSI" => Some(Self::Csrrsi),
            "CSRRCI" => Some(Self::Csrrci),
            "ECALL" => Some(Self::Ecall),
            "EBREAK" => Some(Self::Ebreak),
            "URET" => Some(Self::Uret),
            "SRET" => Some(Self::Sret),
            "MRET" => Some(Self::Mret),
            "WFI" => Some(Self::Wfi),
            "SFENCEVMA" => Some(Self::Sfencevma),
            "LB" => Some(Self::Lb),
            "LH" => Some(Self::Lh),
            "LW" => Some(Self::Lw),
            "LBU" => Some(Self::Lbu),
            "LHU" => Some(Self::Lhu),
            "SB" => Some(Self::Sb),
            "SH" => Some(Self::Sh),
            "SW" => Some(Self::Sw),
            "JAL" => Some(Self::Jal),
            "JALR" => Some(Self::Jalr),
            "BEQ" => Some(Self::Beq),
            "BNE" => Some(Self::Bne),
            "BLT" => Some(Self::Blt),
            "BGE" => Some(Self::Bge),
            "BLTU" => Some(Self::Bltu),
            "BGEU" => Some(Self::Bgeu),

            //RV32M extension ops
            "MUL" => Some(Self::Mul),
            "MULH" => Some(Self::Mulh),
            "MULHSU" => Some(Self::Mulhsu),
            "MULHU" => Some(Self::Mulhu),
            "DIV" => Some(Self::Div),
            "DIVU" => Some(Self::Divu),
            "REM" => Some(Self::Rem),
            "REMU" => Some(Self::Remu),

            //RV64I extension ops
            "ADDIW" => Some(Self::Addiw),
            "SLLIW" => Some(Self::Slliw),
            "SRAIW" => Some(Self::Sraiw),
            "SRLIW" => Some(Self::Srliw),
            "ADDW" => Some(Self::Addw),
            "SUBW" => Some(Self::Subw),
            "SLLW" => Some(Self::Sllw),
            "SRLW" => Some(Self::Srlw),
            "SRAW" => Some(Self::Sraw),
            "LWU" => Some(Self::Lwu),
            "LD" => Some(Self::Ld),
            "SD" => Some(Self::Sd),
            
            //RV64M extension ops
            "MULW" => Some(Self::Mulw),
            "DIVW" => Some(Self::Divw),
            "DIVUW" => Some(Self::Divuw),
            "REMW" => Some(Self::Remw),
            "REMUW" => Some(Self::Remuw),

            //RV64 Zba extension ops
            "ADD.UW" => Some(Self::Adduw),
            "SH1ADD" => Some(Self::Sh1add),
            "SH2ADD" => Some(Self::Sh2add),
            "SH3ADD" => Some(Self::Sh3add),
            "SLLI.UW" => Some(Self::Slliuw),
            "SH1ADD.UW" => Some(Self::Sh1adduw),
            "SH2ADD.UW" => Some(Self::Sh2adduw),
            "SH3ADD.UW" => Some(Self::Sh3adduw),

            //RV32/64 Zbs extension ops
            "BCLR" => Some(Self::Bclr),
            "BCLRI" => Some(Self::Bclri),
            "BEXT" => Some(Self::Bext),
            "BEXTI" => Some(Self::Bexti),
            "BINV" => Some(Self::Binv),
            "BINVI" => Some(Self::Binvi),
            "BSET" => Some(Self::Bset),
            "BSETI" => Some(Self::Bseti),

            //RV32F, 64D
            "FMADD.S" => Some(Self::Fmadds),
            "FMSUB.S" => Some(Self::Fmsubs),
            "FNMSUB.S" => Some(Self::Fnmsubs),
            "FNMADD.S" => Some(Self::Fnmadds),
            "FLW" => Some(Self::Flw),
            "FLD" => Some(Self::Fld),
            "FSW" => Some(Self::Fsw),
            "FSD" => Some(Self::Fsd),
            "FADD.S" => Some(Self::Fadds),
            "FSUB.S" => Some(Self::Fsubs),
            "FMUL.S" => Some(Self::Fmuls),
            "FDIV.S" => Some(Self::Fdivs),
            "FSQRT.S"=> Some(Self::Fsqrts),
            "FSGNJ.S" => Some(Self::Fsgnjs),
            "FSGNJN.S" => Some(Self::Fsgnjns),
            "FSGNJX.S" => Some(Self::Fsgnjxs),
            "FMIN.S" => Some(Self::Fmins),
            "FMAX.S" => Some(Self::Fmaxs),
            "FCVT.W.S" => Some(Self::Fcvtws),
            "FCVT.WU.S" => Some(Self::Fcvtwus),
            "FMV.X.S" |
            "FMV.X.W" => Some(Self::Fmvxw),
            "FEQ.S" => Some(Self::Feqs),
            "FLT.S" => Some(Self::Flts),
            "FLE.S" => Some(Self::Fles),
            "FCLASS.S" => Some(Self::Fclasss),
            "FCVT.S.W" => Some(Self::Fcvtsw),
            "FCVT.S.WU" => Some(Self::Fcvtswu),
            "FMV.S.X" |
            "FMV.W.X" => Some(Self::Fmvwx),
            "FMADD.D" => Some(Self::Fmaddd),
            "FMSUB.D" => Some(Self::Fmsubd),
            "FNMSUB.D" => Some(Self::Fnmsubd),
            "FNMADD.D" => Some(Self::Fnmaddd),
            "FADD.D" => Some(Self::Faddd),
            "FSUB.D" => Some(Self::Fsubd),
            "FMUL.D" => Some(Self::Fmuld),
            "FDIV.D" => Some(Self::Fdivd),
            "FSQRT.D" => Some(Self::Fsqrtd),
            "FSGNJ.D" => Some(Self::Fsgnjd),
            "FSGNJN.D" => Some(Self::Fsgnjnd),
            "FSGNJX.D" => Some(Self::Fsgnjxd),
            "FMIN.D" => Some(Self::Fmind),
            "FMAX.D" => Some(Self::Fmaxd),
            "FCVT.S.D" => Some(Self::Fcvtsd),
            "FCVT.D.S" => Some(Self::Fcvtds),
            "FEQ.D" => Some(Self::Feqd),
            "FLT.D" => Some(Self::Fltd),
            "FLE.D" => Some(Self::Fled),
            "FCLASS.D" => Some(Self::Fclassd),
            "FCVT.W.D" => Some(Self::Fcvtwd),
            "FCVT.WU.D" => Some(Self::Fcvtwud),
            "FCVT.D.W" => Some(Self::Fcvtdw),
            "FCVT.D.WU" => Some(Self::Fcvtdwu),
            "FCVT.L.S" => Some(Self::Fcvtls),
            "FCVT.LU.S" => Some(Self::Fcvtlus),
            "FCVT.S.L" => Some(Self::Fcvtsl),
            "FCVT.S.LU" => Some(Self::Fcvtslu),
            "FCVT.L.D" => Some(Self::Fcvtld),
            "FCVT.LU.D" => Some(Self::Fcvtlud),
            "FMV.X.D" => Some(Self::Fmvxd),
            "FMV.D.X" => Some(Self::Fmvdx), 
            "FCVT.D.L" => Some(Self::Fcvtdl),
            "FCVT.D.LU" => Some(Self::Fcvtdlu),
            "LR.W" => Some(Self::Lrw),
            "SC.W" => Some(Self::Scw),
            "AMOSWAP.W" => Some(Self::Amoswapw),
            "AMOADD.W" => Some(Self::Amoaddw),
            "AMOXOR.W" => Some(Self::Amoxorw),
            "AMOAND.W" => Some(Self::Amoandw),
            "AMOOR.W" => Some(Self::Amoorw),
            "AMOMIN.W" => Some(Self::Amominw),
            "AMOMAX.W" => Some(Self::Amomaxw),
            "AMOMINU.W" => Some(Self::Amominuw),
            "AMOMAXU.W" => Some(Self::Amomaxuw),
            "LR.D" => Some(Self::Lrd),
            "SC.D" => Some(Self::Scd),
            "AMOSWAP.D" => Some(Self::Amoswapd),
            "AMOADD.D" => Some(Self::Amoaddd),
            "AMOXOR.D" => Some(Self::Amoxord),
            "AMOAND.D" => Some(Self::Amoandd),
            "AMOOR.D" => Some(Self::Amoord),
            "AMOMIN.D" => Some(Self::Amomind),
            "AMOMAX.D" => Some(Self::Amomaxd),
            "AMOMINU.D" => Some(Self::Amominud),
            "AMOMAXU.D" => Some(Self::Amomaxud),

            //compact inc's
            "C.ADDI4SPN" => Some(Self::Caddi4spn),
            "C.FLD" => Some(Self::Fld),
            "C.LQ" => Some(Self::Clq),
            "C.LW" => Some(Self::Clw),
            "C.FLW" => Some(Self::Cflw),
            "C.LD" => Some(Self::Cld),
            "C.FSD" => Some(Self::Cfsd),
            "C.SQ" => Some(Self::Csq),
            "C.SW" => Some(Self::Csw),
            "C.FSW" => Some(Self::Cfsw),
            "C.SD" => Some(Self::Csd),

            "C.NOP" => Some(Self::Cnop),
            "C.ADDI" => Some(Self::Caddi),
            "C.JAL" => Some(Self::Cjal),
            "C.ADDIW" => Some(Self::Caddiw),
            "C.LI" => Some(Self::Cli),
            "C.ADDI16SP"  => Some(Self::Caddi16sp),
            "C.LUI" => Some(Self::Clui),
            "C.SRLI" => Some(Self::Csrli),
            "C.SRLI64" => Some(Self::Csrli64),
            "C.SRAI" => Some(Self::Csrai),
            "C.SRAI64" => Some(Self::Csrai64),
            "C.ANDI" => Some(Self::Candi),
            "C.SUB" => Some(Self::Csub),
            "C.XOR" => Some(Self::Cxor),
            "C.OR" => Some(Self::Cor),
            "C.AND" => Some(Self::And),
            "C.SUBW" => Some(Self::Csubw),
            "C.ADDW" => Some(Self::Addw),
            "C.J" => Some(Self::Cj),
            "C.BEQZ" => Some(Self::Cbeqz),
            "C.BNEZ" => Some(Self::Cbnez),

            "C.SLLI" => Some(Self::Cslli),
            "C.SLLI64" => Some(Self::Cslli64),
            "C.FLDSP" => Some(Self::Cfldsp),
            "C.LQSP" => Some(Self::Clqsp),
            "C.LWSP" => Some(Self::Clwsp),
            "C.FLWSP" => Some(Self::Cflwsp),
            "C.LDSP" => Some(Self::Cldsp),
            "C.JR" => Some(Self::Cjr),
            "C.MV" => Some(Self::Cmv),
            "C.EBREAK" => Some(Self::Cebreak),
            "C.JALR" => Some(Self::Cjalr),
            "C.ADD" => Some(Self::Cadd),
            "C.FSDSP" => Some(Self::Cfsdsp),
            "C.SQSP" => Some(Self::Csqsp),
            "C.SWSP" => Some(Self::Cswsp),
            "C.FSWSP" => Some(Self::Cfswsp),
            "C.SDSP" => Some(Self::Csdsp),

            _ => None,
        }
    }

    fn to_instruction_type(op: u32) -> InstructionTypes {
        match op {
            0b1000_111 |
            0b1001_111 |
            0b1001_011 |
            0b1000_011 |
            0b1010_011 |
            0b0101_111 |
            0b0110_011 | 
            0b0111_011 => InstructionTypes::R,
            0b0010_111 | 
            0b0110_111 => InstructionTypes::U,
            0b0000_111 |
            0b1100_111 | 
            0b0010_011 | 
            0b1110_011 | 
            0b0000_011 | 
            0b0011_011 => InstructionTypes::I,
            0b1100_011 => InstructionTypes::B,
            0b0100_111 |
            0b0100_011 => InstructionTypes::S,
            0b1101_111 => InstructionTypes::J,
            0b00 |
            0b01 |
            0b10 => InstructionTypes::COMPACT,
            _ => InstructionTypes::UnKnown,
        }
    }

    pub (crate) fn get_instruction_type(&self) -> InstructionTypes {
        let v = self.get_value();
        Self::to_instruction_type(v as u32)
    }

    pub (crate) fn get_instruction_type_from_string(str:&str) -> Result<InstructionTypes, AsmError> {
        if let Some(r) = Self::from_str(str) {
            let rr = r.get_instruction_type();
            Ok(rr)
        }
        else {
            //warn_string(format!("{str} cannot find an instruction"));
            Ok(InstructionTypes::UnKnown)   //psueduo code
        }
    }

    pub fn get_funct3_str(&self) -> Result<String, AsmError> {
        let r = self.get_funct3().ok_or(AsmError::ConverstionError((file!(), line!()).into(), format!("cannot find funct3 for {self:?}")))?;
        Ok(format!("{r:0>3b}"))
    }

    pub fn get_funct7_str(&self) -> Result<String, AsmError> {
        let r = self.get_funct7().ok_or(AsmError::ConverstionError((file!(), line!()).into(), format!("cannot find funct7 for {self:?}")))?;
        Ok(format!("{r:0>7b}"))
    }
}
