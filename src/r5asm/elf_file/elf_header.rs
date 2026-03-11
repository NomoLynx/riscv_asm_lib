use std::ops::ShlAssign;

use rust_macro_internal::*;
use rust_macro::*;
use core_utils::number::u8_array_to_u32_little_endian;

use super::{bytes_to_hex, hex_to_bytes};
use crate::r5asm::elf_section::ElfSectionStructure;

#[packet_struct("src/r5asm/elf_file/elf_header.mermaid")]
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[derive(Accessors)]
pub struct ELFHeader {

}

pub type Elf64Header = ELFHeader;  // Alias for clarity

// Constants for ELF header
const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];
const ELF_CLASS_64: u8 = 2;    // ELFCLASS64
const ELF_DATA_LSB: u8 = 1;    // ELFDATA2LSB
const ELF_VERSION_CURRENT: u8 = 1;
const ELF_OSABI_SYSV: u8 = 0;  // ELFOSABI_NONE
const ELF_ABI_VERSION: u8 = 0;

// RISC-V specific constants
const ET_EXEC: u16 = 2;        // Executable file type
const ET_DYN: u16 = 3;         // Shared object file type
const EM_RISCV: u16 = 243;     // RISC-V architecture
const RISCV_EFLAGS: u32 = 4;   // Default flags for RV64GC

impl Default for ELFHeader {
    fn default() -> Self {
        ELFHeader {
            EI_MAG: u8_array_to_u32_little_endian(&ELF_MAGIC).unwrap(),
            EI_CLASS: ELF_CLASS_64,
            EI_DATA: ELF_DATA_LSB,
            EI_VERSION : ELF_VERSION_CURRENT,
            EI_OSABI : ELF_OSABI_SYSV,
            EI_ABIVERSION : ELF_ABI_VERSION,
            EI_PAD : 0,
            e_type: ET_EXEC,
            e_machine: EM_RISCV,
            e_version: ELF_VERSION_CURRENT as u32,
            e_entry: 0x10000,  // Typical default entry point address
            e_phoff: Self::SERIALIZED_SIZE as u64,  // Program headers follow immediately
            e_shoff: 0,        // No section headers by default
            e_flags: RISCV_EFLAGS,
            e_ehsize: Self::SERIALIZED_SIZE as u16,
            e_phentsize: 56,   // Size of 64-bit program header
            e_phnum: 1,        // Single program header entry
            e_shentsize: 64,   // Size of 64-bit section header
            e_shnum: 0,        // No section headers
            e_shstrndx: 0,     // No section header string table
        }
    }
}

// Verification method example
impl ELFHeader {
    pub fn validate(&self) -> Result<(), &'static str> {
        // Verify magic number
        if self.EI_MAG != u8_array_to_u32_little_endian(&ELF_MAGIC).unwrap() {
            return Err("Invalid ELF magic number");
        }

        // Verify 64-bit format
        if self.EI_CLASS != ELF_CLASS_64 {
            return Err("Not a 64-bit ELF file");
        }

        // Verify RISC-V architecture
        if self.e_machine != EM_RISCV {
            return Err("Not a RISC-V ELF file");
        }

        Ok(())
    }

    /// set e_type to ET_DYN for shared object
    pub fn set_dyn_type(&mut self) {
        self.e_type =  ET_DYN; // set to ET_DYN for shared object;
        // self.e_type =  ET_EXEC; // set to ET_EXEC for now as some loaders do not support ET_DYN;
    }

    /// Convert ELF header to hex string with 16 bytes per line
    pub fn to_hex(&self) -> String {
        bytes_to_hex(&self.to_bytes())
    }

    /// Create ELF header from byte slice
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() != Self::SERIALIZED_SIZE {
            return Err("Invalid byte length for ELF header");
        }

        // Safe because we've verified the length and use #[repr(C)]
        let header = unsafe { 
            &*(bytes.as_ptr() as *const ELFHeader)
        };
        
        Ok(*header)
    }

    /// Deserialize from hex string to ELF header
    pub fn deserialize_elf(hex: &str) -> Result<Self, &'static str> {
        let bytes = hex_to_bytes(hex)?;
        let header = Self::from_bytes(&bytes)?;
        header.validate()?;
        Ok(header)
    }
}

impl std::fmt::Debug for ELFHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ELFHeader")
            .field("EI_MAG", &self.EI_MAG)
            .field("EI_CLASS", &self.EI_CLASS)
            .field("EI_DATA", &self.EI_DATA)
            .field("EI_VERSION", &self.EI_VERSION)
            .field("EI_OSABI", &self.EI_OSABI)
            .field("EI_ABIVERSION", &self.EI_ABIVERSION)
            .field("EI_PAD", &self.EI_PAD)
            .field("e_type", &self.e_type)
            .field("e_machine", &self.e_machine)
            .field("e_version", &self.e_version)
            .field("e_entry", &format_args!("{:#x}", self.e_entry))
            .field("e_phoff", &self.e_phoff)
            .field("e_shoff", &self.e_shoff)
            .field("e_flags", &self.e_flags)
            .field("e_ehsize", &self.e_ehsize)
            .field("e_phentsize", &self.e_phentsize)
            .field("e_phnum", &self.e_phnum)
            .field("e_shentsize", &self.e_shentsize)
            .field("e_shnum", &self.e_shnum)
            .field("e_shstrndx", &self.e_shstrndx)
            .finish()
    }
}

impl ShlAssign<&ElfSectionStructure> for ELFHeader {
    fn shl_assign(&mut self, rhs: &ElfSectionStructure) {
        self.e_shoff = rhs.section_headers().get_offset();
        self.e_shnum = rhs.get_section_header_count() as u16;
        self.e_shstrndx = rhs.get_section_name_string_table_index() as u16;
    }
}