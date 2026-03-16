use super::super::dynamic_structure::*;

pub trait ToDataSectionTrait {
    fn to_data_section(&self) -> super::super::elf_file::data_section::DataSection;
}


impl ToDataSectionTrait for GOTSection {
    fn to_data_section(&self) -> super::super::elf_file::data_section::DataSection {
        super::super::elf_file::data_section::DataSection::new(self.get_data())
    }
}

impl ToDataSectionTrait for ELFDynamicStructure {
    fn to_data_section(&self) -> super::super::elf_file::data_section::DataSection {
        super::super::elf_file::data_section::DataSection::new(self.get_dyanmic_bytes())
    }
}

impl ToDataSectionTrait for ELFDynamicSymbolTable {
    fn to_data_section(&self) -> super::super::elf_file::data_section::DataSection {
        super::super::elf_file::data_section::DataSection::new(self.get_bytes())
    }
}

impl ToDataSectionTrait for ELFStringTable {
    fn to_data_section(&self) -> super::super::elf_file::data_section::DataSection {
        super::super::elf_file::data_section::DataSection::new(self.to_bytes())
    }
}

impl ToDataSectionTrait for ELFPLTRelocationTable {
    fn to_data_section(&self) -> super::super::elf_file::data_section::DataSection {
        super::super::elf_file::data_section::DataSection::new(self.get_data())
    }
}

impl ToDataSectionTrait for GnuHashSection {
    fn to_data_section(&self) -> super::super::elf_file::data_section::DataSection {
        super::super::elf_file::data_section::DataSection::new(self.to_bytes())
    }
}

impl ToDataSectionTrait for GnuVersionSection {
    fn to_data_section(&self) -> super::super::elf_file::data_section::DataSection {
        super::super::elf_file::data_section::DataSection::new(self.to_bytes())
    }
}

impl ToDataSectionTrait for GnuVersionRequiredSection {
    fn to_data_section(&self) -> super::super::elf_file::data_section::DataSection {
        super::super::elf_file::data_section::DataSection::new(self.to_bytes())
    }
}