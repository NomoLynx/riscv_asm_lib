
use std::fmt::Display;

use rust_macro_internal::*;
use rust_macro::*;

use super::super::section_type::*;

use super::super::super::elf_file::{bytes_to_hex, hex_to_bytes};

// Program Header Structure
#[packet_struct("src/r5asm/elf_section/program_header/program_header.mermaid")]
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[derive(Accessors)]
pub struct ProgramHeader {

}

#[ini_enum("src/r5asm/elf_section/program_header/program_type.ini", repr = u32)]
enum ProgramType  {

}

#[ini_enum("src/r5asm/elf_section/program_header/segment_flag.ini", repr = u32)]
enum SegmentFlag { }

impl ProgramHeader {
    pub fn new2(section_type:SectionType, offset:u64, vadd:u64, file_size:u64, mem_size:u64, alignment:u64) -> Self {
        Self { segment_type: section_type.to_load_type(), 
                segment_flags: section_type.to_segment_flag(), 
                file_offset : offset, 
                virtual_address : vadd, 
                physical_address: vadd, 
                file_size, 
                memory_size : mem_size, 
                alignment,
            }
    }

    pub fn new_code_program_header(offset: u64, vaddr: u64, file_size: u64, mem_size: u64, alignment: u64) -> Self {
        ProgramHeader {
            segment_type: ProgramType::PROGRAM_TYPE_LOAD as u32,
            segment_flags: SegmentFlag::SEGMENT_FLAG_READ_EXECUTE as u32,
            file_offset: offset,
            virtual_address: vaddr,
            physical_address: vaddr,
            file_size,
            memory_size : mem_size,
            alignment,
        }
    }

    pub fn new_data_program_header(offset: u64, vaddr: u64, file_size: u64, mem_size: u64, alignment: u64) -> Self {
        ProgramHeader {
            segment_type: ProgramType::PROGRAM_TYPE_LOAD as u32,
            segment_flags: SegmentFlag::SEGMENT_FLAG_READ_WRITE as u32,
            file_offset: offset,
            virtual_address: vaddr,
            physical_address: vaddr,
            file_size,
            memory_size : mem_size,
            alignment,
        }
    }

    pub fn new_readonly_data_program_header(offset: u64, vaddr: u64, file_size: u64, mem_size: u64, alignment: u64) -> Self {
        ProgramHeader {
            segment_type: ProgramType::PROGRAM_TYPE_LOAD as u32,
            segment_flags: SegmentFlag::SEGMENT_FLAG_READ as u32,
            file_offset: offset,
            virtual_address: vaddr,
            physical_address: vaddr,
            file_size,
            memory_size : mem_size,
            alignment,
        }
    }

    pub fn new_note_program_header(offset: u64, file_size: u64, alignment: u64) -> Self {
        ProgramHeader {
            segment_type: ProgramType::PROGRAM_TYPE_NOTE as u32,
            segment_flags: SegmentFlag::SEGMENT_FLAG_READ as u32,
            file_offset: offset,
            virtual_address: offset,
            physical_address: offset,
            file_size,
            memory_size : file_size,
            alignment,
        }
    }

    pub fn new_bss_program_header(offset: u64, vaddr: u64, mem_size: u64, alignment: u64) -> Self {
        ProgramHeader {
            segment_type: ProgramType::PROGRAM_TYPE_LOAD as u32,
            segment_flags: SegmentFlag::SEGMENT_FLAG_READ_WRITE as u32,
            file_offset: offset,
            virtual_address: vaddr,
            physical_address: vaddr,
            file_size: 0,
            memory_size : mem_size,
            alignment,
        }
    }

    pub (crate) fn new3(section_type:SectionType, offset:u64, vadd:u64, file_size:u64) -> Self {
        Self::new2(section_type, offset, vadd, file_size, file_size, 0x1000)
    }

    /// new bss segment header
    pub (crate) fn new_bss(offset:u64, vadd:u64, file_size:u64) -> Self {
        Self::new2(SectionType::Bss, offset, vadd, 0, file_size, 0x1000)
    }

    /// new dynamic segment header
    pub (crate) fn new_dynamic(offset:u64, vadd:u64, file_size:u64, alignment:u64) -> Self {
        Self::new2(SectionType::Dynamic, offset, vadd, file_size, file_size, alignment)
    }

    /// new PHDR segment header
    pub (crate) fn new_phdr(file_size:u64, va_offset:u64) -> Self {
        // 0x40 is the starting points for the program header table, right after the elf header
        // alignment is usually 0x40 for phdr
        Self::new2(SectionType::Phdr, 0x40, 0x40 + va_offset, file_size, file_size, 0x8)
    }

    /// create note segment header
    pub (crate) fn new_note(offset:u64, vadd:u64, file_size:u64, alignment:u64) -> Self {
        Self::new2(SectionType::Note, offset, vadd, file_size, file_size, alignment)
    }

    /// new rodata segment for phdr
    pub (crate) fn new_rodata_for_phdr(file_size:u64, va_offset:u64) -> Self {
        Self::new2(SectionType::Readonlydata, 0, va_offset, file_size, file_size, 0x1000)
    }

    /// new interp segment header
    pub (crate) fn new_interp(offset:u64, vadd:u64, file_size:u64, mem_size:u64) -> Self {
        Self::new2(SectionType::Interp, offset, vadd, file_size, mem_size, 0x1)
    }

    /// get the farest offset covered by this segment 
    pub fn farest_offset(&self) -> u64 {
        self.get_file_offset() + self.get_file_size()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() != ProgramHeader::SERIALIZED_SIZE {
            return Err("Invalid program header size");
        }

        Ok(unsafe { 
            std::ptr::read_unaligned(bytes.as_ptr() as *const ProgramHeader) 
        })
    }

    /// Convert to uppercase hex string with space-separated bytes
    pub fn to_hex(&self) -> String {
        bytes_to_hex(&self.to_bytes())
    }

    /// Create from hex string
    pub fn from_hex(hex: &str) -> Result<Self, &'static str> {
        let bytes = hex_to_bytes(hex)?;
        if bytes.len() != ProgramHeader::SERIALIZED_SIZE {
            return Err("Invalid hex length for program header");
        }
        ProgramHeader::from_bytes(&bytes)
    }

    pub fn is_interp_segment(&self) -> bool {
        self.segment_type == ProgramType::PROGRAM_TYPE_INTERP as u32
    }

    pub fn is_dynamic_segment(&self) -> bool {
        self.segment_type == ProgramType::PROGRAM_TYPE_DYNAMIC as u32
    }

    pub fn is_phdr_segment(&self) -> bool {
        self.segment_type == ProgramType::PROGRAM_TYPE_PHDR as u32
    }

    pub fn is_note_segment(&self) -> bool {
        self.segment_type == ProgramType::PROGRAM_TYPE_NOTE as u32
    }
}

impl Display for ProgramHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SegmentHeader {{ load_type: 0x{:1X}, flag: 0x{:02X}, offset: 0x{:X}, vadd: 0x{:X}, padd: 0x{:X}, file_size: 0x{:X}, mem_size: 0x{:X}, alignment: 0x{:X} }}", 
                                    self.get_segment_type(),
                                    self.get_segment_flags(), 
                                    self.get_file_offset(), 
                                    self.get_virtual_address(), 
                                    self.get_physical_address(), 
                                    self.get_file_size(), 
                                    self.get_memory_size(), 
                                    self.get_alignment())
    }
}

impl std::fmt::Debug for ProgramHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SegmentHeader {{ load_type: 0x{:1X}, flag: 0x{:02X}, offset: 0x{:X}, vadd: 0x{:X}, padd: 0x{:X}, file_size: {}/0x{:X}, mem_size: {}/0x{:X}, alignment: 0x{:X} }}\n", 
                                    self.get_segment_type(), 
                                    self.get_segment_flags(), 
                                    self.get_file_offset(), 
                                    self.get_virtual_address(), 
                                    self.get_physical_address(), 
                                    self.get_file_size(), self.get_file_size(),
                                    self.get_memory_size(), self.get_memory_size(),
                                    self.get_alignment())
    }
}
