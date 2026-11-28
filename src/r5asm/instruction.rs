use pest::{iterators::Pair, Parser};

use super::asm_error::AsmError;
use super::external_label::*;
use super::imm::Imm;
use super::imm_macro::ImmMacro;
use core_utils::debug::*;
use core_utils::number::*;
use core_utils::traits::generate_code::GenerateCode;
use super::label_offset::*;

use super::basic_instruction_extensions::BasicInstructionExtensions;
use super::{code_gen_config::CodeGenConfiguration, macro_instruction::MacroArgs, opcode::OpCode, r5asm_pest::*, register::Register};

use std::fmt;
use std::fmt::Display;
use std::hash::Hasher;

macro_rules! create_register_name_same_fn {
    ($fn_name:ident, $reg:ident) => {
        pub fn $fn_name(&self, name:&str) -> bool {
            if let Some(r) = self.$reg.as_ref() {
                r == name
            }
            else {
                false
            }
        }
    };
}

macro_rules! create_set_reg_value_fn {
    ($fn_name:ident, $reg:ident) => {
        pub (crate) fn $fn_name(&mut self, r:&str) {
            self.$reg = Some(r.to_string())
        }
    };
}

pub type ExternalSymbol = String;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SourcePosition {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl SourcePosition {
    fn from_position(pos: &pest::Position<'_>) -> Self {
        let (line, column) = pos.line_col();
        Self {
            line,
            column,
            offset: pos.pos(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SourceRange {
    pub start: SourcePosition,
    pub end: SourcePosition,
}

impl SourceRange {
    pub fn from_pair(pair: &Pair<Rule>) -> Self {
        let span = pair.as_span();
        Self {
            start: SourcePosition::from_position(&span.start_pos()),
            end: SourcePosition::from_position(&span.end_pos()),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InstructionSourceRanges {
    pub name: Option<SourceRange>,
    pub r0: Option<SourceRange>,
    pub r1: Option<SourceRange>,
    pub r2: Option<SourceRange>,
    pub r3: Option<SourceRange>,
    pub imm: Option<SourceRange>,
    pub option: Option<SourceRange>,
    pub rel_fun: Option<SourceRange>,
}

/// Represents a RISC-V assembly instruction, including its name, type, registers, 
/// immediate values, and source code locations.
#[derive(Clone, PartialEq)]
pub struct Instruction {
    /// instruction name, e.g., "add", "sub", "lw", etc.
    pub(super) name : String, 

    pub(super) inc_type : InstructionTypes,
    pub(super) inc_extensions_and_type : BasicInstructionExtensions,
    pub(super) r0_name : Option<String>,
    pub(super) r1_name : Option<String>,
    pub(super) r2_name : Option<String>, 
    pub(super) r3_name : Option<String>,
    pub(super) option : Option<String>, 
    imm : Option<Imm>,
    pub(super) rel_fun : Option<String>,
    pub(super) operations : EmitOperation,
    pub(super) label_virtual_address : u32,   // this field be store imm or offset, it consolidate the value for machine code based on inc type
    pub(super) source_ranges: InstructionSourceRanges,

    pub (super) is_generate : bool,
    external_symbol : ExternalSymbol,
}

impl Instruction {
    fn process_fixed_i_imm_value(inc_name: &str) -> Option<String> {
        match inc_name.to_lowercase().as_str() {
            "clz" => Some("1536".to_string()),     // 0b011000000000
            "ctz" => Some("1537".to_string()),     // 0b011000000001
            "cpop" => Some("1538".to_string()),    // 0b011000000010
            "sext.b" => Some("1540".to_string()),  // 0b011000000100
            "sext.h" => Some("1541".to_string()),  // 0b011000000101
            "sext.w" => Some("0".to_string()),     // alias form of addiw rd, rs1, 0
            "orc.b" => Some("647".to_string()),    // 0b001010000111
            "rev8" => Some("1720".to_string()),    // 0b011010111000 (rv64)
            "clzw" => Some("1536".to_string()),    // 0b011000000000 (RV64 word)
            "ctzw" => Some("1537".to_string()),    // 0b011000000001 (RV64 word)
            "cpopw" => Some("1538".to_string()),   // 0b011000000010 (RV64 word)
            "brev8" => Some("1671".to_string()),   // 0b011010000111
            "zip" => Some("143".to_string()),      // 0b000010001111 (RV32)
            "unzip" => Some("143".to_string()),    // 0b000010001111 (RV32)
            "aes64im" => Some("768".to_string()),  // 0b001100000000
            "sha256sig0" => Some("258".to_string()), // 0b000100000010
            "sha256sig1" => Some("259".to_string()), // 0b000100000011
            "sha256sum0" => Some("256".to_string()), // 0b000100000000
            "sha256sum1" => Some("257".to_string()), // 0b000100000001
            "sha512sig0" => Some("262".to_string()), // 0b000100000110
            "sha512sig1" => Some("263".to_string()), // 0b000100000111
            "sha512sum0" => Some("260".to_string()), // 0b000100000100
            "sha512sum1" => Some("261".to_string()), // 0b000100000101
            "sm3p0" => Some("264".to_string()),      // 0b000100001000
            "sm3p1" => Some("265".to_string()),      // 0b000100001001
            _ => None,
        }
    }

    fn process_shamt_value(inc_name:&String, p2:&Pair<Rule>) -> Result<String, AsmError> {
        let imm = match inc_name.to_lowercase().as_str() {
            "slliw" |
            "srliw" => {
                let imm_value = pair_to_i64(p2)? & 0x1f;
                format!("{imm_value}")
            }
            "srai" => {
                // srai encodes arithmetic shift with imm[11:5]=0b0100000 and shamt in imm[4:0] (or imm[5:0] on RV64).
                let imm_value = (pair_to_i64(p2)? & 0x3f) | 0x400;
                format!("{imm_value}")
            }
            "sraiw" => {
                // sraiw encodes arithmetic shift with imm[11:5]=0b0100000 and shamt in imm[4:0].
                let imm_value = (pair_to_i64(p2)? & 0x1f) | 0x400;
                format!("{imm_value}")
            }
            "rori" => {
                // rori encodes 0b0110000 in imm[11:5] with shamt in imm[4:0].
                let imm_value = (pair_to_i64(p2)? & 0x1f) | 0x600;
                format!("{imm_value}")
            }
            "slli.uw" => {
                // slli.uw encodes funct7=0b0000100 in imm[11:5] with shamt in imm[4:0].
                let imm_value = (pair_to_i64(p2)? & 0x1f) | 0x80;
                format!("{imm_value}")
            }
            "bclri" | "bexti" => {
                // bclri/bexti encode 0b0100100 in imm[11:5] with shamt in imm[4:0].
                let imm_value = (pair_to_i64(p2)? & 0x1f) | 0x480;
                format!("{imm_value}")
            }
            "binvi" => {
                // binvi encodes 0b0110100 in imm[11:5] with shamt in imm[4:0].
                let imm_value = (pair_to_i64(p2)? & 0x1f) | 0x680;
                format!("{imm_value}")
            }
            "bseti" => {
                // bseti encodes 0b0010100 in imm[11:5] with shamt in imm[4:0].
                let imm_value = (pair_to_i64(p2)? & 0x1f) | 0x280;
                format!("{imm_value}")
            }
            "roriw" => {
                // roriw encodes 0b0110000 in imm[11:5] with shamt in imm[4:0] (RV64 word).
                let imm_value = (pair_to_i64(p2)? & 0x1f) | 0x600;
                format!("{imm_value}")
            }
            "aes64ks1i" => {
                // aes64ks1i uses imm[11:5]=0b0011000, imm[4]=1, imm[3:0]=rnum (0..10).
                let rnum = pair_to_i64(p2)?;
                if !(0..=10).contains(&rnum) {
                    return Err(AsmError::GeneralError((file!(), line!()).into(),
                        format!("aes64ks1i rnum must be in [0,10], got {rnum}")));
                }
                let imm_value = 0x310 | (rnum & 0xf);
                format!("{imm_value}")
            }
            _ => p2.as_str().to_string()
        };

        Ok(imm)
    }

    pub fn from_string(input:&str, config:&mut CodeGenConfiguration) -> Result<Vec<Self>, AsmError> {
        let input = input.trim();
        let mut pairs = R5AsmParser::parse(Rule::instruction, input)
                                            .map_err(|e| AsmError::GeneralError((file!(), line!()).into(), format!("source error @ {} and {:?}", e.line(), &e.line_col)))?;
        if let Some(pair) = pairs.find(|n| n.as_str().len() == input.len()) {
            let prog_r = Self::from_pair(&pair, config);
            if prog_r.is_err() {
                error_str("cannot get program from Rule::instruction");
                Err(AsmError::ParsingConversionError((file!(), line!()).into(), "cannot get program from string".to_string()))
            }
            else { 
                let prog = prog_r.unwrap();
                Ok(prog)
            }
        }
        else {
            error_string(format!("Error: {} at {}", "does not catch all string", input.to_owned()));
            debug_string(format!("input: {}\r\nParsed: {:#?}", input, pairs));
            let count = pairs.count();
            debug_string(format!("Pairs count = {count}\r\n"));                
            Err(AsmError::ParsingConversionError((file!(), line!()).into(), "parsing does not catch all string".to_string()))
        }
    }

    fn la_to_incs(p:&Pair<Rule>, p1:&str) -> Result<Vec<Self>, AsmError> {
        let r0_name = p.as_str();
        Self::la_to_incs_from_str(r0_name, p1)
    }

    fn mv_to_incs(p:&Pair<Rule>, p1:&Pair<Rule>) -> Result<Vec<Self>, AsmError> {
        let r0_name = p.as_str();
        let r1_name = p1.as_str();
        Self::mv_to_incs_from_str(r0_name, r1_name)
    }

    fn mv_to_incs_from_str(r0_name:&str, r1_name:&str) -> Result<Vec<Self>, AsmError> {
        let (inc_name, inc_type) = Self::get_inc_and_inc_type("addi")?;                                    
        let mut r = Self::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions);
        r.r0_name = Some(r0_name.to_string());
        r.r1_name = Some(r1_name.to_string());
        r.set_imm(Some("0".to_string().into()));
        Ok([r].to_vec())
    }

    fn la_to_incs_from_str(r0_name:&str, p1:&str) -> Result<Vec<Self>, AsmError> {
        let (inc_name, inc_type) = Self::get_inc_and_inc_type("auipc")?;                                    
        let mut r = Self::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions);
        r.r0_name = Some(r0_name.to_string());
        r.set_imm(Some(p1.to_string().into()));
        r.rel_fun = Some("%pcrel_hi_la".to_string());
        r.operations = EmitOperation::ApplyOffset(0);

        let (inc_name2, inc_type2) = Self::get_inc_and_inc_type("addi")?; 
        let mut r2 = Self::new(inc_name2, inc_type2, BasicInstructionExtensions::BaseIntegerInstructions);
        r2.r0_name = Some(r0_name.to_string());
        r2.r1_name = Some(r0_name.to_string());
        r2.set_imm(Some(p1.to_string().into()));
        r2.rel_fun = Some("%pcrel_lo_la".to_string());
        r2.operations = EmitOperation::ApplyOffset(4);   //the calculation is based on previous instruction's offset to need to substract 4 bytes more. 4 bytes is 1 instruction's length

        Ok([r, r2].to_vec())
    }

    fn lw_to_incs(p:&str, p1:&str) -> Result<Vec<Self>, AsmError> {
        let (inc_name, inc_type) = Self::get_inc_and_inc_type("auipc")?;                                    
        let mut r = Self::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions);
        r.r0_name = Some(p.to_string());
        r.set_imm(Some(p1.to_string().into()));
        r.rel_fun = Some("%pcrel_hi_la".to_string());
        r.operations = EmitOperation::ApplyOffset(0);

        let (inc_name2, inc_type2) = Self::get_inc_and_inc_type("lw")?; 
        let mut r2 = Self::new(inc_name2, inc_type2, BasicInstructionExtensions::BaseIntegerInstructions);
        r2.r0_name = Some(p.to_string());
        r2.r1_name = Some(p.to_string());
        r2.set_imm(Some(p1.to_string().into()));
        r2.rel_fun = Some("%pcrel_lo_la".to_string());
        r2.operations = EmitOperation::ApplyOffset(4);   //the calculation is based on previous instruction's offset to need to substract 4 bytes more. 4 bytes is 1 instruction's length

        Ok([r, r2].to_vec())
    }

    fn ld_to_incs(p:&str, p1:&str) -> Result<Vec<Self>, AsmError> {
        let (inc_name, inc_type) = Self::get_inc_and_inc_type("auipc")?;                                    
        let mut r = Self::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions);
        r.r0_name = Some(p.to_string());
        r.set_imm(Some(p1.to_string().into()));
        r.rel_fun = Some("%pcrel_hi_la".to_string());
        r.operations = EmitOperation::ApplyOffset(0);

        let (inc_name2, inc_type2) = Self::get_inc_and_inc_type("ld")?; 
        let mut r2 = Self::new(inc_name2, inc_type2, BasicInstructionExtensions::BaseIntegerInstructions);
        r2.r0_name = Some(p.to_string());
        r2.r1_name = Some(p.to_string());
        r2.set_imm(Some(p1.to_string().into()));
        r2.rel_fun = Some("%pcrel_lo_la".to_string());
        r2.operations = EmitOperation::ApplyOffset(4);   //the calculation is based on previous instruction's offset to need to substract 4 bytes more. 4 bytes is 1 instruction's length

        Ok([r, r2].to_vec())
    }

    fn get_pseudo_replacement(inc_name_low_case:&str, args: impl Into<MacroArgs>, config:&CodeGenConfiguration) -> Result<Vec<Self>, AsmError> {
        let macro_archive = config.get_marco_instruction_archive();
        match macro_archive.get_incs(inc_name_low_case, args) {
            Ok(r) => Ok(r),
            Err(err) => {
                error_string(format!("cannot convert pseudo code '{inc_name_low_case}' to base inc"));
                return Err(err);
            }
        }
    }

    fn new_addi(rd:&str, rs1:&str, imm:i64) -> Self {
        let (inc_name, inc_type) = Self::get_inc_and_inc_type("addi").unwrap();
        Self::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions)
            .with_r0(rd)
            .with_r1(rs1)
            .with_imm(imm.to_string().into())
    }

    fn new_lui(rd:&str, imm:i64) -> Self {
        let (inc_name, inc_type) = Self::get_inc_and_inc_type("lui").unwrap();
        Self::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions)
            .with_r0(rd)
            .with_imm(imm.to_string().into())
    }

    fn new_slli(rd:&str, rs1:&str, shamt:i64) -> Self {
        let (inc_name, inc_type) = Self::get_inc_and_inc_type("slli").unwrap();
        Self::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions)
            .with_r0(rd)
            .with_r1(rs1)
            .with_imm(shamt.to_string().into())
    }

    fn new_srli(rd:&str, rs1:&str, shamt:i64) -> Self {
        let (inc_name, inc_type) = Self::get_inc_and_inc_type("srli").unwrap();
        Self::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions)
            .with_r0(rd)
            .with_r1(rs1)
            .with_imm(shamt.to_string().into())
    }

    fn new_ori(rd:&str, rs1:&str, imm:i64) -> Self {
        let (inc_name, inc_type) = Self::get_inc_and_inc_type("ori").unwrap();
        Self::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions)
            .with_r0(rd)
            .with_r1(rs1)
            .with_imm(imm.to_string().into())
    }

    fn generate_load_32bit_imm_incs(rd:&str, imm:i64) -> Result<Vec<Self>, AsmError> {
        assert!(can_fits_in_32bit(imm));

        let r_shiftleft = Self::new_slli(rd, rd, 32);
        let r_shiftright = Self::new_srli(rd, rd, 32);

        if imm < 0 {
            let (low, high) = Self::get_low12_and_high_with_sign_process(imm);
            let r  = Self::new_lui(rd, high.into());
            
            if low == 0 {
                Ok(vec![r])
            }
            else {
                let r2 = Self::new_addi(rd, rd, low.into());
                Ok(vec![r, r2])
            }
        }
        else {
            if can_fits_in_32i(imm) {
                let (low, high) = Self::get_low12_and_high_with_sign_process(imm);
                let r  = Self::new_lui(rd, high.into());
                if low == 0 {
                    if high & 0x8_0000 == 0 {
                        Ok(vec![r])
                    }
                    else {
                        Ok(vec![r, r_shiftleft, r_shiftright])
                    }
                }
                else {
                    if high & 0x8_0000 == 0 {
                        let r2 = Self::new_addi(rd, rd, low.into());
                        Ok(vec![r, r2])
                    }
                    else {
                        let r2 = Self::new_addi(rd, rd, low.into());
                        Ok(vec![r, r2, r_shiftleft, r_shiftright])
                    }
                }
            }
            else {
                let (low, high) = Self::get_low12_and_high_with_sign_process(imm);
                let r  = Self::new_lui(rd, high.into());
                if low == 0 {
                    if high & 0x8_0000 == 0 {
                        Ok(vec![r, r_shiftleft, r_shiftright])
                    }
                    else {
                        Ok(vec![r, r_shiftleft, r_shiftright])
                    }
                }
                else {
                    if high & 0x8_0000 == 0 {
                        let r2 = Self::new_addi(rd, rd, low.into());
                        Ok(vec![r, r2, r_shiftleft, r_shiftright])
                    }
                    else {
                        let r2 = Self::new_addi(rd, rd, low.into());
                        Ok(vec![r, r2, r_shiftleft, r_shiftright])
                    }
                }
            }
        }    
    }

    /// Expand a `li rd, imm` pseudo-instruction into real instructions for any imm width.
    /// Near (-2048..=2047)   → addi rd, x0, imm
    /// 32-bit                → lui + addi
    /// 64-bit                → lui/addi + slli/ori sequence
    pub (crate) fn li_from_imm(rd: &str, imm: i64) -> Result<Vec<Self>, AsmError> {
        if Self::is_near(imm) {
            let r = Self::new_addi(rd, "x0", imm);
            Ok(vec![r])
        } else if can_fits_in_32bit(imm) {            
            // if high's 31th bit is 1, need to set the highest bit and then process rest of 31 bits
            Self::generate_load_32bit_imm_incs(rd, imm)
        } else {
            Self::li_to_incs(rd, imm)
        }
    }

    fn li_to_incs(rd:&str, imm:i64) -> Result<Vec<Self>, AsmError> {
        assert!(!can_fits_in_32bit(imm));

        let mut instrs = Vec::new();
        
        // first load high 32 bits with 2 instructions
        let imm_high32 = (imm as u64) >> 32;
        let imm_low32 = imm as u64 & 0xFFFFFFFF;

        let load_high_32 = Self::generate_load_32bit_imm_incs(rd, imm_high32 as i64)?;
        instrs.extend(load_high_32);

        // the rest 32 bits can be loaded with slli and ori, ori only handle 10, 11, 11 bits
        let top8  = (imm_low32 >> 22) & 0x3FF;
        let mid12 = (imm_low32 >> 11) & 0x7FF;
        let bot12 = (imm_low32 >>  0) & 0x7FF;
        
        let shift_top8 = Self::new_slli(rd, rd, 10);
        instrs.push(shift_top8);
        if top8 != 0 {
            let r = Self::new_ori(rd, rd, top8 as i64);
            instrs.push(r);
        }

        let shift_mid12 = Self::new_slli(rd, rd, 11);
        instrs.push(shift_mid12);
        if mid12 != 0 {
            let r = Self::new_ori(rd, rd, mid12 as i64);
            instrs.push(r);
        }

        let shift_bot12 = Self::new_slli(rd, rd, 11);
        instrs.push(shift_bot12);
        if bot12 != 0 {
            let r = Self::new_ori(rd, rd, bot12 as i64);
            instrs.push(r);
        }

        Ok(instrs)
    }

    fn with_r0(mut self, r0_name:&str) -> Self {
        self.r0_name = Some(r0_name.to_string());
        self
    }

    fn with_r1(mut self, r1_name:&str) -> Self {
        self.r1_name = Some(r1_name.to_string());
        self
    }

    fn with_imm(mut self, imm:Imm) -> Self {
        self.imm = Some(imm);
        self
    }

    fn get_source_range_from_pair(pair: &Pair<Rule>) -> SourceRange {
        SourceRange::from_pair(pair)
    }

    fn get_inc_name_range(pair: &Pair<Rule>) -> Option<SourceRange> {
        if let Some(inc_name) = pair.to_owned().into_inner().find_first_tagged("inc_name") {
            Some(Self::get_source_range_from_pair(&inc_name))
        } else if pair.as_node_tag().is_some() && pair.as_node_tag().unwrap() == "inc_name" {
            Some(Self::get_source_range_from_pair(pair))
        } else {
            None
        }
    }

    pub fn get_source_ranges(&self) -> &InstructionSourceRanges {
        &self.source_ranges
    }

    pub fn get_name_location(&self) -> Option<&SourceRange> {
        self.source_ranges.name.as_ref()
    }

    pub fn get_r0_location(&self) -> Option<&SourceRange> {
        self.source_ranges.r0.as_ref()
    }

    pub fn get_r1_location(&self) -> Option<&SourceRange> {
        self.source_ranges.r1.as_ref()
    }

    pub fn get_r2_location(&self) -> Option<&SourceRange> {
        self.source_ranges.r2.as_ref()
    }

    pub fn get_r3_location(&self) -> Option<&SourceRange> {
        self.source_ranges.r3.as_ref()
    }

    pub fn get_register_location(&self, reg: InstructionRegisterName) -> Option<&SourceRange> {
        match reg {
            InstructionRegisterName::Rs0 |
            InstructionRegisterName::Rd => self.get_r0_location(),
            InstructionRegisterName::Rs1 => self.get_r1_location(),
            InstructionRegisterName::Rs2 => self.get_r2_location(),
            InstructionRegisterName::Rs3 => self.get_r3_location(),
        }
    }

    pub fn get_imm_location(&self) -> Option<&SourceRange> {
        self.source_ranges.imm.as_ref()
    }

    pub fn get_option_location(&self) -> Option<&SourceRange> {
        self.source_ranges.option.as_ref()
    }

    pub fn get_rel_fun_location(&self) -> Option<&SourceRange> {
        self.source_ranges.rel_fun.as_ref()
    }

    pub (crate) fn from_pair(pair:&Pair<Rule>, config:&mut CodeGenConfiguration) -> Result<Vec<Self>, AsmError> {
        let inner = pair.to_owned().into_inner()
                    .nth(0)
                    .ok_or(AsmError::ConversionFailed((file!(), line!()).into(), "conversion failure".to_string()))?;
        let inner_rule = inner.as_rule();
        
        let extension_inc = inner.into_inner().nth(0).unwrap();
        let inc_name = Self::get_inc(&extension_inc)?;
        let inc_name_location = Self::get_inc_name_range(&extension_inc);
        let extention_type = 
            if inner_rule == Rule::pseudoinstructions { BasicInstructionExtensions::PseudoInstructions } 
            else { Self::get_inc_extension_type(&extension_inc)? };
        let inc_type = if extention_type == BasicInstructionExtensions::RvVInstructions {
            InstructionTypes::UnKnown
        } else {
            OpCode::get_instruction_type_from_string(&inc_name)?
        };

        let mut instructions = match inner_rule {
            Rule::basic_instructions => {
                let rules = extension_inc.into_inner().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
                match rules.as_slice() {
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::vtypei, p2)] => {
                        let mut r = Self::new_r0_r1(inc_name, inc_type, extention_type, p, p1);
                        r.set_option(p2);
                        Ok([r].to_vec())
                    }
                    [_, (Rule::registers, p), (Rule::integer, p1), (Rule::vtypei, p2)] => {
                        let mut r = Self::new_r0_imm(inc_name, inc_type, extention_type, p, p1);
                        r.set_option(p2);
                        Ok([r].to_vec())
                    }
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::integer, p2), (Rule::option, option)] => {
                        let r = Self::new_r0_r1_imm_option(inc_name, inc_type, extention_type, p, p1, p2, option);
                        Ok([r].to_vec())
                    },
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::integer, p2)] |
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::var_name, p2)] |
                    [_, (Rule::registers, p), (Rule::integer, p2), (Rule::registers, p1)] |
                    [_, (Rule::registers, p), (Rule::var_name, p2), (Rule::registers, p1)] => {
                        let imm = Self::process_shamt_value(&inc_name, p2)?;
                        let mut r = Self::new_r0_r1(inc_name, inc_type, extention_type, p, p1);
                        r.set_imm_from_pair(p2);
                        r.set_imm(Some(imm.into()));
                        Ok([r].to_vec())
                    }
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::imm_macro, p2)] => {
                        let imm_macro = ImmMacro::from_pair(&p2)?;
                        let mut r = Self::new_r0_r1(inc_name, inc_type, extention_type, p, p1);
                        r.set_imm_from_pair(p2);
                        r.set_imm(Some(imm_macro.into()));
                        Ok([r].to_vec())
                    }
                    [_, (Rule::registers, p), (Rule::registers, p1)] => {
                        let fixed_imm = Self::process_fixed_i_imm_value(&inc_name);
                        let mut r = Self::new_r0_r1(inc_name, inc_type, extention_type, p, p1);
                        if r.name.to_ascii_lowercase() == "zext.h" {
                            // zext.h is an alias form that fixes rs2 to x0.
                            r.r2_name = Some("x0".to_string());
                        } else if let Some(imm) = fixed_imm {
                            r.set_imm(Some(imm.into()));
                        }

                        Ok([r].to_vec())
                    }
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::option, option)] => 
                        Ok([Self::new_r0_r1_option(inc_name, inc_type, extention_type, p, p1, option)].to_vec()),
                    [_, (Rule::registers, p), (Rule::relocation_function, p2), (Rule::registers, p1)] |
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::relocation_function, p2)] =>
                        Ok([Self::new_r0_r1_rel(inc_name, inc_type, extention_type, p, p1, p2)].to_vec()),
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::registers, p2), (Rule::option, option)] => 
                        Ok([Self::new_r0_r1_r2_option(inc_name, inc_type, extention_type, p, p1, p2, option)].to_vec()),
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::registers, p2)] => 
                        Ok([Self::new_r0_r1_r2(inc_name, inc_type, extention_type, p, p1, p2)].to_vec()),
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::registers, p2), (Rule::registers, p3)] =>
                        Ok([Self::new_r0_r1_r2_r3(inc_name, inc_type, extention_type, p, p1, p2, p3)].to_vec()),
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::registers, p2), (Rule::registers, p3), (Rule::option, option)] =>
                        Ok([Self::new_r0_r1_r2_r3_option(inc_name, inc_type, extention_type, p, p1, p2, p3, option)].to_vec()),
                    [_, (Rule::registers, p), (Rule::var_name, p1)]|
                    [_, (Rule::registers, p), (Rule::integer, p1)] => 
                        Ok([Self::new_r0_imm(inc_name, inc_type, extention_type, p, p1)].to_vec()),
                    [_, (Rule::registers, p), (Rule::relocation_function, p1)] =>
                        Ok([Self::new_r0_rel(inc_name, inc_type, extention_type, p, p1)].to_vec()),
                    [_, (Rule::integer, p)] | [_, (Rule::var_name, p)]
                        if inc_type == InstructionTypes::COMPACT =>
                        Ok([Self::new_imm(inc_name, inc_type, extention_type, p)].to_vec()),
                    [_] => 
                        Ok([Self::new(inc_name, inc_type, extention_type)].to_vec()),
                    [_, (Rule::registers, p), (Rule::option, option)] => 
                        Ok([Self::new_r0_option(inc_name, inc_type, extention_type, p, option)].to_vec()),
                    [_, (Rule::registers, p)] => 
                        Ok([Self::new_r0(inc_name, inc_type, extention_type, p)].to_vec()),
                    [_, (Rule::registers, p), (Rule::csr, p1), (Rule::integer, p2)] => 
                        Ok([Self::new_r0_r1_imm(inc_name, inc_type, extention_type, p, p1, p2)].to_vec()),
                    _ => {
                        Err(AsmError::MissingCase((file!(), line!()).into(), Rule::base_integer_instructions))
                    }
                }                
            }
            Rule::rvc_instructions => {
                let rules = extension_inc.into_inner().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
                match rules.as_slice() {
                    [_, (Rule::registers, p), (Rule::integer, p1), (Rule::registers, p2)] |
                    [_, (Rule::registers, p), (Rule::var_name, p1), (Rule::registers, p2)] =>
                        Ok([Self::new_r0_r1_imm(inc_name, inc_type, extention_type, p, p2, p1)].to_vec()),
                    [_, (Rule::registers, p), (Rule::var_name, p1)] |
                    [_, (Rule::registers, p), (Rule::integer, p1)] => 
                        Ok([Self::new_r0_imm(inc_name, inc_type, extention_type, p, p1)].to_vec()),
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::var_name, p2)] |
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::integer, p2)] => 
                        Ok([Self::new_r0_r1_imm(inc_name, inc_type, extention_type, p, p1, p2)].to_vec()),
                    [_, (Rule::registers, p), (Rule::registers, p1)] => 
                        Ok([Self::new_r0_r1(inc_name, inc_type, extention_type, p, p1)].to_vec()),
                    [_, (Rule::registers, p)] => 
                        Ok([Self::new_r0(inc_name, inc_type, extention_type, p)].to_vec()),
                    [_, (Rule::var_name, p)] | 
                    [_, (Rule::integer, p)] => 
                        Ok([Self::new_imm(inc_name, inc_type, extention_type, p)].to_vec()),
                    [_] => 
                        Ok([Self::new(inc_name, inc_type, extention_type)].to_vec()),
                    _ => Err(AsmError::MissingCase((file!(), line!()).into(), Rule::rvc_instructions)),
                }
            }
            Rule::rv_64_128 => {
                let rules = extension_inc.into_inner().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
                match rules.as_slice() {
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::var_name, p2)] |
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::integer, p2)] => {
                        let imm = Self::process_shamt_value(&inc_name, p2)?;
                        let mut r = Self::new_r0_r1(inc_name, inc_type, extention_type, p, p1);
                        r.set_imm_from_pair(p2);
                        r.set_imm(Some(imm.into()));
                        Ok([r].to_vec())
                    }
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::registers, p2)] => 
                        Ok([Self::new_r0_r1_r2(inc_name, inc_type, extention_type, p, p1, p2)].to_vec()),
                    [_, (Rule::registers, p), (Rule::registers, p1)] => {
                        let fixed_imm = Self::process_fixed_i_imm_value(&inc_name);
                        let mut r = Self::new_r0_r1(inc_name, inc_type, extention_type, p, p1);
                        if let Some(imm) = fixed_imm {
                            r.set_imm(Some(imm.into()));
                        }
                        Ok([r].to_vec())
                    }
                    [_, (Rule::registers, p)] => 
                        Ok([Self::new_r0(inc_name, inc_type, extention_type, p)].to_vec()),
                    [_] => 
                        Ok([Self::new(inc_name, inc_type, extention_type)].to_vec()),
                    _ => Err(AsmError::MissingCase((file!(), line!()).into(), Rule::rv_64_128)),
                }
            }
            Rule::rv_zba_instructions => {
                let rules = extension_inc.into_inner().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
                match rules.as_slice() {
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::var_name, p2)] |
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::integer, p2)] => {
                        let imm = Self::process_shamt_value(&inc_name, p2)?;
                        let mut r = Self::new_r0_r1(inc_name, inc_type, extention_type, p, p1);
                        r.set_imm_from_pair(p2);
                        r.set_imm(Some(imm.into()));
                        Ok([r].to_vec())
                    }
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::registers, p2)] =>
                        Ok([Self::new_r0_r1_r2(inc_name, inc_type, extention_type, p, p1, p2)].to_vec()),
                    _ => Err(AsmError::MissingCase((file!(), line!()).into(), Rule::rv_zba_instructions)),
                }
            }
            Rule::rv_zbc_instructions => {
                let rules = extension_inc.into_inner().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
                match rules.as_slice() {
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::registers, p2)] =>
                        Ok([Self::new_r0_r1_r2(inc_name, inc_type, extention_type, p, p1, p2)].to_vec()),
                    _ => Err(AsmError::MissingCase((file!(), line!()).into(), Rule::rv_zbc_instructions)),
                }
            }
            Rule::rv_zicond_instructions => {
                let rules = extension_inc.into_inner().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
                match rules.as_slice() {
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::registers, p2)] =>
                        Ok([Self::new_r0_r1_r2(inc_name, inc_type, extention_type, p, p1, p2)].to_vec()),
                    _ => Err(AsmError::MissingCase((file!(), line!()).into(), Rule::rv_zicond_instructions)),
                }
            }
            Rule::rv_zbs_instructions => {
                let rules = extension_inc.into_inner().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
                match rules.as_slice() {
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::var_name, p2)] |
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::integer, p2)] => {
                        let imm = Self::process_shamt_value(&inc_name, p2)?;
                        let mut r = Self::new_r0_r1(inc_name, inc_type, extention_type, p, p1);
                        r.set_imm_from_pair(p2);
                        r.set_imm(Some(imm.into()));
                        Ok([r].to_vec())
                    }
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::registers, p2)] =>
                        Ok([Self::new_r0_r1_r2(inc_name, inc_type, extention_type, p, p1, p2)].to_vec()),
                    _ => Err(AsmError::MissingCase((file!(), line!()).into(), Rule::rv_zbs_instructions)),
                }
            }
            Rule::rv_zbb_instructions => {
                let rules = extension_inc.into_inner().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
                match rules.as_slice() {
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::var_name, p2)] |
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::integer, p2)] => {
                        let imm = Self::process_shamt_value(&inc_name, p2)?;
                        let mut r = Self::new_r0_r1(inc_name, inc_type, extention_type, p, p1);
                        r.set_imm_from_pair(p2);
                        r.set_imm(Some(imm.into()));
                        Ok([r].to_vec())
                    }
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::registers, p2)] =>
                        Ok([Self::new_r0_r1_r2(inc_name, inc_type, extention_type, p, p1, p2)].to_vec()),
                    [_, (Rule::registers, p), (Rule::registers, p1)] => {
                        let fixed_imm = Self::process_fixed_i_imm_value(&inc_name);
                        let mut r = Self::new_r0_r1(inc_name, inc_type, extention_type, p, p1);
                        if let Some(imm) = fixed_imm {
                            r.set_imm(Some(imm.into()));
                        }

                        Ok([r].to_vec())
                    }
                    _ => Err(AsmError::MissingCase((file!(), line!()).into(), Rule::rv_zbb_instructions)),
                }
            }
            Rule::rv_k_crypto_instructions => {
                let rules = extension_inc.into_inner().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
                match rules.as_slice() {
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::registers, p2)] =>
                        Ok([Self::new_r0_r1_r2(inc_name, inc_type, extention_type, p, p1, p2)].to_vec()),
                    [_, (Rule::registers, p), (Rule::registers, p1)] => {
                        let fixed_imm = Self::process_fixed_i_imm_value(&inc_name);
                        let mut r = Self::new_r0_r1(inc_name, inc_type, extention_type, p, p1);
                        if let Some(imm) = fixed_imm {
                            r.set_imm(Some(imm.into()));
                        }

                        Ok([r].to_vec())
                    }
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::integer, p2)] |
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::var_name, p2)] => {
                        let imm = Self::process_shamt_value(&inc_name, p2)?;
                        let mut r = Self::new_r0_r1(inc_name, inc_type, extention_type, p, p1);
                        r.set_imm_from_pair(p2);
                        r.set_imm(Some(imm.into()));
                        Ok([r].to_vec())
                    }
                    _ => Err(AsmError::MissingCase((file!(), line!()).into(), Rule::rv_k_crypto_instructions)),
                }
            }
            Rule::rv_privileged_instructions => {
                let rules = extension_inc.into_inner().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
                match rules.as_slice() {
                    [_, (Rule::registers, p), (Rule::csr, p1), (Rule::var_name, p2)] |
                    [_, (Rule::registers, p), (Rule::csr, p1), (Rule::integer, p2)] => 
                        Ok([Self::new_r0_r1_imm(inc_name, inc_type, extention_type, p, p1, p2)].to_vec()),
                    [_, (Rule::registers, p), (Rule::csr, p1), (Rule::registers, p2)] => 
                        Ok([Self::new_r0_r1_r2(inc_name, inc_type, extention_type, p, p1, p2)].to_vec()),
                    _ => Err(AsmError::MissingCase((file!(), line!()).into(), Rule::rv_privileged_instructions)),
                }
            }
            Rule::rvm32_instructions => {
                let rules = extension_inc.into_inner().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
                match rules.as_slice() {
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::registers, p2)] => 
                        Ok([Self::new_r0_r1_r2(inc_name, inc_type, extention_type, p, p1, p2)].to_vec()),
                    _ => Err(AsmError::MissingCase((file!(), line!()).into(), Rule::rvm32_instructions)),
                }
            }
            Rule::rvm64_128_instructions => {
                let rules = extension_inc.into_inner().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
                match rules.as_slice() {
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::registers, p2)] => 
                        Ok([Self::new_r0_r1_r2(inc_name, inc_type, extention_type, p, p1, p2)].to_vec()),
                    _ => Err(AsmError::MissingCase((file!(), line!()).into(), Rule::rvm64_128_instructions)),
                }
            }
            Rule::rv64a_128a_instructions => {
                let rules = extension_inc.into_inner().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
                match rules.as_slice() {
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::registers, p2)] => 
                        Ok([Self::new_r0_r1_r2(inc_name, inc_type, extention_type, p, p1, p2)].to_vec()),
                    [_, (Rule::registers, p), (Rule::registers, p1)] => 
                        Ok([Self::new_r0_r1(inc_name, inc_type, extention_type, p, p1)].to_vec()),
                    _ => Err(AsmError::MissingCase((file!(), line!()).into(), Rule::rvm64_128_instructions)),
                }
            }
            Rule::rv32a_instructions => {
                let rules = extension_inc.into_inner().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
                match rules.as_slice() {
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::registers, p2)] => 
                        Ok([Self::new_r0_r1_r2(inc_name, inc_type, extention_type, p, p1, p2)].to_vec()),
                    [_, (Rule::registers, p), (Rule::registers, p1)] => 
                        Ok([Self::new_r0_r1(inc_name, inc_type, extention_type, p, p1)].to_vec()),
                    _ => Err(AsmError::MissingCase((file!(), line!()).into(), Rule::rv32a_instructions)),
                }
            }
            Rule::rvf_instructions => {
                let rules = extension_inc.into_inner().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
                match rules.as_slice() {
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::registers, p2)] => 
                        Ok([Self::new_r0_r1_r2(inc_name, inc_type, extention_type, p, p1, p2)].to_vec()),
                    [_, (Rule::registers, p), (Rule::registers, p1)] => 
                        Ok([Self::new_r0_r1(inc_name, inc_type, extention_type, p, p1)].to_vec()),
                    [_, (Rule::registers, p)] => 
                        Ok([Self::new_r0(inc_name, inc_type, extention_type, p)].to_vec()),
                    [_, (Rule::registers, p), (Rule::var_name, p1)] |
                    [_, (Rule::registers, p), (Rule::integer, p1)] => 
                        Ok([Self::new_r0_imm(inc_name, inc_type, extention_type, p, p1)].to_vec()),
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::var_name, p2)] |
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::integer, p2)] => 
                        Ok([Self::new_r0_r1_imm(inc_name, inc_type, extention_type, p, p1, p2)].to_vec()),
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::registers, p2), (Rule::registers, p3)] => 
                        Ok([Self::new_r0_r1_r2_r3(inc_name, inc_type, extention_type, p, p1, p2, p3)].to_vec()),
                    _ => Err(AsmError::MissingCase((file!(), line!()).into(), Rule::rv32a_instructions)),
                }
            }
            Rule::rvf64_128_instructions => {
                let rules = extension_inc.into_inner().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
                match rules.as_slice() {
                    [_, (Rule::registers, p), (Rule::registers, p1)] => 
                        Ok([Self::new_r0_r1(inc_name, inc_type, extention_type, p, p1)].to_vec()),
                    _ => Err(AsmError::MissingCase((file!(), line!()).into(), Rule::rv32a_instructions)),
                }
            }
            Rule::pseudoinstructions => {
                let rules = extension_inc.into_inner().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
                let inc_name_low = inc_name.to_lowercase();
                let inc_name_low_case = inc_name_low.as_str();
                let replace_pseudo = config.get_replace_pseudo_code();
                match rules.as_slice() {
                    [_, (Rule::registers, p), (Rule::symbol, p1)] => {
                        if replace_pseudo {
                            match inc_name_low_case {
                                "la" => Self::la_to_incs(p, p1.as_str()),
                                "lw" => Self::lw_to_incs(p.as_str(), p1.as_str()),
                                "ld" => Self::ld_to_incs(p.as_str(), p1.as_str()),
                                _ => {
                                    Err(AsmError::NoFound((file!(), line!()).into(), format!("cannot convert {inc_name} to base inc")))
                                }
                            }
                        }
                        else {
                            let r = Self::new_r0_imm(inc_name, InstructionTypes::UnKnown, BasicInstructionExtensions::Unknown, p, p1);
                            Ok([r].to_vec())
                        }
                    }
                    [_, (Rule::registers, p), (Rule::registers, p1)] => {
                        if replace_pseudo {
                            match inc_name_low_case {
                                "mv" => {
                                    Self::mv_to_incs(p, p1)
                                }
                                "not" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("xori")?;
                                    let mut r = Self::new_r0_r1(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions, p, p1);
                                    r.set_imm(Some("-1".into()));
                                    Ok([r].to_vec())
                                }
                                "neg" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("sub")?;
                                    let mut r = Self::new_r0(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions, p);
                                    r.r1_name = Some("x0".to_string());
                                    r.r2_name = Some("rs".to_string());
                                    Ok([r].to_vec())
                                }
                                "fsflags" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("csrrw")?;
                                    let mut r = Self::new(inc_name, inc_type, BasicInstructionExtensions::RvPrivilegedInstructions);
                                    r.r0_name = Some(p.as_str().to_string());
                                    r.r1_name = Some("fflags".to_string());
                                    r.r2_name = Some(p1.as_str().to_string());
                                    Ok([r].to_vec())
                                }
                                "fssr" | 
                                "fscsr" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("csrrw")?;
                                    let mut r = Self::new(inc_name, inc_type, BasicInstructionExtensions::RvPrivilegedInstructions);
                                    r.r0_name = Some(p.as_str().to_string());
                                    r.r1_name = Some("fcsr".to_string());
                                    r.r2_name = Some(p1.as_str().to_string());
                                    Ok([r].to_vec())
                                }
                                "save" => {
                                    let la_incs = Self::la_to_incs(p1, &format!("region_reg_{}_address", p.as_str()))?;

                                    let (inc_name2, inc_type2) = Self::get_inc_and_inc_type("sw")?;
                                    let mut r2 = Self::new(inc_name2, inc_type2, BasicInstructionExtensions::BaseIntegerInstructions);
                                    r2.r0_name = Some(p.as_str().to_string());
                                    r2.r1_name = Some(p1.as_str().to_string());
                                    r2.set_imm(Some("0".into()));
                                    Ok(la_incs.iter().chain([r2].iter()).cloned().collect())
                                }
                                "restore" => {
                                    let la_incs = Self::la_to_incs(p1, &format!("region_reg_{}_address", p.as_str()))?;
                                    let (inc_name2, inc_type2) = Self::get_inc_and_inc_type("lw")?;
                                    let mut r2 = Self::new(inc_name2, inc_type2, BasicInstructionExtensions::BaseIntegerInstructions);
                                    r2.r0_name = Some(p.as_str().to_string());
                                    r2.r1_name = Some(p1.as_str().to_string());
                                    r2.set_imm(Some("0".into()));
                                    Ok(la_incs.iter().chain([r2].iter()).cloned().collect())
                                }
                                _ => {
                                    Self::get_pseudo_replacement(inc_name_low_case, [p, p1], config)
                                }
                            }
                        }
                        else {
                            let r = Self::new_r0_r1(inc_name, InstructionTypes::UnKnown, BasicInstructionExtensions::Unknown, p, p1);
                            Ok([r].to_vec())
                        }
                    }
                    [_, (Rule::registers, p), (Rule::var_name, p1)] => {
                        if replace_pseudo {
                            match inc_name_low_case {
                                "beqz" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("beq")?;
                                    let mut r = Self::new_r0_imm(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions, p, p1);
                                    r.r1_name = Some("x0".to_string());
                                    Ok([r].to_vec())
                                }
                                "bnez" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("bne")?;
                                    let mut r = Self::new_r0_imm(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions, p, p1);
                                    r.r1_name = Some("x0".to_string());
                                    Ok([r].to_vec())
                                }
                                "blez" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("ble")?;
                                    let mut r = Self::new_r0_imm(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions, p, p1);
                                    r.r1_name = Some("x0".to_string());
                                    Ok([r].to_vec())
                                }
                                "bgtz" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("bgt")?;
                                    let mut r = Self::new_r0_imm(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions, p, p1);
                                    r.r1_name = Some("x0".to_string());
                                    Ok([r].to_vec())
                                }
                                _ => {
                                    let r = Self::new_r0_imm(inc_name, inc_type, extention_type, p, p1);
                                    Ok([r].to_vec())
                                }
                            }
                        }
                        else {
                            let r = Self::new_r0_imm(inc_name, InstructionTypes::UnKnown, BasicInstructionExtensions::Unknown, p, p1);
                            Ok([r].to_vec())
                        }
                    }
                    [_, (Rule::registers, p), (Rule::single_character, p1)] |
                    [_, (Rule::registers, p), (Rule::integer, p1)] => {
                        if replace_pseudo {
                            let imm = if p1.as_rule() == Rule::integer { pair_to_i64(p1)? as i64 }
                                        else { pair_to_char(p1)? as i64 };
                            match inc_name_low_case {
                                "ld" |
                                "li" => {
                                    Self::li_from_imm(p.as_str(), imm)
                                }
                                "beqz" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("beq")?;
                                    let mut r = Self::new_r0_imm(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions, p, p1);
                                    r.r1_name = Some("x0".to_string());
                                    Ok([r].to_vec())
                                }
                                "bnez" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("bne")?;
                                    let mut r = Self::new_r0_imm(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions, p, p1);
                                    r.r1_name = Some("x0".to_string());
                                    Ok([r].to_vec())
                                }
                                "blez" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("ble")?;
                                    let mut r = Self::new_r0_imm(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions, p, p1);
                                    r.r1_name = Some("x0".to_string());
                                    Ok([r].to_vec())
                                }
                                "bgtz" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("bgt")?;
                                    let mut r = Self::new_r0_imm(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions, p, p1);
                                    r.r1_name = Some("x0".to_string());
                                    Ok([r].to_vec())
                                }
                                _ => {
                                    Self::get_pseudo_replacement(inc_name_low_case, [p, p1], config)
                                }
                            }
                        }
                        else {
                            let r = Self::new_r0_imm(inc_name, InstructionTypes::UnKnown, BasicInstructionExtensions::Unknown, p, p1);
                            Ok([r].to_vec())
                        }

                    }
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::integer, p2)] => {
                        if replace_pseudo {
                            match inc_name_low_case {
                                "bgt" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("bgt")?;
                                    let r = Self::new_r0_r1_imm(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions, p1, p, p2);
                                    Ok([r].to_vec())
                                }
                                "ble" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("bge")?;
                                    let r = Self::new_r0_r1_imm(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions, p1, p, p2);
                                    Ok([r].to_vec())
                                }
                                "bgtu" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("bltu")?;
                                    let r = Self::new_r0_r1_imm(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions, p1, p, p2);
                                    Ok([r].to_vec())
                                }
                                "bleu" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("begu")?;
                                    let r = Self::new_r0_r1_imm(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions, p1, p, p2);
                                    Ok([r].to_vec())
                                }
                                _ => {  // handle other pseudo instructions with 3 parameters, like muli, divi, etc.
                                    Self::get_pseudo_replacement(inc_name_low_case, [p, p1, p2], config)
                                }
                            }
                        }
                        else {
                            let r = Self::new_r0_r1_imm(inc_name, InstructionTypes::UnKnown, BasicInstructionExtensions::Unknown, p, p1, p2);
                            Ok([r].to_vec())
                        }
                    }
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::float, p2)] => {
                        if replace_pseudo {
                            Self::get_pseudo_replacement(inc_name_low_case, [p, p1, p2], config)
                        }
                        else {
                            let r = Self::new_r0_r1_imm(inc_name, InstructionTypes::UnKnown, BasicInstructionExtensions::Unknown, p, p1, p2);
                            Ok([r].to_vec())
                        }
                    }
                    [_, (Rule::registers, p)] => {
                        if replace_pseudo {
                            match inc_name_low_case {
                                "fsflags" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("csrrw")?;
                                    let mut r = Self::new(inc_name, inc_type, BasicInstructionExtensions::RvPrivilegedInstructions);
                                    r.r0_name = Some("x0".to_string());
                                    r.r1_name = Some("fflags".to_string());
                                    r.r2_name = Some(p.as_str().to_string());
                                    Ok([r].to_vec())
                                }
                                "frflags" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("csrrs")?;
                                    let mut r = Self::new(inc_name, inc_type, BasicInstructionExtensions::RvPrivilegedInstructions);
                                    r.r0_name = Some(p.as_str().to_string());
                                    r.r1_name = Some("fflags".to_string());
                                    r.r2_name = Some("x0".to_string());
                                    Ok([r].to_vec())
                                }
                                "frsr" |
                                "frcsr" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("csrrs")?;
                                    let mut r = Self::new(inc_name, inc_type, BasicInstructionExtensions::RvPrivilegedInstructions);
                                    r.r0_name = Some(p.as_str().to_string());
                                    r.r1_name = Some("fcsr".to_string());
                                    r.r2_name = Some("x0".to_string());
                                    Ok([r].to_vec())
                                }
                                "fssr" |
                                "fscsr" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("csrrs")?;
                                    let mut r = Self::new(inc_name, inc_type, BasicInstructionExtensions::RvPrivilegedInstructions);
                                    r.r0_name = Some("x0".to_string());
                                    r.r1_name = Some("fcsr".to_string());
                                    r.r2_name = Some(p.as_str().to_string());
                                    Ok([r].to_vec())
                                }
                                "jr" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("jalr")?;
                                    let mut r = Self::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions);
                                    r.r0_name = Some("x0".to_string());
                                    r.r1_name = Some(p.as_str().to_string());
                                    r.set_imm(Some("0".into()));
                                    Ok([r].to_vec())
                                }
                                "tt_showreg" => {
                                    let reg_name = p.as_str();
                                    let la_incs = Self::la_to_incs_from_str("a0", "tt_fmt")?;
                                    let mv_incs = Self::mv_to_incs_from_str("a1", reg_name)?;
                                    let call_incs = {
                                        let r = Self::new_imm_from_str("call".to_string(), InstructionTypes::UnKnown, BasicInstructionExtensions::PseudoInstructions, "printf");
                                        vec![r]
                                    };
                                    let r = la_incs.iter().chain(mv_incs.iter()).chain(call_incs.iter()).cloned().collect();
                                    Ok(r)
                                }
                                _ => {  // handle other pseudo instructions with 1 parameter, like push, pop, etc.
                                    Self::get_pseudo_replacement(inc_name_low_case, [p], config)
                                }
                            }
                        }
                        else {
                            let r = Self::new_r0(inc_name, InstructionTypes::UnKnown, BasicInstructionExtensions::Unknown, p);
                            Ok([r].to_vec())
                        }
                    }
                    [_, (Rule::var_name, p)] |
                    [_, (Rule::integer, p)] => {
                        if replace_pseudo {
                            match inc_name_low_case {
                                "j" => {
                                    if p.as_rule() == Rule::var_name {
                                        let (inc_name, inc_type) = Self::get_inc_and_inc_type("jal")?;
                                        let mut r = Self::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions);
                                        r.r0_name = Some("x0".to_string());
                                        r.set_imm_from_pair(p);
                                        Ok([r].to_vec())
                                    }
                                    else {
                                        let imm_value = pair_to_i64(p)?;
                                        if Self::is_within_1m(imm_value) {
                                            let (inc_name, inc_type) = Self::get_inc_and_inc_type("jal")?;
                                            let mut r = Self::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions);
                                            r.r0_name = Some("x0".to_string());
                                            r.set_imm_from_pair(p);
                                            Ok([r].to_vec())
                                        }
                                        else {
                                            let (inc_name, inc_type) = Self::get_inc_and_inc_type("auipc")?;
                                            let mut hi = Self::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions);
                                            hi.r0_name = Some("t0".to_string());
                                            hi.set_imm_from_pair(p);
                                            hi.rel_fun = Some("%pcrel_hi".to_string());

                                            let (inc_name2, inc_type2) = Self::get_inc_and_inc_type("jalr")?;
                                            let mut lo = Self::new(inc_name2, inc_type2, BasicInstructionExtensions::BaseIntegerInstructions);
                                            lo.r0_name = Some("x0".to_string());
                                            lo.r1_name = Some("t0".to_string());
                                            lo.set_imm_from_pair(p);
                                            lo.rel_fun = Some("%pcrel_lo".to_string());

                                            Ok([hi, lo].to_vec())
                                        }
                                    }
                                }
                                "call" => {
                                    if p.as_rule() == Rule::var_name {
                                        let r = Self::new_imm(inc_name, inc_type, extention_type, p);
                                        Ok([r].to_vec())
                                    }
                                    else {
                                        let imm_value = pair_to_i64(p)?;
                                        if Self::is_within_1m(imm_value) {
                                            let (inc_name, inc_type) = Self::get_inc_and_inc_type("jal")?;
                                            let mut r = Self::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions);
                                            r.r0_name = Some("ra".to_string());
                                            r.set_imm_from_pair(p);
                                            Ok([r].to_vec())
                                        }
                                        else {
                                            let (low, high) = Self::get_low12_and_high_with_sign_process(imm_value);
                                            let (inc_name, inc_type) = Self::get_inc_and_inc_type("auipc")?;
                                            let mut r = Self::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions);
                                            r.r0_name = Some("rd".to_string());
                                            r.set_imm(Some(high.into()));
                                            r.rel_fun = Some("%pcrel_hi".to_string());

                                            let (inc_name2, inc_type2) = Self::get_inc_and_inc_type("jalr")?;
                                            let mut r2 = Self::new(inc_name2, inc_type2, BasicInstructionExtensions::BaseIntegerInstructions);
                                            r2.r0_name = Some("ra".to_string());
                                            r2.r1_name = Some("ra".to_string());
                                            r2.set_imm(Some(low.into()));
                                            r2.rel_fun = Some("%pcrel_lo".to_string());

                                            Ok([r, r2].to_vec())
                                        }
                                    }
                                }
                                "jal" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("jal")?;
                                    let mut r = Self::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions);
                                    r.r0_name = Some("x1".to_string());
                                    r.set_imm_from_pair(p);
                                    Ok([r].to_vec())
                                }
                                _ => {
                                    Self::get_pseudo_replacement(inc_name_low_case, [p], config)
                                }
                            }
                        }
                        else {
                            let r = Self::new_imm(inc_name, InstructionTypes::UnKnown, BasicInstructionExtensions::Unknown, p);
                            Ok([r].to_vec())
                        }
                    }
                    [_, (Rule::registers, p), (Rule::csr, p1)] => {
                        if replace_pseudo {
                            match inc_name_low_case {
                                "csrr" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("csrrs")?;
                                    let mut r = Self::new(inc_name, inc_type, BasicInstructionExtensions::RvPrivilegedInstructions);
                                    r.r0_name = Some(p.as_str().to_string());
                                    r.r1_name = Some(p1.as_str().to_string());
                                    r.r2_name = Some("x0".to_string());
                                    Ok([r].to_vec())
                                }
                                _ => {
                                    todo!("cannot process pseudo inc: {inc_name}, please consider to add processing logic in csr")
                                }
                            }
                        }
                        else {
                            let r = Self::new_r0_r1(inc_name, InstructionTypes::UnKnown, BasicInstructionExtensions::Unknown, p, p1);
                            Ok([r].to_vec())
                        }                 
                    }
                    [_, (Rule::registers, p), (Rule::symbol, p1), (Rule::registers, p2)] => {
                        if replace_pseudo {
                            match inc_name_low_case {
                                "flw" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("auipc")?;                                    
                                    let mut r = Self::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions);
                                    r.r0_name = Some(p2.as_str().to_string());
                                    r.set_imm_from_pair(p1);
                                    r.rel_fun = Some("%pcrel_hi_la".to_string());
                                    r.operations = EmitOperation::ApplyOffset(0);

                                    let (inc_name2, inc_type2) = Self::get_inc_and_inc_type("flw")?; 
                                    let mut r2 = Self::new(inc_name2, inc_type2, BasicInstructionExtensions::BaseIntegerInstructions);
                                    r2.r0_name = Some(p.as_str().to_string());
                                    r2.r1_name = Some(p2.as_str().to_string());
                                    r2.set_imm_from_pair(p1);
                                    r2.rel_fun = Some("%pcrel_lo_la".to_string());
                                    r2.operations = EmitOperation::ApplyOffset(4);   //the calculation is based on previous instruction's offset to need to substract 4 bytes more. 4 bytes is 1 instruction's length

                                    Ok([r, r2].to_vec())
                                }
                                "fld" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("auipc")?;                                    
                                    let mut r = Self::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions);
                                    r.r0_name = Some(p2.as_str().to_string());
                                    r.set_imm_from_pair(p1);
                                    r.rel_fun = Some("%pcrel_hi_la".to_string());
                                    r.operations = EmitOperation::ApplyOffset(0);

                                    let (inc_name2, inc_type2) = Self::get_inc_and_inc_type("fld")?; 
                                    let mut r2 = Self::new(inc_name2, inc_type2, BasicInstructionExtensions::BaseIntegerInstructions);
                                    r2.r0_name = Some(p.as_str().to_string());
                                    r2.r1_name = Some(p2.as_str().to_string());
                                    r2.set_imm_from_pair(p1);
                                    r2.rel_fun = Some("%pcrel_lo_la".to_string());
                                    r2.operations = EmitOperation::ApplyOffset(4);   //the calculation is based on previous instruction's offset to need to substract 4 bytes more. 4 bytes is 1 instruction's length

                                    Ok([r, r2].to_vec())
                                }
                                "fsw" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("auipc")?;                                    
                                    let mut r = Self::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions);
                                    r.r0_name = Some(p2.as_str().to_string());
                                    r.set_imm_from_pair(p1);
                                    r.rel_fun = Some("%pcrel_hi_la".to_string());
                                    r.operations = EmitOperation::ApplyOffset(0);

                                    let (inc_name2, inc_type2) = Self::get_inc_and_inc_type("fsw")?; 
                                    let mut r2 = Self::new(inc_name2, inc_type2, BasicInstructionExtensions::BaseIntegerInstructions);
                                    r2.r0_name = Some(p.as_str().to_string());
                                    r2.r1_name = Some(p2.as_str().to_string());
                                    r2.set_imm_from_pair(p1);
                                    r2.rel_fun = Some("%pcrel_lo_la".to_string());
                                    r2.operations = EmitOperation::ApplyOffset(4);   //the calculation is based on previous instruction's offset to need to substract 4 bytes more. 4 bytes is 1 instruction's length

                                    Ok([r, r2].to_vec())
                                }
                                "fsd" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("auipc")?;                                    
                                    let mut r = Self::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions);
                                    r.r0_name = Some(p2.as_str().to_string());
                                    r.set_imm_from_pair(p1);
                                    r.rel_fun = Some("%pcrel_hi_la".to_string());
                                    r.operations = EmitOperation::ApplyOffset(0);

                                    let (inc_name2, inc_type2) = Self::get_inc_and_inc_type("fsd")?; 
                                    let mut r2 = Self::new(inc_name2, inc_type2, BasicInstructionExtensions::BaseIntegerInstructions);
                                    r2.r0_name = Some(p.as_str().to_string());
                                    r2.r1_name = Some(p2.as_str().to_string());
                                    r2.set_imm_from_pair(p1);
                                    r2.rel_fun = Some("%pcrel_lo_la".to_string());
                                    r2.operations = EmitOperation::ApplyOffset(4);   //the calculation is based on previous instruction's offset to need to substract 4 bytes more. 4 bytes is 1 instruction's length

                                    Ok([r, r2].to_vec())
                                }
                                _ => Err(AsmError::NoFound((file!(), line!()).into(), format!("cannot convert {inc_name} to base inc in (reg, symbol, rd)")))
                            }
                        }
                        else {
                            let r = Self::new_r0_r1_r2(inc_name, InstructionTypes::UnKnown, BasicInstructionExtensions::Unknown, p, p1, p2);
                            Ok([r].to_vec())
                        }
                    }
                    [_, (Rule::csr, p), (Rule::var_name, p1)] |
                    [_, (Rule::csr, p), (Rule::integer, p1)] => {
                        if replace_pseudo {
                            match inc_name_low_case {
                                "csrwi" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("csrrwi")?;
                                    let mut r = Self::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions);
                                    r.r0_name = Some("x0".to_string());
                                    r.r1_name = Some(p.as_str().to_string());
                                    r.set_imm_from_pair(p1);
                                    Ok([r].to_vec())
                                }
                                _ => {
                                    todo!("cannot process pseudo inc: {inc_name}, please consider to add processing logic in csr +int/identifier")
                                }
                            }
                        }
                        else {
                            let r = Self::new_r0_r1(inc_name, InstructionTypes::UnKnown, BasicInstructionExtensions::Unknown, p, p1);
                            Ok([r].to_vec())
                        }
                    }
                    [_, (Rule::csr, _p), (Rule::registers, _p1)] => 
                        todo!("cannot process pseudo inc: {inc_name}, please consider to add processing logic in csr + reg"),
                    [_] => {
                        if replace_pseudo {
                            match inc_name_low_case {
                                "ret" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("jalr")?;
                                    let mut r = Self::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions);
                                    r.r0_name = Some("x0".to_string());
                                    r.r1_name = Some("x1".to_string());   //x1 is ra (return address)
                                    r.set_imm(Some("0".into()));
                                    Ok([r].to_vec())
                                }
                                "nop" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("addi")?;
                                    let mut r = Self::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions);
                                    r.r0_name = Some("x0".to_string());
                                    r.r1_name = Some("x0".to_string());
                                    r.set_imm(Some("0".into()));
                                    Ok([r].to_vec())
                                }
                                _ => {
                                    match Self::get_pseudo_replacement(inc_name_low_case, MacroArgs::empty(), config) {
                                        Ok(r) => Ok(r),
                                        Err(_) => {
                                            let msg = format!("cannot convert pseudo code '{inc_name}' to base inc");
                                            error_string(msg.clone());
                                            Err(AsmError::NoFound((file!(), line!()).into(), msg))
                                        }
                                    }
                                }
                            }
                        }
                        else {
                            let r = Self::new(inc_name, InstructionTypes::UnKnown, BasicInstructionExtensions::Unknown);
                            Ok([r].to_vec())
                        }
                    }
                    [_, (Rule::registers, p), (Rule::registers, p1), (Rule::var_name, p2)] => {
                        if replace_pseudo {
                            match inc_name_low_case {
                                "ble" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("bge")?;                                    
                                    let mut r = Self::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions);
                                    r.r0_name = Some(p1.as_str().to_string());
                                    r.r1_name = Some(p.as_str().to_string());
                                    r.set_imm_from_pair(p2);
                                    Ok([r].to_vec())
                                }
                                "bgt" => {
                                    let (inc_name, inc_type) = Self::get_inc_and_inc_type("blt")?;                                    
                                    let mut r = Self::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions);
                                    r.r0_name = Some(p1.as_str().to_string());
                                    r.r1_name = Some(p.as_str().to_string());
                                    r.set_imm_from_pair(p2);
                                    Ok([r].to_vec())
                                }
                                _=> {
                                    todo!("cannot process pseudo inc: {inc_name}, please consider to add processing logc in reg + reg + identifier")
                                }
                            }
                        }
                        else {
                            let r = Self::new_r0_r1_imm(inc_name, InstructionTypes::UnKnown, BasicInstructionExtensions::Unknown, p, p1, p2);
                            Ok([r].to_vec())
                        }
                    }
                    _ => { 
                        error_string(format!("does not match happens at {} {}", file!(), line!()));
                        Err(AsmError::MissingCase((file!(), line!()).into(), Rule::pseudoinstructions))
                    }
                }
            }
            _ =>  Err(AsmError::MissingCase((file!(), line!()).into(), Rule::instruction)),
        }?;

        for instruction in instructions.iter_mut() {
            if instruction.source_ranges.name.is_none() {
                instruction.source_ranges.name = inc_name_location;
            }
        }

        Ok(instructions)
    }

    pub (crate) fn set_imm(&mut self, v: Option<Imm>) {
        self.imm = v;
    }

    pub fn get_imm(&self) -> Option<&Imm> {
        self.imm.as_ref()
    }

    /// Get the extension type of the instruction.
    pub fn get_extension_type(&self) -> &BasicInstructionExtensions {
        &self.inc_extensions_and_type
    }

    pub (crate) fn get_alternative_r2_value(&self) -> Option<u32> {
        let op = OpCode::from_str(&self.name)?;
        match op {
            OpCode::Fsqrts |
            OpCode::Fsqrtd |
            OpCode::Fsqrth |
            OpCode::Fclasss |
            OpCode::Fclassd |
            OpCode::Fclassh |
            OpCode::Fmvxw |
            OpCode::Fmvwx |
            OpCode::Fmvxd |
            OpCode::Fmvdx |
            OpCode::Fmvxh |
            OpCode::Fmvhx |
            OpCode::Fcvths |
            OpCode::Fcvthw |
            OpCode::Fcvtwh => Some(0b0000_0),
            OpCode::Fcvtbf16s => Some(0b0100_0),
            OpCode::Fcvtsbf16 => Some(0b0011_0),
            OpCode::Fcvtsd |
            OpCode::Fcvtwus |
            OpCode::Fcvtwud |
            OpCode::Fcvtdwu |
            OpCode::Fcvtswu |
            OpCode::Fcvtwuh |
            OpCode::Fcvthd |
            OpCode::Fcvthwu => Some(0b0000_1),
            OpCode::Fcvtdl |
            OpCode::Fcvtld |
            OpCode::Fcvtls |
            OpCode::Fcvtsl |
            OpCode::Fcvtsh |
            OpCode::Fcvtdh |
            OpCode::Fcvtqh |
            OpCode::Fcvtlh |
            OpCode::Fcvthl => Some(0b000_10),
            OpCode::Fcvtlus |
            OpCode::Fcvtslu |
            OpCode::Fcvtlud |
            OpCode::Fcvtdlu |
            OpCode::Fcvtluh |
            OpCode::Fcvthlu |
            OpCode::Fcvthq => Some(0b000_11),
            _ => None,
        }
    }

    pub (crate) fn get_inc_and_inc_type(inc:&str) -> Result<(String, InstructionTypes), AsmError> {
        let new_inc_name = inc.to_string();
        let inc_type = OpCode::get_instruction_type_from_string(&new_inc_name)?;
        Ok((new_inc_name, inc_type))
    }

    /// Check whether the immediate value is within the near range (-2048 to 2047)
    fn is_near(imm:i64) -> bool {
        imm >= -2048 && imm<= 2047
    }

    pub (crate) fn is_within_1m(imm:i64) -> bool {
        imm >= ( -1 * 2i64.pow(21) ) && imm <= (2i64.pow(21) - 4)
    }

    pub (crate) fn get_low12_and_high(imm:i64) -> (u32, u32) {
        let low = (imm & 0xfff) as u32;
        let high = ((imm >> 12) as u32) & (0xffff_ffff>>12);
        (low, high)
    }

    pub (crate) fn get_low12_and_high_with_sign_process(imm:i64) -> (u32, u32) {
        let (low, high) = Self::get_low12_and_high(imm);
        if low & 0x800 != 0 {
            (low, high+1)
        }
        else {
            (low, high)
        }
    }

    pub (crate) fn new_r0_r1(inc_name:String, inc_type: InstructionTypes, extention_type:BasicInstructionExtensions, p:&Pair<Rule>, p1:&Pair<Rule>) -> Self {
        let mut r = Self::new(inc_name, inc_type, extention_type);
        r.set_r0(p);
        r.set_r1(p1);
        r
    }

    pub (crate) fn new_r0_r1_option(inc_name:String, inc_type: InstructionTypes, extention_type:BasicInstructionExtensions, p:&Pair<Rule>, p1:&Pair<Rule>, option:&Pair<Rule>) -> Self {
        let mut r = Self::new_r0_r1(inc_name, inc_type, extention_type, p, p1);
        r.set_option(option);
        r
    }

    pub (crate) fn new_r0_r1_imm(inc_name:String, inc_type: InstructionTypes, extention_type:BasicInstructionExtensions, p:&Pair<Rule>, p1:&Pair<Rule>, p2:&Pair<Rule>) -> Self {
        let mut r = Self::new(inc_name, inc_type, extention_type);
        r.set_r0(p);
        r.set_r1(p1);
        r.set_imm_from_pair(p2);
        r
    }

    fn new_r0_r1_rel(inc_name:String, inc_type: InstructionTypes, extention_type:BasicInstructionExtensions, p:&Pair<Rule>, p1:&Pair<Rule>, p2:&Pair<Rule>) -> Self {
        let mut r = Self::new(inc_name, inc_type, extention_type);
        r.set_r0(p);
        r.set_r1(p1);
        r.set_rel_fun(p2);
        r
    }

    fn new_r0_r1_r2(inc_name:String, inc_type: InstructionTypes, extention_type:BasicInstructionExtensions, p:&Pair<Rule>, p1:&Pair<Rule>, p2:&Pair<Rule>) -> Self {
        let mut r = Self::new(inc_name, inc_type, extention_type);
        r.set_r0(p);
        r.set_r1(p1);
        r.set_r2(p2);
        r
    }

    fn new_r0_r1_imm_option(inc_name:String, inc_type: InstructionTypes, extention_type:BasicInstructionExtensions, p:&Pair<Rule>, p1:&Pair<Rule>, p2:&Pair<Rule>, option:&Pair<Rule>) -> Self {
        let mut r = Self::new_r0_r1_imm(inc_name, inc_type, extention_type, p, p1, p2);
        r.set_option(option);
        r
    }

    fn new_r0_r1_r2_option(inc_name:String, inc_type: InstructionTypes, extention_type:BasicInstructionExtensions, p:&Pair<Rule>, p1:&Pair<Rule>, p2:&Pair<Rule>, option:&Pair<Rule>) -> Self { 
        let mut r = Self::new_r0_r1_r2(inc_name, inc_type, extention_type, p, p1, p2);
        r.set_option(option);
        r
    }

    fn new_r0_r1_r2_r3(inc_name:String, inc_type: InstructionTypes, extention_type:BasicInstructionExtensions, p:&Pair<Rule>, p1:&Pair<Rule>, p2:&Pair<Rule>, p3:&Pair<Rule>) -> Self {
        let mut r = Self::new(inc_name, inc_type, extention_type);
        r.set_r0(p);
        r.set_r1(p1);
        r.set_r2(p2);
        r.set_r3(p3);
        r
    }

    fn new_r0_r1_r2_r3_option(inc_name:String, inc_type: InstructionTypes, extention_type:BasicInstructionExtensions, p:&Pair<Rule>, p1:&Pair<Rule>, p2:&Pair<Rule>, p3:&Pair<Rule>, option:&Pair<Rule>) -> Self {
        let mut r = Self::new_r0_r1_r2_r3(inc_name, inc_type, extention_type, p, p1, p2, p3);
        r.set_option(option);
        r
    }

    pub (crate) fn new_r0_imm(inc_name:String, inc_type: InstructionTypes, extention_type:BasicInstructionExtensions, p:&Pair<Rule>, p1:&Pair<Rule>) -> Self {
        let mut r = Self::new(inc_name, inc_type, extention_type);
        r.set_r0(p);
        r.set_imm_from_pair(p1);
        r
    }

    pub (crate) fn new_r0_rel(inc_name:String, inc_type: InstructionTypes, extention_type:BasicInstructionExtensions, p:&Pair<Rule>, p1:&Pair<Rule>) -> Self {
        let mut r = Self::new(inc_name, inc_type, extention_type);
        r.set_r0(p);
        r.set_rel_fun(p1);
        r
    }

    pub fn new(inc_name:String, inc_type: InstructionTypes, extention_type:BasicInstructionExtensions) -> Self {
        Self { name : inc_name, 
            inc_type, 
            inc_extensions_and_type : extention_type, 
            r0_name : None,
            r1_name : None,
            r2_name : None,
            r3_name: None,
            imm : None,
            rel_fun: None,
            option : None,
            operations : EmitOperation::None,
            label_virtual_address : 0,
            source_ranges: InstructionSourceRanges::default(),
            is_generate : true,
            external_symbol : String::new(),
        }
    }

    pub (crate) fn set_is_generate(&mut self, v:bool) {
        self.is_generate = v
    }

    pub (crate) fn new_r0(inc_name:String, inc_type: InstructionTypes, extention_type:BasicInstructionExtensions, p:&Pair<Rule>) -> Self {
        let mut r = Self::new(inc_name, inc_type, extention_type);
        r.set_r0(p);
        r
    }

    pub (crate) fn new_r0_option(inc_name:String, inc_type: InstructionTypes, extention_type:BasicInstructionExtensions, p:&Pair<Rule>, option:&Pair<Rule>) -> Self {
        let mut r = Self::new_r0(inc_name, inc_type, extention_type, p);
        r.set_option(option);
        r
    }

    pub (crate) fn set_rel_fun(&mut self, p:&Pair<Rule>) {
        let inner = p.to_owned().into_inner().collect::<Vec<_>>();
        if inner.len() > 1 {
            let rel_fun_pair = &inner[0];
            let imm_pair = &inner[1];
            self.rel_fun = Some(rel_fun_pair.as_str().to_string());
            self.source_ranges.rel_fun = Some(Self::get_source_range_from_pair(rel_fun_pair));
            self.set_imm(Some(imm_pair.as_str().into()));
            self.source_ranges.imm = Some(Self::get_source_range_from_pair(imm_pair));
        }
        else {
            self.rel_fun = Some(p.as_str().to_string());
            self.source_ranges.rel_fun = Some(Self::get_source_range_from_pair(p));
        }
    }

    pub (crate) fn new_imm(inc_name:String, inc_type: InstructionTypes, extention_type:BasicInstructionExtensions, p:&Pair<Rule>) -> Self {
        let mut r = Self::new(inc_name, inc_type, extention_type);
        r.set_imm_from_pair(p);
        r
    }

    pub (crate) fn new_imm_from_str(inc_name:String, inc_type: InstructionTypes, extention_type:BasicInstructionExtensions, imm_str:&str) -> Self {
        let mut r = Self::new(inc_name, inc_type, extention_type);
        r.set_imm_value(imm_str);
        r
    }

    pub (crate) fn set_option(&mut self, p:&Pair<Rule>) {
        self.option = Some(p.as_str().to_string());
        self.source_ranges.option = Some(Self::get_source_range_from_pair(p));
    }

    pub fn set_r0(&mut self, p:&Pair<Rule>) {
        self.r0_name = Some(p.as_str().to_string());
        self.source_ranges.r0 = Some(Self::get_source_range_from_pair(p));
    }

    pub fn set_r1(&mut self, p:&Pair<Rule>) {
        self.r1_name = Some(p.as_str().to_string());
        self.source_ranges.r1 = Some(Self::get_source_range_from_pair(p));
    }

    pub fn set_r2(&mut self, p:&Pair<Rule>) {
        self.r2_name = Some(p.as_str().to_string());
        self.source_ranges.r2 = Some(Self::get_source_range_from_pair(p));
    }

    pub fn set_r3(&mut self, p:&Pair<Rule>) {
        self.r3_name = Some(p.as_str().to_string());
        self.source_ranges.r3 = Some(Self::get_source_range_from_pair(p));
    }

    pub fn set_imm_from_pair(&mut self, p:&Pair<Rule>) {
        self.set_imm_value(p.as_str());
        self.source_ranges.imm = Some(Self::get_source_range_from_pair(p));
    }

    pub fn set_imm_value(&mut self, v:&str) {
        self.set_imm(Some(v.into()))
    }

    fn get_inc_extension_type(pair:&Pair<Rule>) -> Result<BasicInstructionExtensions, AsmError> {
        let rule = pair.as_rule();
        let rule_str = format!("{rule:?}");
        let r = BasicInstructionExtensions::from_str(&rule_str)
                                                .ok_or(AsmError::NoFound((file!(), line!()).into(), format!("cannot find {rule_str} as BasicInstructionExtensions")))?;
        Ok(r)
    }

    fn get_inc(pair:&Pair<Rule>) -> Result<String, AsmError> {
        if let Some(inc_name) = pair.to_owned().into_inner().find_first_tagged("inc_name")
        {
            let r = inc_name.as_str().to_string();
            Ok(r)
        }
        else {
            if pair.as_node_tag().is_some() && pair.as_node_tag().unwrap() == "inc_name" {
                let r = pair.as_str().to_string();
                Ok(r)
            }
            else {
                let str = format!("cannot find inc_name in {pair:?}, consider to add #inc_name= in the pest file");
                error_string(str.to_string());
                Err(AsmError::NoFound((file!().to_string(), line!()).into(), str))
            }
        }
    }

    /// get inc's name and convert to upper case, like ADD, SUB
    pub fn get_name(&self) -> String {
        self.name.clone().to_uppercase()
    }

    /// get the original instruction name (e.g. "vadd.vv", "lw")
    pub fn get_instruction_name(&self) -> &str {
        &self.name
    }

    /// get the instruction option string (e.g. "v0.t")
    pub fn get_instruction_option(&self) -> Option<&str> {
        self.option.as_deref()
    }

    /// Determine element width from instruction name (e.g. "vadd.vv" has no width suffix -> use SEW)
    pub fn get_elem_width_from_name(name: &str) -> Option<usize> {
        let lower = name.to_lowercase();
        if lower.contains("1024") { return Some(128); }
        if lower.contains("512") { return Some(64); }
        if lower.contains("256") { return Some(32); }
        if lower.contains("128") { return Some(16); }
        if lower.contains("64") { return Some(8); }
        if lower.contains("32") { return Some(4); }
        if lower.contains("16") { return Some(2); }
        if lower.contains("8") { return Some(1); }
        None
    }

    pub fn get_op_code(&self) -> Result<OpCode, AsmError> {
        if let Some(op) = OpCode::from_str(&self.name) {
            Ok(op)
        }
        else {
            Err(AsmError::NoFound((file!(), line!()).into(), format!("cannot find op code from input '{n}', check if OpCode has '{n}'", n = self.name)))
        }
    }

    /// if the instruction has %pcrel_hi etc. which requires PC as offset
    pub (crate) fn need_offset(&self) -> bool {
        if let Ok(op) = self.get_op_code() {
            if !op.need_offset_compute_address() {               //op do not need offset, but rel_fun might need, so check more
                if let Some(f) = self.rel_fun.as_ref() {
                    match f.as_str() {
                        "%pcrel_hi_la" |
                        "%pcrel_lo_la" |
                        "%pcrel_hi" | 
                        "%pcrel_lo" => true,
                        _ => false,
                    }
                }
                else {
                    false 
                }
            }
            else {
                true 
            }
        }
        else {
            false   //if can't find opcode, then return "does not need offset to compute"
        }
    }

    /// get imm value from imme string, not considering the label or register case
    /// if imm cannot be converted to i32, return None
    pub (crate) fn get_imm_value_from_imm_string(&self) -> Option<i32> {
        if let Some(imm_str) = self.get_imm().and_then(|x| x.to_string_option()) {
            if let Ok(v) = get_i32_from_str(&imm_str) {
                Some(v)
            }
            else {
                None
            }
        }
        else {
            None
        }
    }

    /// get imm value from imm string, if imm is label, then get label's address, 
    /// if imm is register, then get register's value
    pub fn get_imm_value(&self, labels:&LabelOffsetTable, regs:&Register, external_symbols: &Vec<ExternalLabel>, current_offset:usize) -> Result<u32, AsmError> {
        if let Some(_imm_str) = self.imm.as_ref() {
            let pc = if self.need_offset() { current_offset as u32 } else { 0 };

            // process if the symbol is external symbol, the offset is 0 or the offset stored in the external symbol table
            if let Some(imm_str) = self.imm.as_ref().and_then(|x| x.to_string_option()) {
                if let Some(symbol) = external_symbols.iter().find(|x| x.get_name() == imm_str) {
                    let symbol_value = symbol.get_value();
                    let v = symbol_value.get_address().map(|v| v as u32);  //just to check if address is available
                    if let Some(addr) = v {
                        return self.process_rel_fun(addr, pc);
                    } else {
                        let msg = format!("external symbol '{imm_str}' is not resolved to a PLT address");
                        return Err(AsmError::NoFound((file!(), line!()).into(), msg));
                    }
                }
            }
            
            if let Some(_f) = self.rel_fun.as_ref() {
                if self.use_rel_fun64() {
                    let imm_address = self.get_imm_u64_from_str(labels, regs, current_offset)?;
                    self.process_rel_fun64(imm_address, pc as u64)
                }
                else {
                    let imm_address = self.get_imm_from_str(labels, regs, current_offset)?;
                    self.process_rel_fun(imm_address, pc)
                }
            }
            else {
                let imm_address = self.get_imm_asi64_from_str(labels, regs, current_offset)?;
                let v = (pc as i64 + imm_address) as u32;
                Ok(v)
            }
        }
        else {
            Ok(0)
        }
    }

    fn use_rel_fun64(&self) -> bool {
        self.rel_fun
            .as_ref()
            .map(|x| {
                matches!(
                    x.to_lowercase().as_str(),
                    "%hi20" | "%lo12" | "%c1" | "%c2" | "%c3"
                )
            })
            .unwrap_or(false)
    }

    fn pcrel_lo(imm_address:u32, pc:u32) -> u32 {
        let offset = (imm_address as i64) - (pc as i64);
        let r = (offset << 52) >> 52 as i32;
        r as u32
    }

    fn pcrel_hi(imm_address:u32, pc:u32) -> u32 {
        let offset = (imm_address as i64) - (pc as i64);
        // Add 0x800 to round properly when the low 12 bits overflow into high bits
        let r = (((offset + 0x800) >> 12) & 0xFFFFF) as i32;
        r as u32
    }

    fn process_rel_fun64(&self, imm_address:u64, _pc:u64) -> Result<u32, AsmError> {
        let hi32 = (imm_address >> 32) as u32;
        if let Some(f) = self.rel_fun.as_ref() {
            match f.to_lowercase().as_str() {
                "%hi20" => { Ok(hi32.wrapping_add(0x800) >> 12) }
                "%lo12" => { Ok(hi32 & 0xfff) }
                "%c1" => { 
                    let r = ((imm_address >> 21) & 0x7ff) as u32; 
                    Ok(r)
                }
                "%c2" => {
                    let r = ((imm_address >> 10) & 0x7ff) as u32;
                    Ok(r)
                }
                "%c3" => {
                    let r = (imm_address & 0x3ff) as u32;
                    Ok(r)
                }
                _ => {
                    let err_str = format!("Cannot process operator {f} for imm process");
                    error_string(err_str.to_string());
                    Err(AsmError::NotSupportedOperation((file!(), line!()).into(), err_str))
                }
            }
        }
        else {
            Ok(imm_address as u32)
        }
    }

    fn process_rel_fun(&self, imm_address:u32, pc:u32) -> Result<u32, AsmError> {
        if let Some(f) = self.rel_fun.as_ref() {
            match f.to_lowercase().as_str() {
                "%pcrel_hi" => Ok(Self::pcrel_hi(imm_address, pc)),
                "%pcrel_lo" => Ok(Self::pcrel_lo(imm_address, pc)),
                "%hi" => {
                    let r = imm_address.wrapping_add(0x800) >> 12;
                    Ok(r)
                }
                "%lo" => {
                    let r = imm_address & 0xfff;
                    Ok(r)
                }                    
                "%pcrel_hi_la" => {
                    let (hi, _low) = Self::decompose_la(imm_address, pc);
                    Ok(hi as u32)                        
                }
                "%pcrel_lo_la" => {
                    let (_hi, low) = Self::decompose_la(imm_address, pc);
                    Ok(low as u32)
                }
                _ => {
                    let err_str = format!("Cannot process operator {f} for imm process");
                    error_string(err_str.to_string());
                    Err(AsmError::NotSupportedOperation((file!(), line!()).into(), err_str))
                }
            }
        }
        else {
            Ok(imm_address)
        }
    }

    fn decompose_la(target: u32, pc: u32) -> (i32, i32) {
        let offset = target.wrapping_sub(pc) as i32; // Treat as signed
        let auipc_imm = (offset + 0x800) >> 12;
        let addi_imm = offset - (auipc_imm << 12);
        (auipc_imm, addi_imm)
    }

    /// Decomposes an offset into AUIPC and ADDI immediates.
    /// 
    /// Given:
    ///    offset = target_address - PC
    /// It computes:
    ///    imm_auipc = (offset + 0xFFF) >> 12
    ///    imm_addi  = offset - (imm_auipc << 12)
    /// 
    /// If imm_addi > 0x7FF (i.e. it would not fit in a signed 12-bit field),
    /// then we adjust by increasing imm_auipc by 1 and subtracting 0x1000 from imm_addi.
    fn decompose_offset(offset: i32) -> (i32, i32) {
        // Compute the AUIPC immediate by rounding up:
        let mut imm_auipc = (offset + 0xFFF) >> 12;
        // Compute the lower 12-bit immediate (raw remainder)
        let mut imm_addi = offset - (imm_auipc << 12);

        // If the lower part doesn't fit in a signed 12-bit immediate,
        // adjust by incrementing the upper immediate and subtracting 4096.
        if imm_addi > 0x7FF {
            imm_auipc += 1;
            imm_addi -= 0x1000;
        }
        (imm_auipc, imm_addi)
    }

    pub (crate) fn get_imm_from_str(&self, labels:&LabelOffsetTable, regs:&Register, offset:usize) -> Result<u32, AsmError> {
        let imm =self.get_imm().unwrap().to_string_option().unwrap();
        if imm.is_empty() {
            Ok(0)
        }
        else {
            if labels.contains_key(&imm) {  //get label's address
                if let Some(v) = labels.get_offset(&imm, offset, None) {
                    Ok(v as u32)
                }
                else {
                    let err_str = format!("cannot get label '{imm}' address from label table");
                    error_string(err_str.clone());
                    Err(AsmError::NoFound((file!(), line!()).into(), err_str))
                }
            } 
            else if regs.is_register_name(imm.clone()) {
                regs.get_register_value(Some(&imm)).map(|x| x as u32)
            }
            else { //convert imm to u32 
                if let Some(rel_fun) = self.get_rel_fun() {
                    let rel_fun = rel_fun.to_lowercase();
                    if (rel_fun == "%hi" || rel_fun == "%lo")
                        && get_i64_from_str(&imm).is_err()
                    {
                        if let Ok(v) = imm.trim().replace('_', "").parse::<f32>() {
                            return Ok(v.to_bits());
                        }
                    }
                }
                Self::imm_str_to_u32(&imm)
            }
        }
    }

    pub (crate) fn get_imm_u64_from_str(&self, labels:&LabelOffsetTable, regs:&Register, offset:usize) -> Result<u64, AsmError> {
        let imm = self.get_imm().unwrap().to_string_option().unwrap();
        if imm.is_empty() {
            Ok(0)
        }
        else if labels.contains_key(&imm) {
            if let Some(v) = labels.get_offset(&imm, offset, None) {
                Ok(v as u64)
            }
            else {
                let err_str = format!("cannot get label '{imm}' address from label table");
                error_string(err_str.clone());
                Err(AsmError::NoFound((file!(), line!()).into(), err_str))
            }
        }
        else if regs.is_register_name(imm.clone()) {
            regs.get_register_value(Some(&imm)).map(|x| x as u64)
        }
        else {
            if self.use_rel_fun64() && get_i64_from_str(&imm).is_err() {
                if let Ok(v) = imm.trim().replace('_', "").parse::<f64>() {
                    return Ok(v.to_bits());
                }
            }

            if let Ok(v) = get_u64_from_str(&imm) {
                Ok(v)
            }
            else {
                get_i64_from_str(&imm)
                    .map(|v| v as u64)
                    .map_err(|_| AsmError::ConversionFailed((file!(), line!()).into(), format!("convert {imm} to u64 failed")))
            }
        }
    }

    pub (crate) fn get_imm_asi64_from_str(&self, labels:&LabelOffsetTable, regs:&Register, offset:usize) -> Result<i64, AsmError> {
        let imm =self.get_imm().unwrap().to_string_option().unwrap();
        if imm.is_empty() {
            Ok(0)
        }
        else {
            if labels.contains_key(&imm) {  //get label's address
                let v = labels.get_offset(&imm, offset, None).unwrap();
                Ok(v as i64)
            } 
            else if regs.is_register_name(imm.clone()) {
                regs.get_register_value(Some(&imm)).map(|x| x as i64)
            }
            else { //convert imm to i64
                Self::imm_str_to_i64(&imm)
            }
        }
    }

    pub (crate) fn imm_to_u32(&self) -> Result<u32, AsmError> {
        if let Some(imm) = self.get_imm().and_then(|x| x.to_string_option()) {
            Self::imm_str_to_u32(&imm)
        }
        else {
            Ok(0)
        }
    }

    pub (crate) fn imm_to_i64(&self) -> Result<i64, AsmError> {
        if let Some(imm) = self.get_imm().and_then(|x| x.to_string_option()) {
            Self::imm_str_to_i64(&imm)
        }
        else {
            Ok(0)
        }
    }

    fn imm_str_to_u32(imm:&String) -> Result<u32, AsmError> {
        get_u32_from_str(imm)
            .map_err(|_| AsmError::ConversionFailed((file!(), line!()).into(), format!("convert {imm} to u32 failed")))       
    }

    fn imm_str_to_i64(imm:&String) -> Result<i64, AsmError> {
        get_i64_from_str(imm)
            .map_err(|_| AsmError::ConversionFailed((file!(), line!()).into(), format!("convert {imm} to i64 failed")))
    }

    pub fn get_instruction_size(&self) -> Result<usize, AsmError> {
        if self.is_pseodu_inc() {
            Ok(4 * 2)   //  return as 2 instruction size
        }
        else if self.inc_extensions_and_type == BasicInstructionExtensions::RvVInstructions {
            Ok(4)
        }
        else {
            let r = self.get_op_code()?.get_instruction_size() as usize;
            Ok(r)
        }
    }

    pub fn get_register_id_as_string(&self, reg:InstructionRegisterName, regs:&Register) -> Result<String, AsmError> {
        match reg {
            InstructionRegisterName::Rs0 |
            InstructionRegisterName::Rd => {
                let v = regs.get_register_value(self.r0_name.as_ref())?;
                Ok(format!("{v:0>5b}"))
            }
            InstructionRegisterName::Rs1 => {
                let v = regs.get_register_value(self.r1_name.as_ref())?;
                Ok(format!("{v:0>5b}"))
            }
            InstructionRegisterName::Rs2 => {
                if let Some(r2_value) = self.get_alternative_r2_value() {
                    Ok(format!("{r2_value:0>5b}"))
                }
                else {
                    let v = regs.get_register_value(self.r2_name.as_ref())?;
                    Ok(format!("{v:0>5b}"))
                }
            }
            InstructionRegisterName::Rs3 => {
                let v = regs.get_register_value(self.r3_name.as_ref())?;
                Ok(format!("{v:0>5b}"))
            }
        }
    }

    pub fn get_compact_register_id_as_string(&self, reg:InstructionRegisterName, regs:&Register) -> Result<String, AsmError> {
        match reg {
            InstructionRegisterName::Rs0 |
            InstructionRegisterName::Rd => {
                let v = regs.get_register_compressed_value(self.r0_name.as_ref())?;
                Ok(format!("{v:0>3b}"))
            }
            InstructionRegisterName::Rs1 => {
                let v = regs.get_register_compressed_value(self.r1_name.as_ref())?;
                Ok(format!("{v:0>3b}"))
            }
            InstructionRegisterName::Rs2 => {
                let v = regs.get_register_compressed_value(self.r2_name.as_ref())?;
                Ok(format!("{v:0>3b}"))
            }
            InstructionRegisterName::Rs3 => {
                let v = regs.get_register_compressed_value(self.r3_name.as_ref())?;
                Ok(format!("{v:0>3b}"))
            }
        }
    }

    pub fn has_reg(&self, reg:InstructionRegisterName) -> bool {
        match reg {
            InstructionRegisterName::Rd |
            InstructionRegisterName::Rs0 => self.r0_name.is_some(),
            InstructionRegisterName::Rs1 => self.r1_name.is_some(),
            InstructionRegisterName::Rs2 => self.r2_name.is_some(),
            InstructionRegisterName::Rs3 => self.r3_name.is_some(),
        }
    }

    pub fn get_option_value(&self) -> Option<u32> {
        if let Some(n) = self.option.as_ref() {
            match n.to_uppercase().as_str() {
                "RNE" => Some(0b000),
                "RTZ" => Some(0b001),
                "RDN" => Some(0b010),
                "RUP" => Some(0b011),
                "RMM" => Some(0b100),
                "DYN" => Some(0b111),
                _ => None,
            }
        }
        else {
            None
        }
    }

    pub fn get_option_value_str(&self) -> Option<String> {
        if let Some(n) = self.get_option_value() {
            let v = format!("{n:0>3b}");
            Some(v)
        }
        else {
            None 
        }
    }

    pub fn is_atomic(&self) -> bool {
        let ty = &self.inc_extensions_and_type;
        match ty {
            BasicInstructionExtensions::Rv32aInstructions |
            BasicInstructionExtensions::Rv64a128aInstructions => true,
            _ => false,
        }
    }

    pub fn get_atomic_option(&self) -> Result<u32, AsmError> {
        if self.is_atomic() {
            if let Some(n) = self.option.as_ref() {
                match n.to_uppercase().as_str() {
                    "AQ" => Ok(0b10),
                    "REL" => Ok(0b01),
                    "AQREL" => Ok(0b11),
                    _ => Err(AsmError::WrongType((file!(), line!()).into(), format!("cannot find {n} as a valid atomic option value"))),
                }
            }
            else {
                Ok(0)
            }
        }
        else {
            Err(AsmError::IncompatibleType((file!(), line!()).into()))
        }
    }

    pub fn is_pseodu_inc(&self) -> bool {
        match self.inc_extensions_and_type {
            BasicInstructionExtensions::PseudoInstructions => true,
            _ => false,
        }
    }

    pub fn set_virtual_address(&mut self, v:u32) {
        self.label_virtual_address = v;
    }

    pub fn get_virtual_address(&self) -> u32 {
        self.label_virtual_address
    }

    pub fn convert_to_compact(&self) -> Option<Instruction> {
        self.convert_to_compact_for_target(8)
    }

    /// Try a semantics-preserving C conversion for build_target 4 (RV32) or 8 (RV64).
    pub fn convert_to_compact_for_target(&self, build_target:u8) -> Option<Instruction> {
        use InstructionRegisterName::{Rd, Rs1, Rs2};
        let op = self.get_op_code().ok()?;
        if !op.has_compact_equ() || !matches!(build_target, 4 | 8)
            || self.rel_fun.is_some() || self.option.is_some() || self.r3_name.is_some()
        {
            return None;
        }
        let regs = Register::new();
        let register_id = |name:&Option<String>, floating:bool| {
            let name = name.as_ref()?;
            let valid = if floating { Register::is_float_register_name(name) }
                else { Register::is_integer_register_name(name) };
            if valid { regs.get_register_value(Some(name)).ok() } else { None }
        };
        let floating = matches!(op, OpCode::Flw | OpCode::Fld | OpCode::Fsw | OpCode::Fsd);
        let r0 = register_id(&self.r0_name, floating);
        let r1 = register_id(&self.r1_name, false);
        let r2 = register_id(&self.r2_name, false);
        let prime = |id:Option<i32>| id.is_some_and(|id| (8..=15).contains(&id));
        let nonzero = |id:Option<i32>| id.is_some_and(|id| (1..=31).contains(&id));
        let imm = self.imm_to_i64().ok()?;
        let signed6 = (-32..=31).contains(&imm);
        let make = |name, r0, r1, imm| Some(self.compact_inc(name, r0, r1, imm));
        match op {
            OpCode::Lw | OpCode::Ld | OpCode::Flw | OpCode::Fld
            | OpCode::Sw | OpCode::Sd | OpCode::Fsw | OpCode::Fsd => {
                if (matches!(op, OpCode::Ld | OpCode::Sd) && build_target != 8)
                    || (matches!(op, OpCode::Flw | OpCode::Fsw) && build_target != 4)
                {
                    return None;
                }
                let (stack, regular, scale) = match op {
                    OpCode::Lw => ("c.lwsp", "c.lw", 4),
                    OpCode::Ld => ("c.ldsp", "c.ld", 8),
                    OpCode::Flw => ("c.flwsp", "c.flw", 4),
                    OpCode::Fld => ("c.fldsp", "c.fld", 8),
                    OpCode::Sw => ("c.swsp", "c.sw", 4),
                    OpCode::Sd => ("c.sdsp", "c.sd", 8),
                    OpCode::Fsw => ("c.fswsp", "c.fsw", 4),
                    OpCode::Fsd => ("c.fsdsp", "c.fsd", 8),
                    _ => unreachable!(),
                };
                if r0.is_none() || imm < 0 || imm % scale != 0
                    || (matches!(op, OpCode::Lw | OpCode::Ld) && !nonzero(r0))
                {
                    None
                } else if r1 == Some(2) && imm <= 63 * scale {
                    make(stack, Some(Rd), None, Some(imm))
                } else if prime(r0) && prime(r1) && imm <= 31 * scale {
                    make(regular, Some(Rd), Some(Rs1), Some(imm))
                } else {
                    None
                }
            }
            OpCode::Addi => {
                if r0 == Some(0) && r1 == Some(0) && imm == 0 {
                    make("c.nop", None, None, None)
                } else if !nonzero(r0) {
                    None
                } else if r1 == Some(0) && signed6 {
                    make("c.li", Some(Rd), None, Some(imm))
                } else if imm == 0 && nonzero(r1) {
                    make("c.mv", Some(Rd), Some(Rs1), None)
                } else if r0 == r1 && signed6 && imm != 0 {
                    make("c.addi", Some(Rd), None, Some(imm))
                } else if r0 == Some(2) && r1 == Some(2) && imm != 0
                    && (-512..=496).contains(&imm) && imm % 16 == 0
                {
                    make("c.addi16sp", Some(Rd), None, Some(imm))
                } else if prime(r0) && r1 == Some(2) && (4..=1020).contains(&imm) && imm % 4 == 0 {
                    make("c.addi4spn", Some(Rd), None, Some(imm))
                } else {
                    None
                }
            }
            OpCode::Addiw if build_target == 8 && nonzero(r0) && r0 == r1 && signed6 =>
                make("c.addiw", Some(Rd), None, Some(imm)),
            OpCode::Lui if nonzero(r0) && r0 != Some(2) => {
                let field = super::compact_inc::lui_immediate(imm)?;
                make("c.lui", Some(Rd), None, Some(field))
            }
            OpCode::Slli | OpCode::Srli | OpCode::Srai if nonzero(r0) && r0 == r1 => {
                // The scalar parser stores SRAI's funct bits with its shamt.
                let shift = if op == OpCode::Srai { imm.checked_sub(0x400)? } else { imm };
                let max_shift = i64::from(build_target) * 8 - 1;
                if !(1..=max_shift).contains(&shift)
                    || (op != OpCode::Slli && !prime(r0))
                {
                    return None;
                }
                let name = match op {
                    OpCode::Slli => "c.slli",
                    OpCode::Srli => "c.srli",
                    _ => "c.srai",
                };
                make(name, Some(Rd), None, Some(shift))
            }
            OpCode::Andi if prime(r0) && r0 == r1 && signed6 =>
                make("c.andi", Some(Rd), None, Some(imm)),
            OpCode::Add if nonzero(r0) => {
                if r1 == Some(0) && nonzero(r2) {
                    make("c.mv", Some(Rd), Some(Rs2), None)
                } else if r2 == Some(0) && nonzero(r1) {
                    make("c.mv", Some(Rd), Some(Rs1), None)
                } else if r0 == r1 && nonzero(r2) {
                    make("c.add", Some(Rd), Some(Rs2), None)
                } else if r0 == r2 && nonzero(r1) {
                    make("c.add", Some(Rd), Some(Rs1), None)
                } else {
                    None
                }
            }
            OpCode::And | OpCode::Or | OpCode::Xor | OpCode::Sub | OpCode::Addw | OpCode::Subw => {
                if !prime(r0) || (matches!(op, OpCode::Addw | OpCode::Subw) && build_target != 8) {
                    return None;
                }
                let source = if r0 == r1 && prime(r2) {
                    Rs2
                } else if r0 == r2 && prime(r1) && !matches!(op, OpCode::Sub | OpCode::Subw) {
                    Rs1
                } else {
                    return None;
                };
                let name = match op {
                    OpCode::And => "c.and",
                    OpCode::Or => "c.or",
                    OpCode::Xor => "c.xor",
                    OpCode::Sub => "c.sub",
                    OpCode::Addw => "c.addw",
                    _ => "c.subw",
                };
                make(name, Some(Rd), Some(source), None)
            }
            OpCode::Jal if (-2048..=2046).contains(&imm) && imm % 2 == 0 => {
                if r0 == Some(0) {
                    make("c.j", None, None, Some(imm))
                } else if r0 == Some(1) && build_target == 4 {
                    make("c.jal", None, None, Some(imm))
                } else {
                    None
                }
            }
            OpCode::Jalr if imm == 0 && nonzero(r1) => match r0 {
                Some(0) => make("c.jr", Some(Rs1), None, None),
                Some(1) => make("c.jalr", Some(Rs1), None, None),
                _ => None,
            },
            OpCode::Beq | OpCode::Bne if (-256..=254).contains(&imm) && imm % 2 == 0 => {
                let source = if prime(r0) && r1 == Some(0) {
                    Rd
                } else if r0 == Some(0) && prime(r1) {
                    Rs1
                } else {
                    return None;
                };
                make(if op == OpCode::Beq { "c.beqz" } else { "c.bnez" }, Some(source), None, Some(imm))
            }
            OpCode::Ebreak => make("c.ebreak", None, None, None),
            _ => None,
        }
    }

    fn compact_inc(&self, name:&str, r0_source:Option<InstructionRegisterName>, r1_source:Option<InstructionRegisterName>, imm:Option<i64>) -> Instruction {
        let operand = |source| match source {
            Some(InstructionRegisterName::Rd | InstructionRegisterName::Rs0) =>
                (self.r0_name.clone(), self.source_ranges.r0),
            Some(InstructionRegisterName::Rs1) => (self.r1_name.clone(), self.source_ranges.r1),
            Some(InstructionRegisterName::Rs2) => (self.r2_name.clone(), self.source_ranges.r2),
            Some(InstructionRegisterName::Rs3) => (self.r3_name.clone(), self.source_ranges.r3),
            None => (None, None),
        };
        let mut result = self.clone();
        result.name = name.to_string();
        result.inc_type = InstructionTypes::COMPACT;
        result.inc_extensions_and_type = BasicInstructionExtensions::CompactInstructions;
        (result.r0_name, result.source_ranges.r0) = operand(r0_source);
        (result.r1_name, result.source_ranges.r1) = operand(r1_source);
        result.r2_name = None;
        result.r3_name = None;
        result.source_ranges.r2 = None;
        result.source_ranges.r3 = None;
        result.imm = imm.map(|value| value.to_string().into());
        if result.imm.is_none() {
            result.source_ranges.imm = None;
        }
        result
    }

    pub fn get_r0(&self) -> Option<&String> {
        self.r0_name.as_ref()
    }

    pub fn get_r1(&self) -> Option<&String> {
        self.r1_name.as_ref()
    }

    pub fn get_r2(&self) -> Option<&String> {
        self.r2_name.as_ref()
    }

    pub fn get_r3(&self) -> Option<&String> {
        self.r3_name.as_ref()
    }

    /// get a list of register name which has been used in the instruction
    pub fn get_registers_used(&self) -> Vec<String> {
        let mut r = Vec::default();
        if let Some(n) = self.r0_name.as_ref() {
            r.push(n.to_string())
        }

        if let Some(n) = self.r1_name.as_ref() {
            r.push(n.to_string())
        }

        if let Some(n) = self.r2_name.as_ref() {
            r.push(n.to_string())
        }

        if let Some(n) = self.r3_name.as_ref() {
            r.push(n.to_string())
        }

        r
    }

    create_set_reg_value_fn!(set_r0_value, r0_name);
    create_set_reg_value_fn!(set_r1_value, r1_name);
    create_set_reg_value_fn!(set_r2_value, r2_name);
    create_set_reg_value_fn!(set_r3_value, r3_name);

    pub fn set_inc_name(&mut self, v:&str) {
        self.name = v.to_string()
    }

    /// switch registers a and b in current instruction
    pub (crate) fn switch_registers(&mut self, a:&str, b:&str) {
        if self.r0_name.as_ref().map_or(false, |x| x == a) {
            self.set_r0_value(b);
        }
        if self.r1_name.as_ref().map_or(false, |x| x == a) {
            self.set_r1_value(b);
        }
        if self.r2_name.as_ref().map_or(false, |x| x == a) {
            self.set_r2_value(b);
        }
        if self.r3_name.as_ref().map_or(false, |x| x == a) {
            self.set_r3_value(b);
        }
    }

    /// replace registers in the instruction with the tuple of (name, replacement) list
    pub fn replace_registers_from_tuple(&mut self, tuples:&Vec<(String, String)>) {
        if let Some((name, replacement)) = tuples.iter().find(|(a, _)| self.is_r0_name_same(a)) {
            self.replace_r0(name, replacement);
        }

        if let Some((name, replacement)) = tuples.iter().find(|(a, _)| self.is_r1_name_same(a)) {
            self.replace_r1(name, replacement);
        }

        if let Some((name, replacement)) = tuples.iter().find(|(a, _)| self.is_r2_name_same(a)) {
            self.replace_r2(name, replacement);
        }

        if let Some((name, replacement)) = tuples.iter().find(|(a, _)| self.is_r3_name_same(a)) {
            self.replace_r3(name, replacement);
        }
    }

    create_register_name_same_fn!(is_r0_name_same, r0_name);
    create_register_name_same_fn!(is_r1_name_same, r1_name);
    create_register_name_same_fn!(is_r2_name_same, r2_name);
    create_register_name_same_fn!(is_r3_name_same, r3_name);

    pub fn replace_r0(&mut self, name:&str, replacement:&str) {
        if self.is_r0_name_same(name) {
            self.set_r0_value(replacement);
        }
    }

    pub fn replace_r1(&mut self, name:&str, replacement:&str) {
        if self.is_r1_name_same(name) {
            self.set_r1_value(replacement);
        }
    }

    pub fn replace_r2(&mut self, name:&str, replacement:&str) {
        if self.is_r2_name_same(name) {
            self.set_r2_value(replacement);
        }
    }

    pub fn replace_r3(&mut self, name:&str, replacement:&str) {
        if self.is_r3_name_same(name) {
            self.set_r3_value(replacement);
        }
    }

    /// replace imm value (can be lable in jump instruction) in the instruction
    /// replace only happens when the name is the same as the imm value
    pub fn replace_imm(&mut self, name:&str, replacement:&str) {
        if let Some(r) = self.get_imm().and_then(|x| x.get_value()) {
            if r == name {
                self.set_imm_value(replacement);
            }
        }
    }

    /// get a list of register name which has been used in the instruction, 
    /// the list will not contain duplicate register name
    /// the result keep the register order (r0, r1, ...)
    pub (crate) fn get_involved_regs(&self) -> Vec<String> {
        let mut r = Vec::default();
        if let Some(n) = self.r0_name.as_ref() {
            if !r.iter().any(|x| x==n) {
                r.push(n.to_string())
            }
        }

        if let Some(n) = self.r1_name.as_ref() {
            if !r.iter().any(|x| x==n) {
                r.push(n.to_string())
            }
        }

        if let Some(n) = self.r2_name.as_ref() {
            if !r.iter().any(|x| x==n) {
                r.push(n.to_string())
            }
        }

        if let Some(n) = self.r3_name.as_ref() {
            if !r.iter().any(|x| x==n) {
                r.push(n.to_string())
            }
        }

        r 
    }

    pub fn get_external_label(&self) -> &ExternalSymbol {
        & self.external_symbol
    }

    pub fn set_external_label(&mut self, v:ExternalSymbol) {
        self.external_symbol = v;
    }

    /// set instruction's external symbol from its immediate value field, the external symbol is set in the immediate value field
    pub fn tag_imm_as_external_symbol(&mut self) {
        let imm = self.get_imm().and_then(|x| x.get_value()).unwrap();
        self.set_external_label(imm);
    }

    pub fn has_external_symbol(&self, external_symbols: &Vec<ExternalLabel>) -> bool {
        if let Some(imm_str) = self.get_imm().and_then(|x| x.get_value()) {
            external_symbols.contains(& imm_str.into())
        }
        else {
            false
        }
    }

    pub fn get_rel_fun(&self) -> Option<&String> {
        self.rel_fun.as_ref()
    }

    /// Check if the instruction is a PC-relative high immediate instruction like `pcrel_hi`
    pub fn is_pcrel_hi(&self) -> bool {
        self.get_rel_fun().map_or(false, |x| x == "%pcrel_hi")
    }

    /// Check if the instruction is a PC-relative low immediate instruction like `pcrel_lo`
    pub fn is_pcrel_lo(&self) -> bool {
        self.get_rel_fun().map_or(false, |x| x == "%pcrel_lo")
    }

    /// get label in pcrel_hi
    pub fn get_related_label(&self) -> Option<String> {
        if self.is_pcrel_hi() {
            self.get_imm().and_then(|x| x.to_string_option())
        }
        else {
            None
        }
    }

    /// get offset from label table if inc is pcrel_lo
    pub fn get_label_offset(&self, labels:&LabelOffsetTable, offset:usize) -> Option<usize> {
        if self.is_pcrel_lo() {
            let label = self.get_rel_fun().unwrap();
            labels.get_offset(label, offset, None)
        }
        else {
            None
        }
    }

    /// convert instruction to a tuple for e-graph
    pub fn to_egraph_tuple(&self) -> (OpCode, Option<String>, Option<String>, Option<String>, Option<String>, Option<Imm>) {
        let op_code = self.get_op_code().unwrap();
        (op_code, 
            self.r0_name.clone(), 
            self.r1_name.clone(), 
            self.r2_name.clone(), 
            self.r3_name.clone(), 
            self.imm.clone()
        )
    }

    /// set registers and immediate value in the instruction
    pub fn set_registers_imm(&mut self, r0:Option<String>, r1:Option<String>, r2:Option<String>, r3:Option<String>, imm:Option<Imm>) {
        self.r0_name = r0;
        self.r1_name = r1;
        self.r2_name = r2;
        self.r3_name = r3;
        self.imm = imm;
    }
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let label_virtual_address_str = if (self.label_virtual_address as i32) < 0 {
                format!("0x{:x}, {}, or {}", self.label_virtual_address, self.label_virtual_address, self.label_virtual_address as i32)
        }
        else {
            format!("0x{:x}, {}", self.label_virtual_address, self.label_virtual_address)
        };

        write!(f, "Instruction {{ {} ({}, {}, {}, {}), {imm}{rel_fun}{option}({ty:?}, {extension_ty:?}), {op}label_virAdd/real_value: {label_virtual_address_str} }}", 
            self.name.to_uppercase(), 
            option_to_string(self.r0_name.as_ref()), option_to_string(self.r1_name.as_ref()), 
            option_to_string(self.r2_name.as_ref()), option_to_string(self.r3_name.as_ref()),
            imm = match self.imm.as_ref() {
                Some(n) => format!("imm : {n:?}, "),
                None => String::new(),
            },
            rel_fun = match self.rel_fun.as_ref() {
                Some(n) => format!("rel_fun : {n}, "),
                None => String::new(), 
            },
            option = match self.option.as_ref() {
                Some(n) => format!("option: {n}, "),
                None => String::new(),
            },
            op = match self.operations { 
                EmitOperation::None => String::new(), 
                _ => format!("ExtraOps: {:?}, ", self.operations),
            },             
            ty=self.inc_type, extension_ty=self.inc_extensions_and_type)
    }
}

impl GenerateCode for Instruction {
    fn generate_code_string(&self) -> String {
        let mut code = format!("{} ", self.name);

        let is_vector = self.inc_extensions_and_type == BasicInstructionExtensions::RvVInstructions;
        let is_vector_memory = is_vector && self.name.ends_with(".v")
            && self.r2_name.is_none() && self.r3_name.is_none() && self.imm.is_none()
            && (super::vector_incs::vector_pest::parse_vload_kind(&self.name).is_some()
                || super::vector_incs::vector_pest::parse_vstore_kind(&self.name).is_some());

        if is_vector_memory {
            code = format!("{code}{}, ({})",
                option_to_string2(self.r0_name.as_ref(), "", ""),
                option_to_string2(self.r1_name.as_ref(), "", ""));
        }
        else if self.name == "sw" || self.name == "lw" {
            if let Some(rel_fun) = self.get_rel_fun() {
                code = format!("{code} {}, {rel_fun}({})({})", 
                    option_to_string2(self.r0_name.as_ref(), "", ""), 
                    self.get_imm().unwrap().get_value().unwrap(), 
                    option_to_string2(self.r1_name.as_ref(), "", ""));
            }
            else {
                code = format!("{code} {}, {}({})", 
                    option_to_string2(self.r0_name.as_ref(), "", ""), 
                    self.get_imm().unwrap().get_value().unwrap(), 
                    option_to_string2(self.r1_name.as_ref(), "", ""));
            }
        }
        else {
            let regs = vec![self.r0_name.as_ref(), self.r1_name.as_ref(), self.r2_name.as_ref(), self.r3_name.as_ref()];
            let reg_strs = regs.iter().filter_map(|x| x.map(|y| y.to_string())).collect::<Vec<String>>();
            code = format!("{code}{}", reg_strs.join(", "));
            
            if self.imm.is_some() {
                let imm = self.get_imm().unwrap();
                if self.rel_fun.is_some() {
                    code = format!("{code}, {}( {} )", self.rel_fun.as_ref().unwrap(), imm.get_value().unwrap())
                } 
                else {
                    code = if reg_strs.is_empty() {
                                format!("{code} {}", imm.get_value().unwrap())
                            } else {
                                format!("{code}, {}", imm.get_value().unwrap())
                            }
                }
            }
        }
        
        if is_vector {
            if let Some(option) = self.option.as_ref() {
                code.push_str(", ");
                code.push_str(option);
            }
        }

        code
    }
}

fn option_to_string(option:Option<&String>) -> String {
    option_to_string2(option, "_", "")
}

fn option_to_string2(option:Option<&String>, none_value:&str, prefix:&str) -> String {
    match option {
        Some(n) => format!("{prefix}{n}"),
        None => none_value.to_owned(),
    }
}

impl std::hash::Hash for Instruction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.r0_name.hash(state);
        self.r1_name.hash(state);
        self.r2_name.hash(state);
        self.r3_name.hash(state);
        self.imm.hash(state);
        self.rel_fun.hash(state);
        self.option.hash(state);
    }
}

impl Eq for Instruction {

}

impl Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.generate_code_string())
    }
}

#[cfg(test)]
mod vector_serialization_tests {
    use super::*;
    use crate::r5asm::vector_incs::vector_emitter::emit_vector_instruction;

    #[test]
    fn vector_assembly_round_trip_preserves_configuration_and_memory_operands() {
        let mut config = CodeGenConfiguration::default();
        let registers = Register::new();
        for source in [
            "vsetvli t4, t3, e32, m1, ta, ma",
            "vsetivli t4, 5, e32, m1, tu, mu",
            "vle32.v v8, (t0)",
            "vadd.vv v10, v8, v9",
            "vadd.vv v10, v8, v9, v0.t",
            "vse32.v v10, (t2)",
        ] {
            let original = Instruction::from_string(source, &mut config)
                .expect("parse original vector instruction");
            let rendered = original[0].generate_code_string();
            let reparsed = Instruction::from_string(&rendered, &mut config)
                .unwrap_or_else(|error| panic!("cannot reparse '{rendered}': {error:?}"));
            assert_eq!(original[0].option, reparsed[0].option, "{source}");
            let original_bytes = emit_vector_instruction(&original[0], &registers, 0)
                .expect("encode original instruction")[0].to_vec();
            let reparsed_bytes = emit_vector_instruction(&reparsed[0], &registers, 0)
                .expect("encode reparsed instruction")[0].to_vec();
            assert_eq!(original_bytes, reparsed_bytes, "{source}");
            if source.contains('(') {
                assert!(rendered.contains('('), "{rendered}");
            }
        }
    }
}