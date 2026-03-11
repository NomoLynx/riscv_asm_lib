use std::{fs::File, io::Write, ops::ShlAssign, vec};

use core_utils::filesystem::read_file_content;

use core_utils::debug::*;
use super::super::elf_section::*;

use super::{code_section::CodeSection, data_section::{DataSection, ReadOnlySection}};
use super::super::elf_section::NoteSection;
use super::*;
use super::super::elf_section::program_header::*;
use super::traits::*;

pub type ElfHeader = Elf64Header;

// Updated ELF file structure with clear names
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElfFile {
    pub header: ElfHeader,
    pub program_headers: Vec<ProgramHeader>,
    pub segments: Vec<Vec<u8>>,
    pub section_headers: Vec<u8>,

    /// data does not need to be loaded to memory and only stay on disk
    section : Vec<Vec<u8>>, 
    alignment : u32,
}

impl ElfFile {
    pub fn new() -> Self {
        let header = ElfHeader::default();
        Self::new_with_data(header, Vec::new(), Vec::new())
    }

    fn new_with_data(header:ElfHeader, program_headers : Vec<ProgramHeader>, data : Vec<Vec<u8>>) -> Self {
        Self {
            header,
            program_headers, 
            segments : data,
            alignment : 0x1000,
            section : Vec::default(),
            section_headers: Vec::default(),
        }
    }

    pub fn set_entry_point(&mut self, entry: u64) {
        self.header.set_e_entry(entry);
    }

    /// set ELF file type to ET_DYN for dynamic linking
    pub fn set_to_dynamic(&mut self) {
        self.header.set_dyn_type();
    }

    pub fn add_code_section(&mut self, code: CodeSection, vaddr: u64) {
        let size = code.code.len() as u64;
        let alignment = self.alignment as u64;
        let offset = self.next_segment_offset();
        let phdr = ProgramHeader::new_code_program_header(offset, vaddr, size, size, alignment);
        self.program_headers.push(phdr);
        self.segments.push(code.code);
        self.update_header();
    }

    pub fn add_data_section(&mut self, data: DataSection, vaddr: u64) {
        let size = data.data.len() as u64;
        let alignment = self.alignment as u64;
        let offset = self.next_segment_offset();
        let phdr = ProgramHeader::new_data_program_header(offset, vaddr, size, size, alignment);
        self.program_headers.push(phdr);
        self.segments.push(data.data);
        self.update_header();
    }

    /// add dynamic section to the ELF file
    pub fn add_dynamic_section(&mut self, segment_header: &ProgramHeader) {
        let phdr: ProgramHeader = segment_header.clone(); 
        self.program_headers.push(phdr);
        self.segments.push(vec![]);    //this is necessary because to_bytes() requires program header entry MUST have a segment
        self.update_header();
    }

    /// add interpreter section to the ELF file
    pub fn add_interpreter_section(&mut self, segment_header:&ProgramHeader) {
        let phdr = segment_header.clone();
        self.program_headers.push(phdr);
        self.segments.push(vec![]);    //this is necessary because to_bytes() requires program header entry MUST have a segment
        self.update_header();
    }

    /// add phdr section to the ELF file
    pub fn add_phdr_section(&mut self, segment_header:&ProgramHeader) {
        let phdr = segment_header.clone();
        self.program_headers.push(phdr);
        self.segments.push(vec![]);    //this is necessary because to_bytes() requires program header entry MUST have a segment
        self.update_header();
    }

    /// add loadable segment to host phdr and elf header
    pub fn add_segment_for_phdr(&mut self, segment_header:&ProgramHeader) {
        let phdr = segment_header.clone();
        self.program_headers.push(phdr);
        self.segments.push(vec![]);    //this is necessary because to_bytes() requires program header entry MUST have a segment
        self.update_header();
    }

    // Add read-only section to the ELF file
    pub fn add_read_only_section(&mut self, data: ReadOnlySection, vaddr: u64) {
        let size = data.data.len() as u64;
        let alignment = self.alignment as u64;
        let offset = self.next_segment_offset();
        let phdr = ProgramHeader::new_readonly_data_program_header(offset, vaddr, size, size, alignment);
        self.program_headers.push(phdr);
        self.segments.push(data.data);
        self.update_header();
    }

    /// Add a note section to the ELF file
    pub fn add_note_section(&mut self, note: NoteSection, vaddr: u64) {
        let bytes = note.to_bytes();
        let size = bytes.len() as u64;
        let alignment = note.get_alignment().get_value() as u64;

        let phdr = ProgramHeader::new_note_program_header(vaddr, size, alignment);

        self.program_headers.push(phdr);
        self.segments.push(bytes);
        self.update_header();
    }

    /// add bss section to the ELF file
    pub fn add_bss_section(&mut self, vaddr: u64, size: u64) {
        let alignment = self.alignment as u64;
        let offset = self.next_segment_offset();
        let phdr = ProgramHeader::new_bss_program_header(offset, vaddr, size, alignment);        
        self.program_headers.push(phdr);
        self.segments.push(vec![]); //this is necessary because to_bytes() requires program header entry MUST have a segment
        self.update_header();
    }

    fn next_segment_offset(&self) -> u64 {
        let previous_segment_size = self.program_headers.iter()
            .map(|x| if x.is_interp_segment() || x.is_dynamic_segment() || x.is_phdr_segment() { 0 } 
                                     else { self.round_up(x.get_file_size()) } )
            .sum::<u64>();

        // if there is a segement with zero offset, it means the elf header and phdr are already included
        // so we do not need to add their size again
        let contain_zero_offset = self.program_headers.iter().any(|x| x.get_file_offset() == 0);
        if contain_zero_offset {
            self.round_up(previous_segment_size)
        }
        else {
            let r = previous_segment_size + 
                    self.header.get_e_phoff() + 
                    ((self.program_headers.len() + 1) as u64 * ProgramHeader::SERIALIZED_SIZE as u64);
            
            self.round_up(r)
        }
    }

    /// round up to alignment for a given value
    pub fn round_up(&self, value: u64) -> u64 {
        let r = value + self.alignment as u64 - 1;
        r - (r % self.alignment as u64)
    }

    /// Update ELF header fields based on current program headers, mainly header length 
    fn update_header(&mut self) {
        self.header.set_e_phnum(self.program_headers.len() as u16);
    }

    /// get program header index for note segment, return None if not found
    pub fn get_note_segment_index(&self) -> Option<usize> {
        for (i, phdr) in self.program_headers.iter().enumerate() {
            if phdr.is_note_segment() {
                return Some(i);
            }
        }
        None
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // check if program headers and segments are consistent
        if self.program_headers.len() != self.segments.len() {
            let err_msg = format!("Inconsistent ELF file: program headers count {} does not match segments count {}", self.program_headers.len(), self.segments.len());
            error_str(&err_msg);
            panic!("{}", err_msg);
        }

        let mut bytes = Vec::new();
        
        // Serialize ELF header
        bytes.extend(self.header.to_bytes());

        // Serialize program headers
        for phdr in &self.program_headers {
            let header_bytes = phdr.to_bytes();
            bytes.extend(header_bytes);
        }

        // serialize note segment first if exists
        // because it is usually the first segment in ELF file
        let note_index = self.get_note_segment_index();
        if let Some(index) = note_index {
            let phdr = &self.program_headers[index];
            let segment = &self.segments[index];
            let offset = phdr.get_file_offset() as usize;
            
            // Pad with zeros if needed
            if bytes.len() < offset {
                bytes.resize(offset, 0);
            }
            
            // append the segment data
            if offset + segment.len() > bytes.len() {
                bytes.extend_from_slice(segment);
            } else {
                panic!("offset + offset cannot be smaller than current bytes length which means overwrite")
            }
        }

        // Serialize segment data with proper padding
        for (header, segment) in self.program_headers.iter().zip(&self.segments) {
            if segment.is_empty() {
                continue; // Skip empty segments (e.g., BSS)
            }

            // skip note segment because it is already added
            if header.is_note_segment() {
                continue;
            }

            let offset = header.get_file_offset() as usize;
            
            // Pad with zeros if needed
            if bytes.len() < offset {
                bytes.resize(offset, 0);
            }
            
            // append the segment data
            if offset + segment.len() > bytes.len() {
                bytes.extend_from_slice(segment);
            } else {
                panic!("offset + offset cannot be smaller than current bytes length which means overwrite")
            }
        }

        // add disk only data in section structure
        for data in self.section.clone().into_iter() {
            bytes.extend_from_slice(&data);
        }

        // add section headers
        bytes.extend(self.section_headers.clone());

        bytes
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < Elf64Header::SERIALIZED_SIZE {
            return Err("Data too small for ELF header");
        }

        let header = unsafe { &*(data.as_ptr() as *const ElfHeader) };
        header.validate()?;

        // Read program headers using correct ELF field names
        let mut program_headers = Vec::new();
        let ph_start = header.get_e_phoff() as usize;  // Program header table offset
        let ph_size = header.get_e_phentsize() as usize;  // Size of each program header
        let ph_count = header.get_e_phnum() as usize;  // Number of program headers
        let section_header_offset = header.get_e_shoff() as usize; // Section header table offset
        let section_header_size = header.get_e_shentsize() as usize; // Size of each section
        let section_header_count = header.get_e_shnum() as usize; // Number of section headers

        // Validate program header table bounds
        let ph_end = ph_start + (ph_size * ph_count);
        if ph_end > data.len() {
            return Err("Program header table exceeds file size");
        }

        // Parse each program header
        for i in 0..ph_count {
            let start = ph_start + (i * ph_size);
            let end = start + ph_size;
            let phdr = ProgramHeader::from_bytes(&data[start..end])?;
            program_headers.push(phdr);
        }

        // Read segment data
        let mut segments = Vec::with_capacity(ph_count);
        for phdr in &program_headers {
            let start = phdr.get_file_offset() as usize;
            let end = start + phdr.get_file_size() as usize;
            
            if end > data.len() {
                return Err("Segment data exceeds file size");
            }
            
            segments.push(data[start..end].to_vec());
        }

        let overall_alignment = program_headers[0].get_alignment() as u32;

        // read section headers
        let mut section_headers = Vec::default();
        if section_header_count > 0 {
            let sh_start = section_header_offset;
            let sh_size = section_header_size;
            let sh_count = section_header_count;

            // Validate section header table bounds
            let sh_end = sh_start + (sh_size * sh_count);
            if sh_end > data.len() {
                return Err("Section header table exceeds file size");
            }

            // Parse each section header
            section_headers = data[sh_start..sh_end].to_vec();
        }

        let section = Vec::new();

        Ok(ElfFile {
            header: *header,
            program_headers,
            segments,
            section,
            alignment : overall_alignment,
            section_headers,
        })
    }

    // Hex conversion functions
    pub fn to_hex(&self) -> String {
        bytes_to_hex(&self.to_bytes())
    }

    pub fn from_hex(hex: &str) -> Result<Self, &'static str> {
        let bytes = hex_to_bytes(hex)?;
        ElfFile::from_bytes(&bytes)
    }

    /// Save the ELF file to disk
    pub fn save(&self, filename: &str) -> Result<(), String> {
        let bytes = self.to_bytes();
        File::create(filename)
            .map_err(|e| format!("Failed to create file: {}", e))?
            .write_all(&bytes)
            .map_err(|e| format!("Failed to write data: {}", e))?;
        Ok(())
    }

    /// from markdown file to ELF file structure
    pub fn from_markdown_file(filename: &str) -> Result<Self, String> {
        let content = read_file_content(filename).map_err(|e| format!("Failed to read file: {}", e))?;
        let markdown = parser_lib::markdown_lang::markdown_pest::parse(&content).map_err(|e| format!("Failed to parse markdown: {:?}", e))?;
        
        // read elf header
        let code = markdown.get_codes_after_header("ELF Header");
        let bin_str = code.get(0).unwrap().get_code();
        let header = ElfHeader::deserialize_elf(bin_str)?;

        // read program headers
        let mut prog_headers = vec![];
        for item in markdown.get_codes_after_header("Program Headers") {
            let bin_str = item.get_code();
            let ph = ProgramHeader::from_hex(bin_str)?;
            prog_headers.push(ph);
        }

        // read segments
        let mut segments = vec![];
        for item in markdown.get_codes_after_header("Segments") {
            let bin_str = item.get_code();
            let data = hex_to_bytes(&bin_str)?;
            segments.push(data);
        }

        let elf_file = ElfFile::new_with_data(header, prog_headers, segments);
        Ok(elf_file)
    }

    pub fn save_md_file(&self, filename: &str) -> Result<(), String> {
        let md_str = self.to_markdown();
        File::create(filename)
            .map_err(|e| format!("Failed to create file: {}", e))?
            .write_all(md_str.as_bytes())
            .map_err(|e| format!("Failed to write data: {}", e))?;
        Ok(())
    }

    pub fn set_section_headers(&mut self, section_headers: Vec<u8>) {
        self.section_headers = section_headers;
    }
}

impl ShlAssign<&ElfSectionStructure> for ElfFile {
    fn shl_assign(&mut self, rhs: &ElfSectionStructure) {
        self.set_section_headers(rhs.get_section_header_bytes());
        self.section.push( rhs.get_section_bytes() );
        self.header <<= rhs;
    }
}