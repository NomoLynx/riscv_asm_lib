use std::fmt::{self, Debug, Formatter};

use rust_macro_internal::md2struct;

use super::super::{alignment::*, traits::{section_name_trait::SectionNameTrait, section_size_trait::SectionSizeTrait}};

pub type RelaRInfo = u64;

/// Relocation type for RISC-V jump slots
/// This is used in the PLT (Procedure Linkage Table) for dynamic linking.
/// It indicates that the relocation is for a jump slot, which is a placeholder
/// for a function pointer that will be resolved at runtime.
/// The value 5 corresponds to the R_RISCV_JUMP_SLOT relocation type.
pub const R_RISCV_JUMP_SLOT : u32 = 5;

/// plt.rela section
#[md2struct("src/r5asm/dynamic_structure/elf_section.md", "ELF PLT Relocation Table")]
#[derive(Clone)]
pub struct ELFPLTRelocationTable {
}

impl ELFPLTRelocationTable {
    pub fn add_jump_slot_entry(&mut self, r_offset: u64, sym_index: u32) {
        self.add_entry2(r_offset, sym_index, R_RISCV_JUMP_SLOT);
    }

    pub fn add_entry2(&mut self, r_offset: u64, sym_index: u32, rtype: u32) {
        let entry = PLTRelocationEntry::new2(r_offset, sym_index, rtype);
        self.insert_entry(entry);
    }

    pub fn insert_entry(&mut self, entry:PLTRelocationEntry) {
        self.entries.push(entry);
    }

    pub fn get_size(&self) -> usize {
        self.entries.len() * std::mem::size_of::<PLTRelocationEntry>()
    }

    pub fn get_data(&self) -> Vec<u8> {
        let mut bytes = self.alignment.get_padding_vec();
        for entry in &self.entries {
            bytes.extend_from_slice(&entry.get_code());
        }
        bytes
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

    pub fn set_link(&mut self, link: u32) {
        self.link = link;
    }

    pub fn get_link(&self) -> u32 {
        self.link
    }

    pub fn set_info(&mut self, info: u32) {
        self.info = info;
    }

    pub fn get_info(&self) -> u32 {
        self.info
    }

    pub fn update_jump_slot_offset(&mut self, got_vadd:u64) {
        for entry in &mut self.entries {
            if entry.is_jump_slot() {
                let offset = entry.get_offset();
                entry.set_offset(got_vadd + offset);
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &PLTRelocationEntry> {
        self.entries.iter()
    }
}

impl Default for ELFPLTRelocationTable {
    fn default() -> Self {
        Self {
            virtual_address: 0,
            offset : 0,
            entries: vec![],
            alignment: Alignment::new(8).unwrap(),
            link: 0,
            info: 0,
        }
    }
}

impl Debug for ELFPLTRelocationTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = self.get_section_name();
        let data_size = self.get_section_data_size();
        let memory_size = self.get_section_data_size();
        let offset = self.get_offset();

        write!(
            f,
            "{name} @ 0x{:X}, offset = 0x{offset:X}, data size = 0x{data_size:X}/{data_size}, memory size = 0x{memory_size:X}/{memory_size} {{ link: {}, info: {}, {} entries: {:?}, alignment: {:?} }}",
            self.virtual_address,
            self.link,
            self.info,
            self.entries.len(),
            self.entries,
            self.alignment
        )
    }
}

#[md2struct("src/r5asm/dynamic_structure/elf_section.md", "PLT Relocation Entry")]
#[derive(Clone)]
pub struct PLTRelocationEntry {
}

impl PLTRelocationEntry {
    pub fn new2(r_offset:u64, sym_index: u32, rtype: u32) -> Self {
        let r_info = Self::build_elf64_r_info(sym_index, rtype);
        Self::new(r_offset, r_info)
    }

    pub fn new(r_offset:u64, r_info:RelaRInfo) -> Self {
        Self { r_offset, r_info, r_addend: 0 }
    }

    /// Construct r_info from symbol index and relocation type
    fn build_elf64_r_info(sym_index: u32, rtype: u32) -> RelaRInfo {
        let a = (sym_index as u64) << 32;
        let b = (rtype as u64) & 0xffff_ffff;
        let r = a | b;
        r 
    }

    /// Extract symbol index from r_info
    pub fn get_elf64_r_sym_index(&self) -> u32 {
        (self.r_info >> 32) as u32
    }

    /// Extract relocation type from r_info
    pub fn get_elf64_r_type(&self) -> u32 {
        (self.r_info & 0xffff_ffff) as u32
    }

    pub fn is_jump_slot(&self) -> bool {
        self.get_elf64_r_type() == R_RISCV_JUMP_SLOT
    }

    pub fn get_code(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.r_offset.to_le_bytes());
        bytes.extend_from_slice(&self.r_info.to_le_bytes());
        bytes.extend_from_slice(&self.r_addend.to_le_bytes());
        bytes
    }

    /// Check if the offset is zero, which is often invalid for relocation entries, means it's not set properly.
    pub fn is_offset_zero(&self) -> bool {
        self.r_offset == 0
    }

    pub fn set_offset(&mut self, v:u64) {
        self.r_offset = v;
    }

    pub fn get_offset(&self) -> u64 {
        self.r_offset
    }

    pub fn get_size(&self) -> usize {
        std::mem::size_of::<u64>() * 3
    }
}

impl Default for PLTRelocationEntry {
    fn default() -> Self {
        Self { r_offset: 0, r_info: 0, r_addend: 0 }
    }
}

impl Debug for PLTRelocationEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Rela.PLT Entry {{ patch GOT.PLT @ 0x{:X}, r_info: 0x{:X} (sym_index: {}, type: {}), r_addend: {} }}", 
            self.r_offset, self.r_info, self.get_elf64_r_sym_index(), self.get_elf64_r_type(), self.r_addend)
    }
}

impl IntoIterator for ELFPLTRelocationTable {
    type Item = PLTRelocationEntry;
    type IntoIter = std::vec::IntoIter<PLTRelocationEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl<'a> IntoIterator for &'a ELFPLTRelocationTable {
    type Item = &'a PLTRelocationEntry;
    type IntoIter = std::slice::Iter<'a, PLTRelocationEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter()
    }
}