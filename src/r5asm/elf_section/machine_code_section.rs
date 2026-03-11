use super::super::{alignment::*, machinecode::MachineCode};

/// Shared machine code section struct for loadable ELF sections (text, data, read-only data)
#[derive(Debug, Clone)]
pub struct MachineCodeSection {
    pub(crate) code: Vec<MachineCode>,
    pub(crate) offset: u64,
    pub(crate) alignment: Alignment,
}

impl MachineCodeSection {
    /// Create a new ELF section with the specified alignment size
    pub(crate) fn new_with_alignment(code: Vec<MachineCode>, alignment_size: u32) -> Self {
        MachineCodeSection {
            code,
            offset: 0,
            alignment: Alignment::new(alignment_size).unwrap(),
        }
    }

    pub fn get_code(&self) -> &Vec<MachineCode> {
        &self.code
    }

    pub fn get_virtual_address(&self) -> u64 {
        self.offset
    }

    pub fn set_virtual_address(&mut self, vadd: u64) {
        let v = self.alignment.calculate_padding_and_offset(vadd);
        self.offset = v;
    }

    pub fn get_offset(&self) -> u64 {
        self.offset
    }
}

impl From<Vec<MachineCode>> for MachineCodeSection {
    fn from(machine_codes: Vec<MachineCode>) -> Self {
        MachineCodeSection::new_with_alignment(machine_codes, 8)
    }
}

impl From<MachineCodeSection> for Vec<MachineCode> {
    fn from(section: MachineCodeSection) -> Self {
        section.get_code().clone()
    }
}
