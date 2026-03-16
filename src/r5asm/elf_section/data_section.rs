use super::super::machinecode::MachineCode;
use super::section_wrapper::AlignedMachineCodeSection;
use std::ops::{Deref, DerefMut};

/// Data section with 8-byte alignment - wraps machine code section
#[derive(Debug, Clone)]
pub struct DataSection(AlignedMachineCodeSection<8>);

impl DataSection {
    pub fn new(code: Vec<MachineCode>) -> Self {
        DataSection(AlignedMachineCodeSection::new(code))
    }
}

impl Deref for DataSection {
    type Target = AlignedMachineCodeSection<8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DataSection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<MachineCode>> for DataSection {
    fn from(machine_codes: Vec<MachineCode>) -> Self {
        DataSection::new(machine_codes)
    }
}

impl From<DataSection> for Vec<MachineCode> {
    fn from(data_section: DataSection) -> Self {
        data_section.get_code().clone()
    }
}

impl From<DataSection> for super::super::elf_file::data_section::DataSection {
    fn from(data_section: DataSection) -> Self {
        let v: Vec<MachineCode> = data_section.into();
        v.into()
    }
}