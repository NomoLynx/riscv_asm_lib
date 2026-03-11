use core_utils::debug::*;
use super::asm_error::AsmError;

#[derive(Clone)]
pub struct MachineCode {
    code : MachineCodeData,
    offset : usize,
}

impl MachineCode {
    pub (crate) fn new1(code:u32) -> Self {
        Self::new(code, 0)
    }

    pub (crate) fn new(code:u32, offset:usize) -> Self {
        Self { code:MachineCodeData::new(code), offset }
    }

    pub (crate) fn new_align(v:u32) -> Self {
        Self { code:MachineCodeData::Align(v), offset: 0 }
    }

    pub (crate) fn from_string(machine_code_str:&str, offset:usize) -> Result<Self, AsmError> {
        if machine_code_str.chars().all(|x| x == '0' || x=='1') {
            let machine_code = u32::from_str_radix(machine_code_str, 2)
                                    .map_err(|_| AsmError::ConversionFailed((file!(), line!()).into(), format!("cannot convert {machine_code_str} to machine code.")))?;
            Ok(Self::new(machine_code, offset))
        }
        else {
            error_string(format!("parameter error when converting {machine_code_str} to machine code"));
            Err(AsmError::ParameterError((file!(), line!()).into()))
        }
    }

    pub (crate) fn new_bytes(data:Vec<u8>) -> Self {
        Self { code: MachineCodeData::Bytes(data), offset: 0 }
    }

    pub (crate) fn to_vec(&self) -> Vec<u8> {
        self.code.to_vec()
    }

    pub fn get_code_data(&self) -> &MachineCodeData {
        &self.code
    }

    pub fn get_size(&self) -> usize {
        self.code.get_size()
    }
}

impl std::fmt::Debug for MachineCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = format!("MachineCode = {:?} @ {offset}, 0x{offset:x}", self.code, offset = self.offset);
        write!(f, "{}", s)   
    }
}

#[derive(Clone)]
pub enum MachineCodeData {
    U32(u32),
    Bytes(Vec<u8>),
    Align(u32),
}

impl MachineCodeData {
    pub fn new(data:u32) -> Self {
        Self::U32(data)
    }

    pub fn to_vec(&self) -> Vec<u8> {
        match self {
            Self::U32(n) => n.to_le_bytes().to_vec(),
            Self::Bytes(n) => n.clone(),
            Self::Align(_n) => vec![], // Align is not a machine code
        }
    }

    pub fn get_size(&self) -> usize {
        match self {
            Self::U32(_) => 4,
            Self::Bytes(n) => n.len(),
            Self::Align(_) => 0, // Align is not a machine code
        }
    }
}

impl std::fmt::Debug for MachineCodeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::U32(n) => write!(f, "0x{:0>8X}", n),
            Self::Bytes(n) => write!(f, "[{}]", n.iter().map(|x| format!("0x{x:0>2X}")).collect::<Vec<_>>().join(", ")),
            Self::Align(n) => write!(f, "Align({})", n),
        }
    }
}