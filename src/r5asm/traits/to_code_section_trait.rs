use super::super::dynamic_structure::*;

pub trait ToCodeSectionTrait {
    fn to_code_section(&self) -> super::super::elf_file::code_section::CodeSection;
}

impl ToCodeSectionTrait for ELFPLTSection {
    fn to_code_section(&self) -> super::super::elf_file::code_section::CodeSection {
        super::super::elf_file::code_section::CodeSection::new(self.get_code().to_vec())
    }
}
