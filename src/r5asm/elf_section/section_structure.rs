use std::{fmt::{self, Debug, Formatter}, ops::ShlAssign};

use super::super::{alignment::Alignment, dynamic_structure::ELFDynamicStructure, elf_section::*};

type SectionHeaderList = Elf64SectionHeaderList;

/// ELF section structure contains section headers and sections
pub struct ElfSectionStructure {
    /// List of section headers or section data
    section_headers: SectionHeaderList,
    //sections 
    sections: Vec<RawBinarySection>,
    /// Section name string table (usually .shstrtab)
    section_name_strtab: SectionNameStringTable,

    /// offset of the section headers in the final ELF file
    section_offset: usize,
    alignment : Alignment,
}

impl ElfSectionStructure {
    pub fn sections(&self) -> &Vec<RawBinarySection> {
        &self.sections
    }

    pub fn sections_mut(&mut self) -> &mut Vec<RawBinarySection> {
        &mut self.sections
    }

    pub fn set_sections(&mut self, sections: Vec<RawBinarySection>) {
        self.sections = sections;
    }

    pub fn section_headers(&self) -> &SectionHeaderList {
        &self.section_headers
    }

    pub fn section_headers_mut(&mut self) -> &mut SectionHeaderList {
        &mut self.section_headers
    }

    pub fn set_section_headers(&mut self, headers: SectionHeaderList) {
        self.section_headers = headers;
    }

    pub fn section_name_strtab(&self) -> &SectionNameStringTable {
        &self.section_name_strtab
    }

    pub fn section_name_strtab_mut(&mut self) -> &mut SectionNameStringTable {
        &mut self.section_name_strtab
    }

    pub fn set_section_name_strtab(&mut self, strtab: SectionNameStringTable) {
        self.section_name_strtab = strtab;
    }

    pub fn section_offset(&self) -> usize {
        self.section_offset
    }

    pub fn set_section_offset(&mut self, offset: usize) {
        let v = self.alignment.calculate_padding_and_offset(offset as u64) as usize;
        self.section_offset = v;
    }

    pub fn get_section_header_count(&self) -> usize {
        self.section_headers().len()
    }

    pub fn get_section_name_string_table_index(&self) -> usize {
        let strtab = self.section_name_strtab();
        for (i, n) in self.section_headers().iter().enumerate() {
            let name_offset = n.get_name() as usize;
            if let Some(name) = strtab.get_name_by_offset(name_offset) {
                if name == ".shstrtab" {
                    return i;
                }
            }
        }

        0 // Default to 0 if not found
    }

    pub fn get_section_name_table_bytes(&self) -> Vec<u8> {
        let mut r = self.alignment.get_padding_vec();
        r.extend(self.section_name_strtab().to_bytes());
        r
    }

    pub fn get_section_bytes(&self) -> Vec<u8> {
        let mut r = self.get_section_name_table_bytes();
        for section in self.sections() {
            r.extend(section.get_section_data());
        }
        
        r
    }

    /// get size of section data in bytes, excluding alignment padding
    pub fn get_section_data_size(&self) -> usize {
        self.get_section_bytes().len() - self.alignment.get_padding() as usize
    }

    pub fn get_section_header_bytes(&self) -> Vec<u8> {
        let section_headers = self.section_headers();
        section_headers.to_bytes()
    }

    /// create a new text section header and add to section headers
    pub fn new_text_section_header(&mut self, offset:u64, addr: u64, size: u64) {
        let name = ".text";
        let name_offset = self.section_name_strtab_mut().add_name(name);
        let v = ElfSectionHeader::new(
            name_offset as u32,
            1, // SHT_PROGBITS
            6, // SHF_ALLOC + SHF_EXECINSTR
            addr,
            offset, // offset will be set later
            size,
            0, // link
            0, // info
            16, // addralign
            0, // entsize
        );
        self.section_headers_mut().add(name, v);
    }

    /// create a new data section header and add to section headers
    pub fn new_data_section_header(&mut self, offset:u64, addr: u64, size: u64) {
        let name = ".data";
        let name_offset = self.section_name_strtab_mut().add_name(name);
        let v = ElfSectionHeader::new(
            name_offset as u32,
            1, // SHT_PROGBITS
            3, // SHF_ALLOC + SHF_WRITE
            addr,
            offset, // offset will be set later
            size,
            0, // link
            0, // info
            8, // addralign
            0, // entsize
        );
        self.section_headers_mut().add(name, v);
    }

    /// create a new rodata section header and add to section headers
    pub fn new_rodata_section_header(&mut self, offset:u64, addr: u64, size: u64) {
        let name = ".rodata";
        let name_offset = self.section_name_strtab_mut().add_name(name);
        let v = ElfSectionHeader::new(
            name_offset as u32,
            1, // SHT_PROGBITS
            2, // SHF_ALLOC
            addr,
            offset, // offset will be set later
            size,
            0, // link
            0, // info
            8, // addralign
            0, // entsize
        );
        self.section_headers_mut().add(name, v);
    }

    /// perform internal update, such as updating .shstrtab offset
    /// should be called after all sections are added and before writing to ELF file
    pub fn update(&mut self) {
        // set section header offset
        {
            let offset = self.get_section_data_size() + self.section_offset();
            self.section_headers_mut().set_offset(offset as u64);
        }

        // Update .shstrtab section header's offset, section header is added in default constructor
        {
            let offset = self.section_offset() as u64;
            let shstrtab_index = self.get_section_name_string_table_index();
            let shstrtab_size = self.get_section_name_table_bytes().len() as u64;
            let shstrtab_header = &mut self.section_headers_mut()[shstrtab_index];
            shstrtab_header.set_offset(offset);
            shstrtab_header.set_size(shstrtab_size);
        }

        // set up the link and info fields for sections that need them
        {
            // gnu hash section links to dynamic symbol table
            if let Some(dynsym_index) = self.section_headers().find_dynamic_symboles_section_index() {
                if let Some(gnu_hash_index) = self.section_headers().find_gnu_hash_section_index() {
                    let gnu_hash_header = &mut self.section_headers_mut()[gnu_hash_index];
                    gnu_hash_header.set_link(dynsym_index as u32);
                }
            }

            // dynamic string table required to link to: dynamic section, dynamic symbol, gnu version required.
            if let Some(dynstr_index) = self.section_headers().find_dynamic_string_table_section_index() {
                if let Some(dynsym_index) = self.section_headers().find_dynamic_symboles_section_index() {
                    let dynsym_header = &mut self.section_headers_mut()[dynsym_index];
                    dynsym_header.set_link(dynstr_index as u32);
                }

                if let Some(dyn_index) = self.section_headers().find_dynamic_section_index() {
                    let dyn_header = &mut self.section_headers_mut()[dyn_index];
                    dyn_header.set_link(dynstr_index as u32);
                }

                if let Some(gnu_verneed_index) = self.section_headers().find_gnu_version_required_section_index() {
                    let gnu_verneed_header = &mut self.section_headers_mut()[gnu_verneed_index];
                    gnu_verneed_header.set_link(dynstr_index as u32);
                }
            }

            // gnu version link to dynamic symbol table
            if let Some(dynsym_index) = self.section_headers().find_dynamic_symboles_section_index() {
                if let Some(gnu_versym_index) = self.section_headers().find_gnu_version_section_index() {
                    let gnu_versym_header = &mut self.section_headers_mut()[gnu_versym_index];
                    gnu_versym_header.set_link(dynsym_index as u32);
                }
            }

            // rela.plt section links to dynamic symbol table and info is .got section index
            if let Some(rela_plt_index) = self.section_headers().find_rela_plt_section_index() {
                if let Some(dynsym_index) = self.section_headers().find_dynamic_symboles_section_index() {
                    let rela_plt_header = &mut self.section_headers_mut()[rela_plt_index];
                    rela_plt_header.set_link(dynsym_index as u32);
                }

                if let Some(got_index) = self.section_headers().find_got_section_index() {
                    let rela_plt_header = &mut self.section_headers_mut()[rela_plt_index];
                    rela_plt_header.set_info(got_index as u32);
                }
            }
        }
    }
}

impl Default for ElfSectionStructure {
    fn default() -> Self {
        let mut section_name_strtab = SectionNameStringTable::default();
        section_name_strtab.add_name(""); // first entry is empty string
        let name = section_name_strtab.add_name(".shstrtab");

        let mut r = Self {
            sections: Vec::new(),
            section_headers: SectionHeaderList::default(),
            section_name_strtab,
            section_offset: 0,
            alignment : Alignment::new(16).unwrap(),
        };

        r.section_headers_mut().add(".shstrtab", ElfSectionHeader::new(
            name as u32,
            3, // SHT_STRTAB
            0, // flags
            0, // addr
            0, // offset will be set later
            0, // size will be set later
            0, // link
            0, // info
            1, // addralign
            0, // entsize
        ));

        r
    }
}

impl Debug for ElfSectionStructure {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ElfSectionStructure")
            .field("sections", &self.sections)
            .field("section_name_strtab", &self.section_name_strtab)
            .field("section_offset", &self.section_offset)
            .field("section_headers", &self.section_headers)
            .finish()
    }
}

impl ShlAssign<&ELFDynamicStructure> for ElfSectionStructure {
    fn shl_assign(&mut self, rhs: &ELFDynamicStructure) {
        let section_headers = rhs.to_section_headers();
        for (name, mut header) in section_headers.into_iter() {
            if header.is_all_zero() {
                continue;
            }

            let name_offset = self.section_name_strtab.add_name(&name);
            header.set_name(name_offset as u32);
            self.section_headers_mut().add(&name, header);
        }
    }
}