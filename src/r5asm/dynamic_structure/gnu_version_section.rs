use super::super::{alignment::Alignment, traits::SectionNameTrait};
use std::{fmt::Debug, vec};

pub type GnuVersionEntry = u16;

#[derive(Clone)]
pub struct GnuVersionSection {
    virtual_address : u64,
    offset : u64, 

    /// Version information entries
    entries: Vec<GnuVersionEntry>,
    alignment: Alignment,
}

impl GnuVersionSection {

    /// Add a new version entry
    pub fn add_entry(&mut self, entry: GnuVersionEntry) {
        self.entries.push(entry);
    }

    /// Get all version entries
    pub fn get_entries(&self) -> &Vec<GnuVersionEntry> {
        &self.entries  
    }

    /// serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.alignment.get_padding_vec();
        for entry in &self.entries {
            bytes.extend(&entry.to_le_bytes());
        }
        bytes
    }

    /// get size in bytes
    pub fn get_size_in_bytes(&self) -> usize {
        self.entries.len() * std::mem::size_of::<GnuVersionEntry>()
    }

    pub fn get_offset(&self) -> u64 {
        self.offset
    }

    pub fn set_offset(&mut self, offset:u64) {
        let v = self.alignment.calculate_padding_and_offset(offset);
        self.offset = v; 
    }

    /// get virutal address
    pub fn get_virtual_address(&self) -> u64 {
        self.virtual_address
    } 

    /// set virtual address
    pub fn set_virtual_address(&mut self, offset:u64) {
        self.virtual_address = offset;
    }

    /// get alignment
    pub fn get_alignment(&self) -> &Alignment {
        &self.alignment
    }
}

impl Default for GnuVersionSection {
    fn default() -> Self {
        GnuVersionSection {
            virtual_address: 0,
            offset : 0,
            entries: vec![0], // First entry is always 0
            alignment: Alignment::new(2).unwrap(),
        }
    }
}

impl Debug for GnuVersionSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.get_section_name();
        let data_size = self.get_size_in_bytes();
        let memory_size = data_size + self.alignment.get_padding() as usize;
        let va = self.get_virtual_address();
        let offset = self.get_offset();

        write!(f, r###"{name} 0x{va:X}/{va}, offset = 0x{offset:X} {{ 
                    {:?}, data size = 0x{data_size:X}/{data_size}, memory size = 0x{memory_size:X}/{memory_size},
                    {} entries: {:?} }}"###, 
            self.alignment,
            self.get_entries().len(), self.get_entries())
    }
}