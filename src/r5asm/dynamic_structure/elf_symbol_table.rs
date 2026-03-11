use core::panic;
use std::fmt::{self, Debug, Formatter};
use std::ops::{Index, IndexMut};

use crate::r5asm::dynamic_structure::dynamic_symbol_entry::DynamicSymbolEntry;

use super::super::traits::section_name_trait::SectionNameTrait;
use super::super::traits::section_size_trait::SectionSizeTrait;
use super::super::traits::to_markdown_table_row_trait::ToMarkdownTableRow;
use super::ELFStringTable;
use super::super::alignment::*;

#[derive(Clone)]
pub struct ELFDynamicSymbolTable {
    virtual_address: u64,
    offset : u64,
    entries : Vec<DynamicSymbolEntry>,
    alignment: Alignment,
}

impl ELFDynamicSymbolTable {
    pub fn insert_symbol(&mut self, symbol:DynamicSymbolEntry) -> usize{
        self.entries.push(symbol);
        self.entries.len() - 1
    }

    /// Adds a new dynamic symbol entry for an external global function.
    /// `st_name`: The string table index for the symbol name.
    /// Returns the virtual address of the new symbol entry.
    /// The virtual address is calculated as the base address plus the offset of the new entry.
    pub fn add_extern_global_function(&mut self, st_name:u32) -> u64 {
        let entry = DynamicSymbolEntry::new_extern_global_function_entry(st_name);
        self.insert_symbol(entry);
        self.virtual_address + (self.entries.len() as u64 - 1) * std::mem::size_of::<DynamicSymbolEntry>() as u64
    }

    pub fn get_entries(&self) -> &Vec<DynamicSymbolEntry> {
        &self.entries
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

    pub fn get_bytes(&self) -> Vec<u8> {
        let mut bytes = self.alignment.get_padding_vec();
        for entry in &self.entries {
            bytes.extend_from_slice(&entry.to_bytes());
        }
        bytes
    }

    /// get the index of the global symbol entry in the symbol table
    pub fn get_global_symbol_index(&self) -> Option<usize> {
        let valid = self.validate_symbols();
        if !valid {
            panic!("Invalid symbol table: local symbols must precede global symbols, and at least one global symbol is required.");
        }

        for (index, entry) in self.get_entries().iter().enumerate() {
            if entry.is_global() {
                return Some(index);
            }
        }

        None
    }

    /// validate the symbol, (1) all local symbol should be before global symbol
    /// (2) there should be at least one global symbol (the global pointer)
    fn validate_symbols(&self) -> bool {
        let mut found_global = false;
        for entry in &self.entries {
            if entry.is_global() {
                found_global = true;
            } else if found_global {
                // Found a local symbol after a global symbol, which is invalid
                return false;
            }
        }

        found_global // Ensure at least one global symbol exists
    }

    pub fn find_symbol_index_by_name(&self, symbol_name:&str, string_table:&ELFStringTable) -> Option<usize> {
        for (index, entry) in self.entries.iter().enumerate() {
            // Assuming you have a way to get the actual symbol name from st_name
            // This might involve looking up a string table using the st_name index
            if let Some(actual_symbol_name) = string_table.get_string_from_offset(entry.get_st_name() as usize) {
                if actual_symbol_name == symbol_name {
                    return Some(index);
                }
            }
        }

        None
    }

    pub fn get_symbol_entry_by_name(&self, symbol_name:&str, string_table:&ELFStringTable) -> Option<&DynamicSymbolEntry> {
        if let Some(index) = self.find_symbol_index_by_name(symbol_name, string_table) {
            return self.entries.get(index);
        }
        None
    }

    pub fn find_symbol_by_name_mut(&mut self, symbol_name:&str, string_table:&ELFStringTable) -> Option<&mut DynamicSymbolEntry> {
        if let Some(index) = self.find_symbol_index_by_name(symbol_name, string_table) {
            return self.entries.get_mut(index);
        }
        None
    }

    pub fn get_alignment(&self) -> &Alignment {
        &self.alignment
    }

}

impl Default for ELFDynamicSymbolTable {
    fn default() -> Self {
        let entries = vec![DynamicSymbolEntry::default()];
        Self { virtual_address: 0, offset: 0, entries, alignment: Alignment::new(8).unwrap() }
    }
}

impl Debug for ELFDynamicSymbolTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = self.get_section_name();
        let data_size = self.get_section_data_size();
        let memory_size = self.get_section_size();
        
        // all entry string on new line
        let entries_str = self.entries.iter()
            .map(|e| format!("{:?}", e))
            .collect::<Vec<String>>()
            .join(",\n  ");

        let offset = self.get_offset();

        write!(f, "{name} @ 0x{:X}, offset = 0x{offset:X}, data size = 0x{data_size:X}/{data_size}, memory size = 0x{memory_size:X}/{memory_size} {{ {} entries: {entries_str}, alignment: {:?} }}", 
            self.virtual_address, self.entries.len(), self.alignment)
    }
}

impl ToMarkdownTableRow for ELFDynamicSymbolTable {
    fn get_markdown_header(&self) -> String {
        if self.entries.is_empty() {
            return "No entries available".to_string();
        }
        self.entries[0].get_markdown_header()
    }
    
    fn to_markdown(&self) -> String {
        if self.entries.is_empty() {
            return "No entries available".to_string();
        }
        self.entries.iter().map(|entry| entry.to_markdown()).collect::<Vec<_>>().join("\n")
    }
}

impl Index<usize> for ELFDynamicSymbolTable {
    type Output = DynamicSymbolEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for ELFDynamicSymbolTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

pub enum DynamicSymbolInfo {
    Local,
    LocalNoType,
    LocalSection,
    GlobalFunction,
    GlobalObject,
    GlobalNoType,
    WeakFunction,
    WeakObject,
    Unknown(u8, u8), // (bind, type)
}

impl DynamicSymbolInfo {
    pub fn is_local(&self) -> bool {
        matches!(self, DynamicSymbolInfo::Local | DynamicSymbolInfo::LocalNoType | DynamicSymbolInfo::LocalSection)
    }

    pub fn is_global(&self) -> bool {
        matches!(self, DynamicSymbolInfo::GlobalFunction | DynamicSymbolInfo::GlobalObject | DynamicSymbolInfo::GlobalNoType)
    }

    pub fn is_weak(&self) -> bool {
        matches!(self, DynamicSymbolInfo::WeakFunction | DynamicSymbolInfo::WeakObject)
    }
}

impl Debug for DynamicSymbolInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DynamicSymbolInfo::Local => write!(f, "Local"),
            DynamicSymbolInfo::LocalNoType => write!(f, "Local No Type"),
            DynamicSymbolInfo::LocalSection => write!(f, "Local Section"),
            DynamicSymbolInfo::GlobalFunction => write!(f, "Global Function"),
            DynamicSymbolInfo::GlobalObject => write!(f, "Global Object"),
            DynamicSymbolInfo::GlobalNoType => write!(f, "Global No Type"),
            DynamicSymbolInfo::WeakFunction => write!(f, "Weak Function"),
            DynamicSymbolInfo::WeakObject => write!(f, "Weak Object"),
            DynamicSymbolInfo::Unknown(bind, typ) => write!(f, "Unknown (bind: {}, type: {})", bind, typ),
        }
    }
}

impl From<(u8, u8)> for DynamicSymbolInfo {
    fn from(value: (u8, u8)) -> Self {
        match value {
            (0, 0) => DynamicSymbolInfo::Local,
            (0, 1) => DynamicSymbolInfo::LocalNoType,
            (0, 3) => DynamicSymbolInfo::LocalSection,
            (1, 2) => DynamicSymbolInfo::GlobalFunction,
            (1, 1) => DynamicSymbolInfo::GlobalObject,
            (1, 0) => DynamicSymbolInfo::GlobalNoType,
            (2, 2) => DynamicSymbolInfo::WeakFunction,
            (2, 1) => DynamicSymbolInfo::WeakObject,
            other => DynamicSymbolInfo::Unknown(other.0, other.1),
        }
    }
}

impl From<DynamicSymbolInfo> for u8 {
    fn from(info: DynamicSymbolInfo) -> Self {
        match info {
            DynamicSymbolInfo::Local => 0x00,
            DynamicSymbolInfo::LocalNoType => 0x01,
            DynamicSymbolInfo::LocalSection => 0x03,
            DynamicSymbolInfo::GlobalFunction => 0x12, // STB_GLOBAL (1) << 4 | STT_FUNC (2)
            DynamicSymbolInfo::GlobalObject => 0x11,   // STB_GLOBAL (1) << 4 | STT_OBJECT (1)
            DynamicSymbolInfo::GlobalNoType => 0x10,   // STB_GLOBAL (1) << 4 | STT_NOTYPE (0)
            DynamicSymbolInfo::WeakFunction => 0x22,   // STB_WEAK (2) << 4 | STT_FUNC (2)
            DynamicSymbolInfo::WeakObject => 0x21,     // STB_WEAK (2) << 4 | STT_OBJECT (1)
            DynamicSymbolInfo::Unknown(bind, typ) => (bind << 4) | (typ & 0x0F),
        }
    }
}