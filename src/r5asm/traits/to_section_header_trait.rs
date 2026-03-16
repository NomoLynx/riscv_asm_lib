use super::super::dynamic_structure::*;
use super::super::elf_section::*;
use super::section_size_trait::SectionSizeTrait;

pub trait ToSectionHeaderTrait {
    fn to_section_header(&self) -> ElfSectionHeader;
}

impl ToSectionHeaderTrait for TextSection {
    fn to_section_header(&self) -> ElfSectionHeader {
        ElfSectionHeader::new(
            0,
            1, // SHT_PROGBITS
            0x6, // SHF_ALLOC + SHF_EXECINSTR
            0, // addr
            self.get_offset() as u64,
            self.get_section_data_size() as u64,
            0, // link
            0, // info
            0x1000,
            0, // entsize
        )
    }
}

impl ToSectionHeaderTrait for DataSection {
    fn to_section_header(&self) -> ElfSectionHeader {
        ElfSectionHeader::new(
            0,
            1, // SHT_PROGBITS
            0x3, // SHF_ALLOC + SHF_WRITE
            0, // addr
            self.get_offset() as u64,
            self.get_section_data_size() as u64,
            0, // link
            0, // info
            0x1000,
            0, // entsize
        )
    }
}

impl ToSectionHeaderTrait for ROSection {
    fn to_section_header(&self) -> ElfSectionHeader {
        ElfSectionHeader::new(
            0,
            1, // SHT_PROGBITS
            0x2, // SHF_ALLOC
            0, // addr
            self.get_offset() as u64,
            self.get_section_data_size() as u64,
            0, // link
            0, // info
            0x1000,
            0, // entsize
        )
    }
}


impl ToSectionHeaderTrait for ELFStringTable {
    fn to_section_header(&self) -> ElfSectionHeader {
        ElfSectionHeader::new(
            0,
            3, // SHT_STRTAB
            0, // no flags
            self.get_virtual_address(), // addr
            self.get_offset() as u64,
            self.get_section_data_size() as u64,
            0, // link
            0, // info
            self.get_alignment().get_value() as u64, // align
            0, // entsize
        )
    }
}

impl ToSectionHeaderTrait for DynamicStringTable {
    fn to_section_header(&self) -> ElfSectionHeader {
        ElfSectionHeader::new(
            0,
            3, // SHT_STRTAB
            0, // no flags
            self.0.get_virtual_address(), // addr
            self.0.get_offset() as u64,
            self.0.get_section_data_size() as u64,
            0, // link
            0, // info
            self.0.get_alignment().get_value() as u64, // align
            0, // entsize
        )
    }
}

impl ToSectionHeaderTrait for ELFDynamicStructure {
    fn to_section_header(&self) -> ElfSectionHeader {
        ElfSectionHeader::new(
            0,
            6, // SHT_DYNAMIC
            0x2, // SHF_ALLOC
            self.get_virtual_address(), // addr
            self.get_offset(), // offset
            self.get_section_data_size() as u64,
            0, // link
            0, // info
            self.get_alignment().get_value() as u64, // align
            16, // entsize
        )
    }
}

impl ToSectionHeaderTrait for ELFDynamicSymbolTable {
    fn to_section_header(&self) -> ElfSectionHeader {
        if let Some(global_symbol_starting_index) = self.get_global_symbol_index() {
            if global_symbol_starting_index == 0 {
                panic!("Global symbol index cannot be zero");
            }

            ElfSectionHeader::new(
                0,
                11, // SHT_DYNSYM
                0x2, // SHF_ALLOC
                self.get_virtual_address(), // addr
                self.get_offset(), // offset
                self.get_section_data_size() as u64,
                0, // link
                global_symbol_starting_index as u32, // info field shows where global symbols start
                self.get_alignment().get_value() as u64, // align
                24, // entsize
            )
        }
        else {
            panic!("Global symbol index not set for dynamic symbol table");
        }
    }
}

impl ToSectionHeaderTrait for ELFPLTRelocationTable {
    fn to_section_header(&self) -> ElfSectionHeader {
        // SHT_RELA, SHF_ALLOC and SHF_INFO_LINK (0x40) to indicate link/info usage
        ElfSectionHeader::new(
            0,
            4, // SHT_RELA
            0x2 | 0x40, // SHF_ALLOC | SHF_INFO_LINK
            self.get_virtual_address(), // addr
            self.get_offset(), // offset
            self.get_section_data_size() as u64,
            self.get_link(), // link (e.g., dynsym)
            self.get_info(), // info (e.g., .plt index)
            self.get_alignment().get_value() as u64, // align
            24, // entsize (r_offset + r_info + r_addend for ELF64; 8+8+8)
        )
    }
}

impl ToSectionHeaderTrait for GnuHashSection {
    fn to_section_header(&self) -> ElfSectionHeader {
        ElfSectionHeader::new(
            0,
            0x6ffffff6, // SHT_GNU_HASH
            0x2, // SHF_ALLOC
            self.get_virtual_address(), // addr
            self.get_offset(), // offset
            self.get_section_data_size() as u64,
            0, // link
            0, // info
            self.get_alignment().get_value() as u64, // align
            4, // entsize
        )
    }
}

impl ToSectionHeaderTrait for GOTSection {
    fn to_section_header(&self) -> ElfSectionHeader {
        ElfSectionHeader::new(
            0,
            1, // SHT_PROGBITS
            0x3, // SHF_ALLOC + SHF_WRITE
            self.get_virtual_address(), // addr
            self.get_offset() as u64,
            self.get_section_data_size() as u64,
            0, // link
            0, // info
            self.get_alignment().get_value() as u64, // align
            8, // entsize
        )
    }
}

impl ToSectionHeaderTrait for ELFPLTSection {
    fn to_section_header(&self) -> ElfSectionHeader {
        ElfSectionHeader::new(
            0,
            1, // SHT_PROGBITS
            0x6, // SHF_ALLOC + SHF_EXECINSTR
            self.get_virtual_address(), // addr
            self.get_offset() as u64,
            self.get_section_data_size() as u64,
            0, // link
            0, // info
            self.get_alignment().get_value() as u64, // align
            16, // entsize
        )
    }
}

impl ToSectionHeaderTrait for GnuVersionSection {
    fn to_section_header(&self) -> ElfSectionHeader {
        ElfSectionHeader::new(
            0,
            0x6fffffff, // SHT_GNU_VERSYM
            0x2, // SHF_ALLOC
            self.get_virtual_address(), // addr
            self.get_offset(), // offset
            self.get_section_data_size() as u64,
            0, // need to link to dynsym so loader knows how many versyms are there
            0, // info
            self.get_alignment().get_value() as u64, // align
            2, // entsize
        )
    }
}

impl ToSectionHeaderTrait for GnuVersionRequiredSection {
    fn to_section_header(&self) -> ElfSectionHeader {
        ElfSectionHeader::new(
            0,
            0x6ffffffe, // SHT_GNU_VERNEED
            0x2, // SHF_ALLOC
            self.get_virtual_address(), // addr
            self.get_offset(), // offset
            self.get_section_data_size() as u64,
            0, // link
            self.get_headers().len() as u32, // info, number of version definitions
            self.get_alignment().get_value() as u64, // align
            4, // entsize
        )
    }
}