use super::super::{dynamic_structure::*, elf_section::note_section::NoteSection};

pub trait SectionNameTrait {
    fn get_section_name(&self) -> String;
}

impl SectionNameTrait for ELFDynamicSymbolTable {
    fn get_section_name(&self) -> String {
        format!(".dynsym")
    }
}

impl SectionNameTrait for ELFStringTable {
    fn get_section_name(&self) -> String {
        format!(".strtab")
    }
}

impl SectionNameTrait for DynamicStringTable {
    fn get_section_name(&self) -> String {
        format!("{}", DYNAMIC_STRING_TABLE_NAME)
    }
}

impl SectionNameTrait for ELFPLTRelocationTable {
    fn get_section_name(&self) -> String {
        format!(".rela.plt")
    }
}

impl SectionNameTrait for ELFPLTSection {
    fn get_section_name(&self) -> String {
        format!(".plt")
    }
}

impl SectionNameTrait for GOTSection {
    fn get_section_name(&self) -> String {
        format!("{}", GOT_SECTION_NAME)
    }
}

impl SectionNameTrait for ELFDynamicStructure {
    fn get_section_name(&self) -> String {
        format!(".dynamic")
    }
}

impl SectionNameTrait for GnuHashSection {
    fn get_section_name(&self) -> String {
        format!(".gnu.hash")
    }
}

impl SectionNameTrait for GnuVersionSection {
    fn get_section_name(&self) -> String {
        format!(".gnu.version")
    }
}

impl SectionNameTrait for GnuVersionRequiredSection {
    fn get_section_name(&self) -> String {
        format!(".gnu.version_r")
    }
}

impl SectionNameTrait for GnuVersionRequiredHeader {
    fn get_section_name(&self) -> String {
        format!(".gnu.version_r.header")
    }
}

impl SectionNameTrait for NoteSection {
    fn get_section_name(&self) -> String {
        format!(".note.{}", self.get_name())
    }
}