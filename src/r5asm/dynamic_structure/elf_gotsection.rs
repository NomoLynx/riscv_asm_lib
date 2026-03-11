use std::fmt::{self, Debug, Formatter};

use rust_macro_internal::*;
use parser_lib::json::*;

use super::super::{alignment::*, traits::*};

pub const GOT_SECTION_NAME: &str = ".got";

json_struct2!("src/r5asm/dynamic_structure/elf_gotsection.json", 
    GOTSection,
    "src/r5asm/dynamic_structure/elf_gotsection.json.ini");

impl GOTSection {
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

    pub fn new2(plt0_address:u64) -> Self {
        let mut data = Vec::new(); // 2 reserved entries × 8 bytes
        
        // Entry 0: link_map (initialized to 0)
        data.push(0xFFFF_FFFF_FFFF_FFFF);
        
        // Entry 1: _dl_runtime_resolve (initialized to 0)
        data.push(0);
        
        Self { virtual_address:0, offset : 0, data, plt0_address, alignment: Alignment::new(8).unwrap() }
    }

    /// Adds a GOT slot for a symbol (e.g., `printf`).
    /// `resolver_addr`: Address of the PLT resolver code (e.g., 0x1004).
    /// Returns the virtual address of the new GOT slot.
    pub fn add_symbol(&mut self, plt0_addr: u64) -> u64 {
        let offset = 8 * self.data.len() as u64; // Offset from base_addr
        self.data.push(plt0_addr);
        self.plt0_address = plt0_addr;
        self.set_initial_resolver(plt0_addr);
        self.get_virtual_address() + offset
    }

    pub fn get_data(&self) -> Vec<u8> {
        let mut data = self.alignment.get_padding_vec();
        let buf = self.data.to_le_bytes();
        data.extend_from_slice(&buf);
        data
    }

    /// get size of got
    pub fn get_size(&self) -> usize {
        let r = self.data.len() * std::mem::size_of::<u64>();
        r
    }

    /// set plt0 address as initial value in got[2..]
    pub fn set_initial_resolver(&mut self, plt0_addr: u64) {
        self.plt0_address = plt0_addr;
        let start = 2;
        for i in start..self.data.len() {
            self.data[i] = plt0_addr;
        }
    }

    /// append the last element which has .dynamic virtual address
    pub fn append_last_element(&mut self, dyn_vaddr: u64) {
        self.data.push(dyn_vaddr);
    }

    pub fn get_got_unit_size(&self) -> usize {
        8 // Each GOT entry is 8 bytes (u64)
    }

    pub fn get_alignment(&self) -> &Alignment {
        &self.alignment
    }

    pub fn get_entry_number(&self) -> usize {
        self.data.len()
    }
}

impl Clone for GOTSection {
    fn clone(&self) -> Self {
        Self {
            virtual_address: self.virtual_address,
            offset: self.offset,
            data: self.data.clone(),
            plt0_address: self.plt0_address,
            alignment: self.alignment.clone(),
        }
    }
}

impl Default for GOTSection {
    fn default() -> Self {
        Self::new2(0)
    }
}

impl Debug for GOTSection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let data_string = self.data.iter()
            .map(|entry| format!("0x{:016X}", entry))
            .collect::<Vec<_>>()
            .join(", ");
        let name = self.get_section_name();
        let data_size = self.get_section_data_size();
        let memory_size = self.get_section_size();
        let offset = self.get_offset();

        write!(f, "{name} @ 0x{:X}, offset = 0x{offset:0X} and data size = 0x{data_size:X}/{data_size}, memory size = 0x{memory_size:X}/{memory_size}  {{ [ {} ], plt0 @ 0x{:X}, alignment: {:?} }}", 
            self.virtual_address, data_string, self.plt0_address, self.alignment)
    }
}