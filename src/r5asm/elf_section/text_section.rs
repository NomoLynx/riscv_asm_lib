use super::super::{elf_file::code_section::CodeSection, machinecode::MachineCode};
use super::section_wrapper::AlignedMachineCodeSection;
use std::ops::{Deref, DerefMut};

/// Text (code) section with 4-byte alignment - wraps machine code section
#[derive(Debug, Clone)]
pub struct TextSection(AlignedMachineCodeSection<4>);

impl TextSection {
    pub fn new(codes: Vec<MachineCode>) -> Self {
        TextSection(AlignedMachineCodeSection::new(codes))
    }
}

impl Deref for TextSection {
    type Target = AlignedMachineCodeSection<4>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TextSection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<MachineCode>> for TextSection {
    fn from(machine_codes: Vec<MachineCode>) -> Self {
        TextSection::new(machine_codes)
    }
}

impl From<TextSection> for Vec<MachineCode> {
    fn from(text_section: TextSection) -> Self {
        text_section.get_code().clone()
    }
}

impl From<TextSection> for CodeSection {
    fn from(text_section: TextSection) -> Self {
        let x: Vec<MachineCode> = text_section.into();
        x.into()
    }
}