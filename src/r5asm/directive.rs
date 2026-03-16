use pest::iterators::Pair;
use core_utils::string::*;
use core_utils::number::*;
use parser_lib::expr_lang::*;
use parser_lib::string_format::formatted_string::FormattedString;

use super::asm_error::AsmError;
use core_utils::traits::generate_code::GenerateCode;
use super::{code_gen_config::CodeGenConfiguration, machinecode::MachineCode, r5asm_pest::Rule};

/// Represents the names of directives in R5ASM assembly language.
/// These directives are used to control the assembly process, define data, and set various options.
#[derive(Debug, Clone, PartialEq)]
pub enum DirectiveName {
    Align,
    File,
    Globl,
    Local,
    Common,
    Ident,
    Size,
    String,
    ASCIIZ,
    Equ,
    Option,
    Byte,
    TwoByte,
    FourByte,
    EightByte,
    Short,
    Long,
    Quad,
    Half,
    Float,
    Double,
    DWord,
    Word,
    DtpRelWord,
    DtpRelDWorld,
    SLEB128,
    ULEB128,
    P2Align,
    BAlign,
    Zero,
    Space,
    VariantCC,
    Attribute,
    End,
    Extern,
    Type,
}

impl DirectiveName {
    pub fn from_str(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            ".align" => DirectiveName::Align,
            ".file" => DirectiveName::File,
            ".local" => DirectiveName::Local,
            ".globl" => DirectiveName::Globl,
            ".common" => DirectiveName::Common,
            ".comm" => DirectiveName::Common,
            ".ident" => DirectiveName::Ident,
            ".size" => DirectiveName::Size,
            ".string" => DirectiveName::String,
            ".asciz" => DirectiveName::ASCIIZ,
            ".equ" => DirectiveName::Equ,
            ".option" => DirectiveName::Option,
            ".byte" => DirectiveName::Byte,
            ".2byte" => DirectiveName::TwoByte,
            ".4byte" => DirectiveName::FourByte,
            ".8byte" => DirectiveName::EightByte,
            ".short" => DirectiveName::Short,
            ".long" => DirectiveName::Long,
            ".quad" => DirectiveName::Quad,
            ".half" => DirectiveName::Half,
            ".float" => DirectiveName::Float,
            ".double" => DirectiveName::Double,
            ".dword" => DirectiveName::DWord,
            ".word" => DirectiveName::Word,
            ".dtprelword" => DirectiveName::DtpRelWord,
            ".dtpreldworld" => DirectiveName::DtpRelDWorld,
            ".sleb128" => DirectiveName::SLEB128,
            ".uleb128" => DirectiveName::ULEB128,
            ".p2align" => DirectiveName::P2Align,
            ".balign" => DirectiveName::BAlign,
            ".space" => DirectiveName::Space,
            ".zero" => DirectiveName::Zero,
            ".varaintcc" => DirectiveName::VariantCC,
            ".attribute" => DirectiveName::Attribute,
            ".end" => DirectiveName::End,
            ".extern" => DirectiveName::Extern,
            ".type" => DirectiveName::Type,
            _ => panic!("Unknown directive name: {}", name),
        }
    }

    pub fn is_string_based(&self) -> bool {
        match self {
            DirectiveName::File |
            DirectiveName::String |
            DirectiveName::ASCIIZ => true,
            _ => false,
        }
    }
}

impl GenerateCode for DirectiveName {
    fn generate_code_string(&self) -> String {
        let name = match self {
            DirectiveName::Align => ".align",
            DirectiveName::File => ".file",
            DirectiveName::Globl => ".globl",
            DirectiveName::Local => ".local",
            DirectiveName::Common => ".common",
            DirectiveName::Ident => ".ident",
            DirectiveName::Size => ".size",
            DirectiveName::String => ".string",
            DirectiveName::ASCIIZ => ".asciiz",
            DirectiveName::Equ => ".equ",
            DirectiveName::Option => ".option",
            DirectiveName::Byte => ".byte",
            DirectiveName::TwoByte => ".2byte",
            DirectiveName::FourByte => ".4byte",
            DirectiveName::EightByte => ".8byte",
            DirectiveName::Short => ".short",
            DirectiveName::Long => ".long",
            DirectiveName::Quad => ".quad",
            DirectiveName::Half => ".half",
            DirectiveName::Float => ".float",
            DirectiveName::Double => ".double",
            DirectiveName::DWord => ".dword",
            DirectiveName::Word => ".word",
            DirectiveName::DtpRelWord => ".dtprelword",
            DirectiveName::DtpRelDWorld => ".dtpreldword",
            DirectiveName::SLEB128 => ".sleb128",
            DirectiveName::ULEB128 => ".uleb128",
            DirectiveName::P2Align => ".p2align",
            DirectiveName::BAlign => ".balign",
            DirectiveName::Zero => ".zero",
            DirectiveName::Space => ".space",
            DirectiveName::VariantCC => ".variantcc",
            DirectiveName::Attribute => ".attribute",
            DirectiveName::End => ".end",
            DirectiveName::Extern => ".extern",
            DirectiveName::Type => ".type",
        };

        name.to_string()
    }
}

impl From<&str> for DirectiveName {
    fn from(value: &str) -> Self {
        DirectiveName::from_str(value)
    }
}

impl From<String> for DirectiveName {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}

#[derive(Clone, PartialEq)]
pub struct Directive {
    inc_name : DirectiveName,

    /// the parameters of the directive, for .align it's value can be changed
    parameters : Vec<String>,
}

impl Directive {
    pub fn new(name:&str, parameters:Vec<String>) -> Option<Self> {
        let inc_name = name.into();
        let r = Self { inc_name, parameters };
        Some(r)
    }

    pub fn compute_expr_as_u64(expr_str:&String) -> String {
        let y = expr_to_clrobj(expr_str, None).unwrap();
        let yy = y.conv_u64().unwrap().u64().unwrap();
        format!("{yy}")
    }

    pub fn compute_expr_as_i64(expr_str:&String) -> String {
        let y = expr_to_clrobj(expr_str, None).unwrap();
        let yy = y.conv_i64().unwrap().i64().unwrap();
        format!("{yy}")
    }

    pub fn compute_expr_as_float(expr_str:&String) -> String {
        let y = expr_to_clrobj(expr_str, None).unwrap();
        let yy = y.conv_f32().unwrap().f32().unwrap();
        format!("{yy}")
    }

    pub fn compute_expr_as_double(expr_str:&String) -> String {
        let y = expr_to_clrobj(expr_str, None).unwrap();
        let yy = y.conv_f64().unwrap().f64().unwrap();
        format!("{yy}")
    }

    /// get size directive like .size label, size (for example: .size   add2, .-add2)
    pub fn get_size_directive(&self) -> Option<(String, String)> {
        match self.inc_name {
            DirectiveName::Size => {
                let name = self.parameters.iter().nth(0).unwrap().to_string();
                let value = self.parameters.iter().nth(1).unwrap().to_string();
                Some((name, value))
            }
            _ => None,
        }
    }

    pub fn from_pair(pair:&Pair<Rule>, _config:&mut CodeGenConfiguration) -> Result<Self, AsmError> {
        let inner = pair.to_owned().into_inner();
        let inc_name_str = inner
                                .find_first_tagged("inc_name")
                                .ok_or(AsmError::NoFound((file!(), line!()).into(), format!("cannot find #inc_name = in {pair}, consider add #inc_name")))?
                                .as_str();
        
        let parameters = match inc_name_str.into() {
            DirectiveName::ASCIIZ |
            DirectiveName::String => {
                inner.skip(1)
                    .map(|x| { 
                            let s = FormattedString::parse(& x.as_str().to_string()).unwrap();
                            let s2 = s.process(&Vec::default()).unwrap();
                            s2
                        } )
                    .collect::<Vec<_>>()
            }
            DirectiveName::TwoByte | 
            DirectiveName::FourByte |
            DirectiveName::EightByte |
            DirectiveName::Half | 
            DirectiveName::Word |           
            DirectiveName::Byte => {
                inner.skip(1)
                .map(|x| {
                    let expr_str = x.as_str().to_string();
                    if x.as_rule() == Rule::expression {
                        Self::compute_expr_as_u64(&expr_str)
                    }
                    else {
                        expr_str
                    }
                })
                .collect::<Vec<_>>()
            }
            DirectiveName::Long |
            DirectiveName::Short => {
                inner.skip(1)
                    .map(|x| {
                        let expr_str = x.as_str().to_string();
                        if x.as_rule() == Rule::expression {
                            Self::compute_expr_as_i64(&expr_str)
                        }
                        else {
                            expr_str
                        }
                    })
                    .collect::<Vec<_>>()
            }
            DirectiveName::Float => 
                inner.skip(1)
                    .map(|x| {
                        let expr_str = x.as_str().to_string();
                        if x.as_rule() == Rule::expression {
                            Self::compute_expr_as_float(&expr_str)
                        }
                        else {
                            expr_str
                        }
                    })
                    .collect::<Vec<_>>(),
            DirectiveName::Double => 
                inner.skip(1)
                    .map(|x| {
                        let expr_str = x.as_str().to_string();
                        if x.as_rule() == Rule::expression {
                            Self::compute_expr_as_double(&expr_str)
                        }
                        else {
                            expr_str
                        }
                    })
                    .collect::<Vec<_>>(),
            DirectiveName::Align => {
                inner.skip(1)
                    .map(|x| {
                        let expr_str = x.as_str().to_string();
                        if x.as_rule() == Rule::expression {
                            Self::compute_expr_as_u64(&expr_str)
                        }
                        else {
                            expr_str
                        }
                    })
                    .collect::<Vec<_>>()
            }
            _ => inner.skip(1)
                    .map(|x| {
                        let expr_str = x.as_str().to_string();
                        if x.as_rule() == Rule::expression {
                            if let Ok(y) = expr_to_clrobj(&expr_str, None) {
                                if y.is_invalid() {
                                    return expr_str;
                                }
                            }

                            Self::compute_expr_as_i64(&expr_str)
                        }
                        else {
                            expr_str
                        }
                    })
                    .collect::<Vec<_>>(),
        };

        let r = Self::new(&inc_name_str, parameters)
            .ok_or(AsmError::NoFound((file!(), line!()).into(), format!("cannot create directive from {pair} with inc_name {inc_name_str}")))?;
        Ok(r)
    }

    pub fn get_directive_size(&self) -> Result<usize, AsmError> {
        match self.inc_name {
            DirectiveName::Align => if self.parameters.len() <2 { Ok(0) } 
                else { self.parameters[1].parse::<usize>()
                        .map_err(|_| AsmError::ConversionFailed((file!(), line!()).into(), format!("cannot convert {} to usize", self.parameters[1]))) },
            DirectiveName::File => Ok(0),
            DirectiveName::Globl => Ok(0),
            DirectiveName::Local => Ok(0),
            DirectiveName::Common => Ok(0),
            DirectiveName::Ident => Ok(0),
            DirectiveName::Size => Ok(0),
            DirectiveName::String => Ok(self.parameters.iter().fold(0, |acc, x| acc + x.len() + 1)),  //plus one because extra \0 is inserted at end
            DirectiveName::ASCIIZ => Ok(self.parameters.iter().fold(0, |acc, x| acc + x.len() + 1)),  //plus one because extra \0 is inserted at end
            DirectiveName::Equ => Ok(0),
            DirectiveName::Option => Ok(0),
            DirectiveName::Byte => Ok(self.parameters.len()),
            DirectiveName::TwoByte => Ok(self.parameters.len() * 2),
            DirectiveName::FourByte => Ok(self.parameters.len() * 4),
            DirectiveName::EightByte => Ok(self.parameters.len() * 8),
            DirectiveName::Short => Ok(self.parameters.len() * 2),
            DirectiveName::Long => Ok(self.parameters.len() * 4),
            DirectiveName::Quad => Ok(self.parameters.len() * 8),
            DirectiveName::Half => Ok(self.parameters.len() * 2),
            DirectiveName::Float => Ok(self.parameters.len() * 4),
            DirectiveName::Double => Ok(self.parameters.len() * 8),
            DirectiveName::DWord => Ok(self.parameters.len() * 8),
            DirectiveName::Word => Ok(self.parameters.len() * 4),
            DirectiveName::DtpRelWord => Ok(self.parameters.len() * 4),
            DirectiveName::DtpRelDWorld => Ok(self.parameters.len() * 8),
            DirectiveName::SLEB128 => Ok(self.parameters.len() * 16),
            DirectiveName::ULEB128 => Ok(self.parameters.len() * 16),
            DirectiveName::P2Align => Ok(0),
            DirectiveName::BAlign => Ok(0),
            DirectiveName::Space |
            DirectiveName::Zero => {
                let len_string = &self.parameters[0];
                let r = len_string.parse::<usize>()
                                .map_err(|_| AsmError::ConversionFailed((file!(), line!()).into(), format!("cannot convert {len_string} to usize")))?;
                Ok(r)
            }
            DirectiveName::VariantCC => Ok(0),
            DirectiveName::Attribute => Ok(0),
            DirectiveName::Extern => Ok(0),
            DirectiveName::End => Ok(0),
            DirectiveName::Type => Ok(0),
        }
    }

    pub fn get_machine_code(&self) -> Option<MachineCode> {
            match self.inc_name {
            DirectiveName::Long |
            DirectiveName::FourByte |
            DirectiveName::Word => {
                let data = self.parameters.iter()
                                                        .map(|x| { get_u64_from_str(x).unwrap() as u32} )
                                                        .flat_map(|x| x.to_le_bytes().to_vec())
                                                        .collect::<Vec<_>>();
                Some(MachineCode::new_bytes(data))
            }
            DirectiveName::EightByte |
            DirectiveName::DWord => {
                let data = self.parameters.iter()
                                                        .map(|x| { get_u64_from_str(x).unwrap() } )
                                                        .flat_map(|x| x.to_le_bytes().to_vec())
                                                        .collect::<Vec<_>>();
                Some(MachineCode::new_bytes(data))
            }
            DirectiveName::TwoByte |
            DirectiveName::Short => {
                let data = self.parameters.iter()
                                                        .map(|x: &String| { get_u64_from_str(x).unwrap() as u16} )
                                                        .flat_map(|x| x.to_le_bytes().to_vec())
                                                        .collect::<Vec<_>>();
                Some(MachineCode::new_bytes(data))
            }
            DirectiveName::Byte => {
                let data = self.parameters.iter()
                                                        .map(|x: &String| { get_u64_from_str(x).unwrap() as u8} )
                                                        .flat_map(|x| x.to_le_bytes().to_vec())
                                                        .collect::<Vec<_>>();
                Some(MachineCode::new_bytes(data))
            }
            DirectiveName::Float => {
                let data = self.parameters.iter()
                                                        .map(|x: &String| { x.parse::<f32>().unwrap() } )
                                                        .flat_map(|x| x.to_le_bytes().to_vec())
                                                        .collect::<Vec<_>>();
                Some(MachineCode::new_bytes(data))
            }
            DirectiveName::Double => {
                let data = self.parameters.iter()
                                                        .map(|x: &String| { x.parse::<f64>().unwrap() } )
                                                        .flat_map(|x| x.to_le_bytes().to_vec())
                                                        .collect::<Vec<_>>();
                Some(MachineCode::new_bytes(data))
            }
            DirectiveName::String => {
                let mut data = self.parameters.iter()
                                        .flat_map(|x| { x.as_bytes().to_vec() } )
                                        .collect::<Vec<_>>();
                data.push(0);
                Some(MachineCode::new_bytes(data))
            }
            DirectiveName::ASCIIZ => {
                let mut data = self.parameters.iter()
                                .flat_map(|x| { 
                                    let s = x.chars().map(|n| {n as u8}).collect::<Vec<_>>();
                                    s
                                })
                                .collect::<Vec<_>>();
                data.push(0);
                Some(MachineCode::new_bytes(data))
            }
            DirectiveName::Space | 
            DirectiveName::Zero => {
                let size = usize::from_str_radix(self.parameters[0].as_str(), 10).ok()?;
                let data = vec![0; size].to_vec();
                Some(MachineCode::new_bytes(data))
            }
            DirectiveName::End => None,
            DirectiveName::Align => {
                if self.parameters.len() < 2 {
                    return None;
                }
                
                let size = usize::from_str_radix(self.parameters[1].as_str(), 10).ok()?;
                if size == 0 {
                    return None;
                }
                
                let data = vec![0; size].to_vec();
                Some(MachineCode::new_bytes(data))
            }
            _ => None,
        }
    }

    pub fn get_equ(&self) -> Option<(&String, &String)> {
        match self.inc_name {
            DirectiveName::Equ => {
                let name = self.parameters.iter().nth(0).unwrap();
                let value = self.parameters.iter().nth(1).unwrap();
                Some((name, value))
            }
            _ => None,
        }
    }

    /// get directive name, like .equ, .file, .align, etc.
    pub fn get_name(&self) -> &DirectiveName {
        &self.inc_name
    }

    /// check if it is extern directive
    pub fn is_extern(&self) -> bool {
        self.inc_name == DirectiveName::Extern
    }

    /// get align value as u32 option
    pub fn get_align(&self) -> Option<u32> {
        match self.get_name() {
            DirectiveName::Align => get_u32_from_str(& self.parameters[0]).ok(),
            _ => None,
        }
    }

    pub fn get_global_label(&self) -> Option<&String> {
        if self.inc_name == DirectiveName::Globl {
            Some(self.parameters.iter().nth(0).unwrap())
        }
        else {
            None
        }
    }

    /// set align value
    pub fn set_align(&mut self, v:u32) {
        match self.get_name() {
            DirectiveName::Align => self.parameters = vec![v.to_string()],
            _ => ()
        }
    }

    /// get external label name
    pub fn get_extern_label(&self) -> Option<&String> {
        if self.is_extern() {
            Some(self.parameters.iter().nth(0).unwrap())
        }
        else {
            None
        }
    }

    /// change align's value to byte directive
    pub fn set_padding_data(&mut self, size:u32) {
        self.inc_name = DirectiveName::Align;
        self.parameters = vec!["0".to_string(), size.to_string()];
    }

    /// replace parameter with new value
    pub fn replace_parameter(&mut self, old_value:&str, new_value:&str) {
        match self.get_name() {
            DirectiveName::Size => {
                for i in 0..self.parameters.len() {
                    let param = self.parameters[i].trim().to_string();
                    self.parameters[i] = param.replace(old_value, new_value);
                }
            }
            _ => {
                for i in 0..self.parameters.len() {
                    if self.parameters[i] == old_value {
                        self.parameters[i] = new_value.to_string();
                    }
                }
            }
        }
    }
}

impl GenerateCode for Directive {
    fn generate_code_string(&self) -> String {
        let parameters = if self.inc_name.is_string_based() {
                    self.parameters.iter()
                                .map(|x| format!("\"{}\"", x))
                                .collect::<Vec<_>>()
                                .join(", ")
                } else {
                    self.parameters.join(", ")
                };

        let r = format!("{} {}", self.inc_name.generate_code_string(), parameters);
        r
    }
}

impl std::fmt::Debug for Directive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // format the parameters as a comma-separated string
        let parameters = if self.inc_name.is_string_based() {
                    self.parameters.iter().map(|x| format!("\"{}\"", generate_escape_string(x))).collect::<Vec<_>>().join(", ")
                } else {
                    self.parameters.join(", ")
                };
                
        write!(f, "{}({})", self.inc_name.generate_code_string(), parameters)
    }
}
