use super::super::{elf_file::data_section::ReadOnlySection, machinecode::MachineCode};
use super::section_wrapper::AlignedMachineCodeSection;
use std::ops::{Deref, DerefMut};

/// Read-only data section with 8-byte alignment - wraps machine code section
#[derive(Debug, Clone)]
pub struct ROSection(AlignedMachineCodeSection<8>);

impl ROSection {
    pub fn new(code: Vec<MachineCode>) -> Self {
        ROSection(AlignedMachineCodeSection::new(code))
    }
}

impl Deref for ROSection {
    type Target = AlignedMachineCodeSection<8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ROSection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<MachineCode>> for ROSection {
    fn from(machine_codes: Vec<MachineCode>) -> Self {
        ROSection::new(machine_codes)
    }
}

impl From<ROSection> for Vec<MachineCode> {
    fn from(ro_section: ROSection) -> Self {
        ro_section.get_code().clone()
    }
}

impl From<ROSection> for ReadOnlySection {
    fn from(ro_section: ROSection) -> Self {
        let v: Vec<MachineCode> = ro_section.into();
        v.into()
    }
}