use std::ops::Add;

use super::super::machinecode::MachineCode;

use super::{bytes_to_hex, hex_to_bytes};

#[derive(Debug, PartialEq, Eq)]
pub struct CodeSection {
    pub code: Vec<u8>,
}

impl CodeSection {
    /// Create a new code section from raw bytes
    pub fn new(code: Vec<u8>) -> Self {
        CodeSection { code }
    }

    /// Convert code section to space-separated hex string
    pub fn to_hex(&self) -> String {
        bytes_to_hex(&self.code)
    }

    /// Create code section from hex string
    pub fn from_hex(hex: &str) -> Result<Self, &'static str> {
        hex_to_bytes(hex)
            .map(|bytes| CodeSection::new(bytes))
    }

    /// Get raw bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.code
    }

    pub fn get_size(&self) -> usize {
        self.code.len()
    }
}

impl From<Vec<u8>> for CodeSection {
    fn from(code: Vec<u8>) -> Self {
        CodeSection::new(code)
    }
}

impl From<Vec<MachineCode>> for CodeSection {
    fn from(machine_codes: Vec<MachineCode>) -> Self {
        let bin = machine_codes.into_iter().flat_map(|x| { x.to_vec() }).collect::<Vec<_>>();
        bin.into()
    }
}

impl Add for CodeSection {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut code = self.code;
        code.extend(other.code);
        CodeSection::new(code)
    }
}