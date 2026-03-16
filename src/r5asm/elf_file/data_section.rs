use core::fmt;
use core::fmt::Debug;
use std::{fmt::Formatter, ops::Add};

use super::super::machinecode::MachineCode;

use super::*;

#[derive(PartialEq, Eq)]
pub struct DataSection {
    pub data: Vec<u8>,
}

impl DataSection {
    /// Create a new data section from raw bytes
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Convert data section to space-separated hex string
    pub fn to_hex(&self) -> String {
        bytes_to_hex(&self.data)
    }

    /// Create data section from hex string
    pub fn from_hex(hex: &str) -> Result<Self, &'static str> {
        hex_to_bytes(hex)
            .map(|bytes| Self::new(bytes))
    }

    /// Get raw bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get size of data section
    pub fn get_size(&self) -> usize {
        self.data.len()
    }
}

impl Default for DataSection {
    fn default() -> Self {
        Self { data: Vec::new() }
    }
}

pub type ReadOnlySection = DataSection;

impl From<Vec<u8>> for DataSection {
    fn from(data: Vec<u8>) -> Self {
        DataSection::new(data)
    }
}

impl From<Vec<MachineCode>> for DataSection {
    fn from(machine_codes: Vec<MachineCode>) -> Self {
        let bin = machine_codes.into_iter().flat_map(|x| { x.to_vec() }).collect::<Vec<_>>();
        bin.into()
    }
}

impl Add for DataSection {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut data = self.data;
        data.extend(other.data);
        DataSection::new(data)
    }
}

impl Debug for DataSection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "DataSection ({}) {{ \n[{}] \n[{}] }}", 
            self.get_size(), self.to_hex(), bytes_to_ascii(&self.data))
    }
}