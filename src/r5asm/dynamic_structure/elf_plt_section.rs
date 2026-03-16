use std::fmt::{self, Debug, Formatter};

use rust_macro_internal::md2struct;
use super::super::{
    alignment::*,
    assembler::build_asm_snippet,
    traits::{
        section_name_trait::SectionNameTrait,
        section_size_trait::SectionSizeTrait,
        to_markdown_table_row_trait::ToMarkdownTableRow,
    },
};

#[md2struct("src/r5asm/dynamic_structure/elf_section.md", "ELF PLT Section")]
#[derive(Clone)]
pub struct ELFPLTSection {
}

impl ELFPLTSection {
    pub const PLT0_SIZE: u8 = 32; // Size of the initial PLT0 entry
    pub const PLT_ENTRY_SIZE: u8 = 16; // Size of each subsequent PLT entry

    /// Creates a new PLT section.
    /// - `plt_base_addr`: Virtual address of the PLT (e.g., 0x1000).
    /// - `got_plt_base`: Virtual address of .got.plt (e.g., 0x5000).
    fn new(plt_base_addr: u64, got_plt_base: u64) -> Self {        
        if plt_base_addr == 0 || got_plt_base == 0 {
            return Self::default();
        }

        let mut align = Alignment::new(8).unwrap();
        let v = align.calculate_padding_and_offset(plt_base_addr);
        Self {
            code: Vec::new(),
            got_plt_base,
            offset : 0,
            plt_base_addr : v,
            next_got_offset: 16, // Start after 2 reserved entries
            got_entries: Vec::new(),
            alignment: align,
        }
    }

    /// Adds a PLT entry for a symbol (e.g., `printf`)
    pub fn add_symbol(&mut self)  {
        self.got_entries.push(self.next_got_offset);
        self.next_got_offset += 8; // Each GOT entry is 8 bytes
    }

    /// Finalizes the PLT section (returns bytes and section header).
    pub fn generate_code(&self) -> Vec<u8> {
        let got_plt_base = self.get_got_plt_virtual_address();
        let got_slot_size = 8; // each got slot is 8 bytes

        // generate PLT0
        let parameters = [
            ("gotplt".to_string(), format!("{}", got_plt_base)),
            ("pc".to_string(), format!("{}", self.plt_base_addr)),
            ("printf".to_string(), format!("{}", got_plt_base + got_slot_size * 2))   //printf is just a placeholder for the external symbol reference, it can be any value
        ].to_vec().into();

        let mut code = build_asm_snippet(& include_str!("../../data/dynamic_plt0.s"), &parameters).unwrap();

        let plt_stub_size = Self::PLT_ENTRY_SIZE; // each plt stub is 16 bytes
        let plt0_size = Self::PLT0_SIZE; // plt0 is 32 bytes
        
        // generate PLT entries, no need to use got_entry, its value is plt0, which is plt_base_addr
        for (i, _v) in self.got_entries.iter().enumerate() {
            let params = 
                [("pc".to_string(), format!("{}", self.plt_base_addr + plt0_size as u64 + (i as u64 * plt_stub_size as u64))),
                (("printf".to_string(), format!("{}", got_plt_base + (i as u64 + 2) * got_slot_size)))].to_vec().into();
            let new_code = build_asm_snippet(& include_str!("../../data/dynamic_pltn.s"), &params).unwrap();
            code.extend(new_code);
        }

        code
    }

    /// get code binary + padding if needed
    pub fn get_code(&self) -> Vec<u8> {
        let mut bytes = self.alignment.get_padding_vec();
        bytes.extend_from_slice(self.code.as_slice());
        bytes
    }

    pub fn set_code(&mut self, code:Vec<u8>) {
        self.code = code;
    }

    /// Returns the size of the PLT section in bytes
    pub fn get_size(&self) -> usize {
        let plt0_size = 32; // Size of the initial PLT0 entry
        plt0_size + (self.got_entries.len() * 16) // Each PLT entry is 16 bytes
    }

    pub fn get_got_plt_virtual_address(&self) -> u64 {
        self.got_plt_base
    }

    pub fn set_got_plt_virtual_address(&mut self, vadd:u64) {
        self.got_plt_base = vadd;
    }

    pub fn get_virtual_address(&self) -> u64 {
        self.get_plt0_address()
    }

    pub fn set_virtual_address(&mut self, vadd:u64) {
        self.plt_base_addr = vadd;
    }

    pub fn get_offset(&self) -> u64 {
        self.offset
    }

    pub fn set_offset(&mut self, offset:u64) {
        let v = self.alignment.calculate_padding_and_offset(offset);
        self.offset = v;
    }

    pub fn get_plt0_address(&self) -> u64 {
        self.plt_base_addr
    }

    pub fn get_alignment(&self) -> &Alignment {
        &self.alignment
    }

    pub fn get_entry_number(&self) -> usize {
        if self.get_size() < Self::PLT0_SIZE as usize {
            return 0;
        }

        self.get_size() / Self::PLT_ENTRY_SIZE as usize - 1  // -1 for the initial PLT0 entry
    }

    pub fn get_virtual_address_of_entry(&self, index: usize) -> Option<u64> {
        if self.get_entry_number() == 0 {
            return None;
        }

        if index >= self.get_entry_number() {
            return None;
        }

        let plt0_size = Self::PLT0_SIZE as u64;
        let entry_size = Self::PLT_ENTRY_SIZE as u64;
        Some(self.get_virtual_address() + plt0_size + (index as u64 * entry_size))
    }
}

impl Default for ELFPLTSection {
    fn default() -> Self {
        Self { code: Vec::new(), got_plt_base : 0, plt_base_addr : 0, next_got_offset: 0, got_entries: Vec::new(),
            offset : 0, 
            alignment: Alignment::new(16).unwrap() }
    }
}

impl Debug for ELFPLTSection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let code_len = self.code.len();
        if code_len == 0 {
            write!(f, "PLT is empty")
        }
        else {
            let inc_len: usize = code_len / 4;
            let section_len = inc_len / 4 - 1;
            let name = self.get_section_name();
            let data_size = self.get_section_data_size();
            let memory_size = self.get_section_size();
            let offset = self.get_offset();

            write!(f, "{name} @ 0x{:X}, offset = 0x{offset:X}, data size = 0x{data_size:X}/{data_size}, memory size = 0x{memory_size:X}/{memory_size} {{ code ({} bytes/{} inc's/{} sections): {:?}, got_plt_base: 0x{:X}, next_got_offset: 0x{:X}, got_entries: {:?}, alignment: {:?} }}", 
                self.plt_base_addr, 
                code_len, inc_len, section_len, self.code, 
                self.got_plt_base, self.next_got_offset, self.got_entries, self.alignment)
        }
    }
}

impl ToMarkdownTableRow for ELFPLTSection {
    fn get_markdown_header(&self) -> String {
        "| code (hex) | got_plt_base | plt_base_addr | next_got_offset |
|-----------|--------------|--------------|----------------|".to_string()
    }
    
    fn to_markdown(&self) -> String {
        let code_hex: String = self.code
            .chunks(16)
            .map(|chunk| chunk.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "))
            .collect::<Vec<_>>()
            .join("<br>");
        
        format!(
            "| `{}` | {} (0x{:X}) | {} (0x{:X}) | {} (0x{:X}) |",
            code_hex,
            self.got_plt_base, self.got_plt_base,
            self.plt_base_addr, self.plt_base_addr,
            self.next_got_offset, self.next_got_offset
        )
    }
}
