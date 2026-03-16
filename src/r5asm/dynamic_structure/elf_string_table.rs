use super::super::{alignment::*, traits::{section_name_trait::SectionNameTrait, section_size_trait::SectionSizeTrait}};

use super::elf_dynamic_structure::DynamicEntry;
use std::fmt::{self, Debug, Formatter};
use std::ops::{Index, IndexMut};

#[derive(Clone)]
pub struct ELFStringTable {
    virtual_address : u64,
    offset : u64,
    data : Vec<String>,
    alignment: Alignment,
}

impl ELFStringTable {
    pub fn new() -> Self {
        Self { virtual_address: 0, offset : 0, data: vec![], alignment: Alignment::default() }
    }

    pub fn add_string(&mut self, s:&str) {
        self.data.push(s.to_string());
    }

    pub fn get_string_index(&self, s:&str) -> Option<usize> {
        self.data.iter().position(|x| x == s)
    }

    pub fn contains(&self, s:&str) -> bool {
        self.get_string_index(s).is_some()
    }

    /// get string offset, if not exist, add it and return the offset
    pub fn get_string_offset_or_add(&mut self, s:&str) -> usize {
        match self.get_string_index(s) {
            Some(index) => { 
                // get number of chars for the previous strings
                let mut offset = 0;
                for i in 0..index {
                    offset += self.data[i].len() + 1;  //add 1 is needed as the ending /0 is not counted so here to add 1
                }
                offset
            }
            None => {
                self.add_string(s);
                let mut offset = 0;
                for i in 0 ..( self.data.len()-1 ) {
                    offset += self.data[i].len() + 1;  //add 1 is needed as the ending /0 is not counted so here to add 1
                }
                offset
            }
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut final_bytes = self.alignment.get_padding_vec();
        for s in &self.data {
            final_bytes.extend_from_slice(s.as_bytes());
            final_bytes.push(0);
        }
        final_bytes
    }

    pub fn get_size(&self) -> usize {
        self.to_bytes().len()
    }

    /// get string table size entry for dynamic section
    pub fn to_dynamic_entry_string_table_size(&self) -> DynamicEntry {
        DynamicEntry::new_strtab_size(self.get_size() as u64)
    }

    /// get string from file offset
    pub fn get_string_from_file_offset(&self, file_offset: u64) -> Option<String> {
        let offset = file_offset - self.get_offset();
        if offset as usize >= self.get_size() {
            return None;
        }
        let bytes = &self.to_bytes()[offset as usize..];
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        let r = String::from_utf8_lossy(&bytes[..end]).to_string();
        Some(r)
    }

    /// get string from virtual address
    pub fn get_string_from_virtual_address(&self, vadd: u64) -> Option<String> {
        let offset = vadd - self.get_virtual_address();
        if offset as usize >= self.get_size() {
            return None;
        }

        let bytes = &self.to_bytes()[offset as usize..];
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        let r = String::from_utf8_lossy(&bytes[..end]).to_string();
        Some(r)
    }

    /// get string from offset
    pub fn get_string_from_offset(&self, offset: usize) -> Option<String> {
        if offset >= self.get_size() {
            return None;
        }

        let bytes = &self.to_bytes()[offset..];
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        let r = String::from_utf8_lossy(&bytes[..end]).to_string();
        Some(r)
    }

    /// get string's offset or None if not found
    pub fn get_string_offset(&self, s:&str) -> Option<usize> {
        match self.get_string_index(s) {
            Some(index) => { 
                // get number of chars for the previous strings
                let mut offset = 0;
                for i in 0..index {
                    offset += self.data[i].len() + 1;  //add 1 is needed as the ending /0 is not counted so here to add 1
                }
                Some(offset)
            }
            None => {
                None
            }
        }
    }

    pub fn get_virtual_address(&self) -> u64 {
        self.virtual_address
    }

    pub fn set_virtual_address(&mut self, vadd:u64) {
        
        self.virtual_address = vadd;
    }

    pub fn get_offset(&self) -> u64 {
        self.offset
    }

    pub fn set_offset(&mut self, offset:u64) {
        let v = self.alignment.calculate_padding_and_offset(offset);
        self.offset = v;
    }

    pub fn get_alignment(&self) -> &Alignment {
        &self.alignment
    }
}

impl Index <usize> for ELFStringTable {
    type Output = String;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl IndexMut<usize> for ELFStringTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl Default for ELFStringTable {
    fn default() -> Self {
        let mut r = Self::new();
        r.add_string("");
        r
    }
}

impl Debug for ELFStringTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // get a string joined by ", ", and each string element will has its index and offset from beginning
        let mut data = String::new();
        let mut offset = 0;
        for (i, s) in self.data.iter().enumerate() {
            let len = s.len();
            data.push_str(&format!("{i}@0x{offset:X} ({len}): \"{s}\""));
            if i < self.data.len() - 1 {
                data.push_str(", ");
            }

            offset += s.len() + 1; // +1 for the null terminator
        }

        if data.is_empty() {
            data = "[]".to_string();
        } 
        else {
            data = format!("[{data}]");
        }

        let total_len = self.get_size();
        let char_len = total_len - self.data.len(); // remove the ending /0 count
        let name = self.get_section_name();
        let data_size = self.get_section_data_size();
        let memory_size = self.get_section_size();
        let offset = self.get_offset();

        write!(f, "{name} ({total_len}/{char_len}), data size = 0x{data_size:X}/{data_size}, memory size = 0x{memory_size:X}/{memory_size} @0x{:X}, offset = 0x{offset:X}, {{ {} items: {}, alignment: {:?} }}", 
                self.virtual_address, self.data.len(), data, self.alignment)
    }
}