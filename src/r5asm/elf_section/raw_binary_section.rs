use std::fmt::{self, Debug, Formatter};

use super::super::alignment::Alignment;

/// Raw binary section for ELF file metadata (non-loaded sections like string tables, symbol tables)
pub struct RawBinarySection {
    name_offset: usize,
    data: Vec<u8>,

    alignment: Alignment,
}

impl RawBinarySection {
    pub fn new(name_offset: usize, data: Vec<u8>) -> Self {
        Self {
            name_offset,
            data,
            alignment: Alignment::new(8).unwrap(),
        }
    }

    // Getter and setter for name_offset
    pub fn name_offset(&self) -> usize {
        self.name_offset
    }

    pub fn set_name_offset(&mut self, name_offset: usize) {
        self.name_offset = name_offset;
    }

    // Getter and setter for data
    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }

    pub fn alignment(&self) -> &Alignment {
        &self.alignment
    }

    pub fn get_section_data(&self) -> Vec<u8> {
        let mut bytes = self.alignment.get_padding_vec();
        bytes.extend(&self.data);
        bytes        
    }
}

impl Default for RawBinarySection {
    fn default() -> Self {
        Self {
            name_offset: 0,
            data: Vec::new(),
            alignment : Alignment::new(8).unwrap(),
        }
    }
}

impl Debug for RawBinarySection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("RawBinarySection")
            .field("name_offset", &self.name_offset)
            .field("data_len", &self.data.len())
            .finish()
    }
}
