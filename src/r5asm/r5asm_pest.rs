use core_utils::number::get_i64_from_str;
use pest::iterators::Pair;
use pest_derive::Parser;
use std::collections::HashMap;
use super::asm_error::AsmError;
use super::label_offset::*;
use super::external_label::*;
use core_utils::debug::*;
use core_utils::traits::generate_code::GenerateCode;

use super::{code_gen_config::CodeGenConfiguration, directive::Directive, instruction::Instruction, register::Register};

pub fn from_pair_template<T>(pair:&Pair<Rule>, rule:Rule, config:&mut CodeGenConfiguration, f:fn(Vec<(Rule, Pair<Rule>)>, &mut CodeGenConfiguration) -> Result<T, AsmError> ) -> Result<T, AsmError> {
    if pair.as_rule() == rule {
        let inner = pair.to_owned().into_inner();
        let rules : Vec<(Rule, Pair<Rule>)> = inner.map(|x| (x.as_rule(), x)).collect();
        f(rules, config)
    }
    else {
        println!("expected rule = {:?} which cannot match {:#?} so cannot continue in from_pair_template", rule, pair);
        Err(AsmError::ParsingConversionError((file!(), line!()).into(), format!("expected rule = {:?} which cannot match {:#?}", rule, pair)) )
    }
}

pub fn from_pair_vec_template<T, T2>(pair:&Pair<Rule>, rule:Rule, config:&mut CodeGenConfiguration, f:fn(Pair<Rule>, &mut CodeGenConfiguration)->T2, f2:fn(Vec<T2>) -> T) -> Result<T, AsmError> {
    if pair.as_rule() == rule {
        let inner = pair.to_owned().into_inner();
        let rules : Vec<T2> = inner.map(|x| f(x, config) ).collect();
        Ok(f2(rules))
    }
    else {
        println!("{:?} cannot match {:#?}", rule, pair);
        Err(AsmError::ParsingConversionError((file!(), line!()).into(), format!("expected rule = {:?} which cannot match {:#?}", rule, pair)) )

    }
}

pub fn from_pair_vec_null_fn_template<T2>(pair:&Pair<Rule>, rule:Rule, config:&mut CodeGenConfiguration, f:fn(Pair<Rule>, &mut CodeGenConfiguration)->T2) -> Result<Vec<T2>, AsmError> {
    from_pair_vec_template(pair, rule, config, f, |x | { x })
}

pub fn pair_to_i64(pair:&Pair<Rule>) -> Result<i64, AsmError> {
    let err = AsmError::ConversionFailed((file!(), line!()).into(), format!("cannot convert '{}' to i64", pair.as_str()));
    get_i64_from_str(pair.as_str()).map_err(|_| err)
}

pub fn pair_to_char(pair:&Pair<Rule>) -> Result<char, AsmError> {
    let inner =pair.to_owned()
                                            .into_inner()
                                            .into_iter()
                                            .map(|x| (x.as_rule(), x))
                                            .collect::<Vec<_>>();
    match inner.as_slice() {
        [] => {
            let s = pair.as_str().replace("'", "");
            if s.len() == 1 {
                Ok(s.chars().nth(0).unwrap())
            }
            else {
                Err(AsmError::GeneralError((file!(), line!()).into(), format!("the string '{s}' should have only 1 char but have more than 1 character")))
            }
        }
        [(Rule::escape_char, p)] => {
            match p.as_str() {
                "n" => Ok('\n'),
                "r" => Ok('\r'),
                "t" => Ok('\t'),
                "a" => Ok('\x07'),
                "b" => Ok('\x08'),
                "f" => Ok('\x0c'),
                "v" => Ok('\x0b'),
                _ => {
                    let err_str = format!("need to add logic to process escape char '{}' when pair to char", p.as_str());
                    error_string(err_str.to_string());
                    Err(AsmError::NoFound((file!(), line!()).into(), err_str))
                }
            }
        }
        [(Rule::hex_code, p)] |
        [(Rule::unicode_code_4, p)] |
        [(Rule::unicode_code_8, p)] => {
            let hex_value = p.as_str();
            let error = AsmError::ParsingConversionError((file!(), line!()).into(), format!("cannot convert hex value '{}' to char", hex_value));
            let v = u32::from_str_radix(hex_value, 16).map_err(|_| error.clone())?;
            char::from_u32(v).ok_or(error)
        }
        _ => {
            let err_str = format!("need to add logic in pair to char, the pair is {pair:?}");
            error_string(err_str.to_string());
            Err(AsmError::NoFound((file!().to_string(), line!()).into(), err_str))
        }
    }
}

#[derive(Parser)]
#[grammar = "../riscv_asm_lib/src/r5asm/r5asm.pest"]
pub struct R5AsmParser;

pub type EquTable = HashMap<String, String>;


#[derive(Debug, Clone, PartialEq)]
pub enum InstructionTypes {
    R,
    U,
    I,
    /// B type and can be also called SB
    B,
    S,
    /// J type can be used called UJ
    J,

    COMPACT,

    UnKnown,  //psuedo code or extended code
}

pub (crate) type OffsetInSection = usize;

/// SectionItem2 is a wrapper for SectionItem with offset in the section
/// it is used to keep track of the offset of the item in the section
#[derive(Debug, Clone)]
pub struct SectionItem2 {
    offset : OffsetInSection,
    item : SectionItem, 
}

impl SectionItem2 {
    pub fn new(offset:OffsetInSection, item:SectionItem) -> Self {
        Self { offset, item }
    }

    pub fn is_label(&self) -> bool {
        self.item.is_label()
    }

    pub fn get_related_label(&self) -> Option<String> {
        self.item.get_related_label()
    }

    pub fn get_label(&self) -> Option<&String> {
        self.item.get_label()
    }

    pub fn get_label_or_tag(&self) -> Option<&String> {
        self.item.get_label_or_tag()
    }

    /// get external lable which is directive
    pub fn get_external_label(&self) -> Option<&String> {
        self.item.get_directive()
            .and_then(|x| x.get_extern_label())
    }

    /// get external symbol in the instruction
    pub fn get_external_symbol(&self) -> Option<&String> {
        self.item.get_inc()
            .and_then(|x| Some(x.get_external_label()))
    }

    /// get global label which is directive
    pub fn get_global_label(&self) -> Option<&String> {
        self.item.get_directive()
            .and_then(|x| x.get_global_label())
    }

    pub fn get_size_directive(&self) -> Option<(String, String)> {
        let r = self.item.get_directive()
            .and_then(|x| x.get_size_directive());
        r
    }

    pub fn is_inc(&self) -> bool {
        self.item.is_inc()
    }

    pub fn is_pseodu_inc(&self) -> bool {
        self.item.is_pseodu_inc()
    }

    pub fn is_directive(&self) -> bool {
        self.item.is_directive()
    }

    pub fn get_directive(&self) -> Option<&Directive> {
        self.item.get_directive()
    }

    pub fn get_align(&self) -> Option<u32> {
        self.item.get_align()
    }

    pub fn set_align(&mut self, v:u32) {
        self.item.set_align(v)
    }

    pub fn get_inc(&self) -> Option<&Instruction> {
        self.item.get_inc()
    }

    pub fn get_inc_mut(&mut self) -> Option<&mut Instruction> {
        self.item.get_inc_mut()
    }

    pub fn get_offset(&self) -> usize {
        self.offset
    }

    pub fn set_offset(&mut self, offset:usize) {
        self.offset = offset;
    }

    pub fn get_item(&self) -> &SectionItem {
        &self.item
    }

    pub fn set_inc(&mut self, inc:Instruction) {
        self.item = SectionItem::Instruction(inc)
    }

    /// get the offset the value in the segment
    pub fn get_imm_value(&mut self, labels:&LabelOffsetTable, regs:&Register, external_symbols:&Vec<ExternalLabel>) -> Result<u32, AsmError> {
        let current_offset = self.get_offset2(labels);
        if let Some(inc) = self.get_inc_mut() {
            let mut v = inc.get_imm_value(labels, regs, external_symbols, current_offset)?;

            if inc.has_external_symbol(external_symbols) {
                inc.tag_imm_as_external_symbol();
            }
            
            //perform additional compute on value
            v = self.perform_additional_compute(v)?;
            Ok(v)
        }
        else {
            let err_str = format!("cannot get imm value from non-instruction item: {self:?}");
            error_string(format!("only instruction can perform get imm value"));
            Err(AsmError::GeneralError((file!(), line!()).into(), err_str) )
        }
    }

    /// get offset with reference to the label offset table
    fn get_offset2(&self, labels:&LabelOffsetTable) -> usize {
        let offset = self.get_offset();
        if let Some(label) = self.get_label_or_tag() {
            let related_label = self.get_inc().and_then(|x| x.get_related_label());
            labels.get_offset(label, offset, related_label.as_ref()).unwrap_or(offset)
        }
        else {
            offset
        }
    }

    fn perform_additional_compute(&self, v:u32) -> Result<u32, AsmError> {
        let inc = self.get_inc()
            .ok_or(AsmError::NoFound((file!(), line!()).into(), format!("cannot find inc from {self:?}")))?;
        match inc.operations {
            EmitOperation::SubstractCurrentOffset(n) => {
                let r = (v as isize - self.offset as isize + n as isize) as u32;
                Ok(r as u32)
            }
            EmitOperation::ApplyOffset(n) => {
                let r = (v as isize + n as isize) as u32;
                Ok(r as u32)
            }

            EmitOperation::None => Ok(v),
        }
    }

    pub fn set_padding_data(&mut self, size:u32) {
        match self.item {
            SectionItem::Directive(ref mut n) => {
                n.set_padding_data(size);
            }
            _ => (),
        }
    }

    /// get inc's virtual address, this is the consolidate value from instruction. 
    /// this value can be imm value for instruction or address
    /// this value is to be used in the machine code generation
    pub fn get_virtual_address(&self) -> Option<u32> {
        self.get_inc()
            .and_then(|x| Some(x.label_virtual_address) )
    }
}

impl GenerateCode for SectionItem2 {
    fn generate_code_string(&self) -> String {
        self.item.generate_code_string()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SectionItem {
    Lable(String),
    Instruction(Instruction),
    Directive(Directive),
}

impl SectionItem {
    pub fn new_label(s:&str) -> Self {
        let ss = if s.ends_with(":") {
            s.trim_end_matches(':')
        }
        else {
            s
        };

        Self::Lable(ss.to_string())
    }
    
    pub fn from_pair(pair:&Pair<Rule>, config:&mut CodeGenConfiguration) -> Result<Vec<Self>, AsmError> {
        match pair.as_rule() {
            Rule::label => {
                let label = Self::Lable(pair.to_owned().into_inner().nth(0).unwrap().as_str().to_string());
                let vec = [label].to_vec();
                Ok(vec)
            }
            Rule::instruction => {
                let incs = Instruction::from_pair(pair, config)?;
                let r = incs.into_iter().map(|x| Self::Instruction(x)).collect::<Vec<_>>();
                Ok(r)
            }
            Rule::directive => {
                let directive = Self::Directive(Directive::from_pair(pair, config)?);
                let vec = [directive].to_vec();
                Ok(vec)
            } 
            _ => Err(AsmError::MissingCase((file!(), line!()).into(), Rule::section)),
        }
    }

    pub fn get_size(&self) -> Result<usize, AsmError> {
        match self {
            Self::Lable(_) => Ok(0),
            Self::Instruction(n) => n.get_instruction_size(),
            Self::Directive(n) => n.get_directive_size(),
        }
    }

    pub fn is_label(&self) -> bool {
        match self {
            Self::Lable(_) => true,
            _ => false,
        }
    }

    pub fn get_label(&self) -> Option<&String> {
        match self {
            Self::Lable(n) => Some(n), 
            _ => None,
        }
    }

    pub fn set_label(&mut self, v:&str) {
        match self {
            Self::Lable(value) => {
                *value = v.to_string()
            }
            _ => ()
        }
    }

    pub fn is_directive(&self) -> bool {
        match self {
            Self::Directive(_) => true,
            _ => false,
        }
    }

    pub fn get_directive(&self) -> Option<&Directive> {
        match self {
            Self::Directive(n) => Some(n),
            _ => None,
        }
    }

    pub fn get_directive_mut(&mut self) -> Option<&mut Directive> {
        match self {
            Self::Directive(n) => Some(n),
            _ => None,
        }
    }

    pub fn get_align(&self) -> Option<u32> {
        self.get_directive().and_then(|x| x.get_align())
    }

    pub fn set_align(&mut self, v:u32) {
        if let Some(n) = self.get_directive_mut() {
            n.set_align(v);
        }
    }

    pub fn is_inc(&self) -> bool {
        match self {
            Self::Instruction(_) => true,
            _ => false,
        }
    }

    pub fn get_inc(&self) -> Option<&Instruction> {
        match self {
            Self::Instruction(n) => Some(n),
            _ => None,
        }
    }

    pub fn get_inc_mut(&mut self) -> Option<&mut Instruction> {
        match self {
            Self::Instruction(n) => Some(n),
            _ => None,
        }
    }

    /// get inc name if it is a inc type, the inc name is upper case, like ADD, SUB, etc.
    pub fn get_inc_name(&self) -> Option<String> {
            if let Some(inc) = self.get_inc() {
                Some(inc.get_name().clone())
            }
            else {
                None
            }
        }

    pub fn is_pseodu_inc(&self) -> bool {
        if let Some(inc) = self.get_inc() {
            inc.is_pseodu_inc()
        }
        else {
            false
        }
    }

    /// get related label
    pub fn get_related_label(&self) -> Option<String> {
        if let Some(inc) = self.get_inc() {
            inc.get_related_label()
        }
        else {
            None
        }
    }

    /// get label or tag
    /// label is the label, tag is the %pcrel_hi or %pcrel_lo
    pub fn get_label_or_tag(&self) -> Option<&String> {
        match self {
            Self::Lable(_n) => self.get_label(),
            Self::Directive(_n) => None,
            Self::Instruction(n) if n.is_pcrel_hi() || n.is_pcrel_lo() => {
                n.get_rel_fun()
            }
            _ => None,
        }
    }
}

impl GenerateCode for SectionItem {
    fn generate_code_string(&self) -> String {
        match self {
            Self::Directive(n) => n.generate_code_string(),
            Self::Lable(n) => format!("{}:", n.to_string()),
            Self::Instruction(n) => n.generate_code_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EmitOperation {
    None,
    SubstractCurrentOffset(i32),
    ApplyOffset(i32),
}

pub enum InstructionRegisterName {
    Rd,
    Rs0,
    Rs1,
    Rs2,
    Rs3,
}