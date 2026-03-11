use super::super::machinecode::MachineCode;
use super::machine_code_section::MachineCodeSection;

#[derive(Debug, Clone)]
pub struct AlignedMachineCodeSection<const ALIGN: u32>(MachineCodeSection);

impl<const ALIGN: u32> AlignedMachineCodeSection<ALIGN> {
	pub fn new(code: Vec<MachineCode>) -> Self {
		AlignedMachineCodeSection(MachineCodeSection::new_with_alignment(code, ALIGN))
	}

	pub fn get_code(&self) -> &Vec<MachineCode> {
		self.0.get_code()
	}

	pub fn get_virtual_address(&self) -> u64 {
		self.0.get_virtual_address()
	}

	pub fn set_virtual_address(&mut self, vadd: u64) {
		self.0.set_virtual_address(vadd);
	}

	pub fn get_offset(&self) -> u64 {
		self.0.get_offset()
	}
}

impl<const ALIGN: u32> From<Vec<MachineCode>> for AlignedMachineCodeSection<ALIGN> {
	fn from(machine_codes: Vec<MachineCode>) -> Self {
		AlignedMachineCodeSection::new(machine_codes)
	}
}

impl<const ALIGN: u32> From<AlignedMachineCodeSection<ALIGN>> for Vec<MachineCode> {
	fn from(section: AlignedMachineCodeSection<ALIGN>) -> Self {
		section.get_code().clone()
	}
}
