use super::super::{dynamic_structure::*, elf_section::*};

pub trait SectionSizeTrait {
    /// Returns the total size of the section, including any padding.
    fn get_section_size(&self) -> usize;

    /// Returns the size of the actual data in the section, excluding padding.
    fn get_section_data_size(&self) -> usize;
}

impl SectionSizeTrait for ELFDynamicSymbolTable {
    fn get_section_size(&self) -> usize {
        self.get_entries().len() * 24  // each entry is 24 bytes for riscv64
        + self.get_alignment().get_padding() as usize
    }

    fn get_section_data_size(&self) -> usize {
        self.get_entries().len() * 24
    }
}

impl SectionSizeTrait for ELFStringTable {
    fn get_section_size(&self) -> usize {
        self.get_size() + self.get_alignment().get_padding() as usize
    }

    fn get_section_data_size(&self) -> usize {
        self.get_size()
    }
}

impl SectionSizeTrait for ELFPLTRelocationTable {
    fn get_section_size(&self) -> usize {
        self.get_size() + self.get_alignment().get_padding() as usize
    }

    fn get_section_data_size(&self) -> usize {
        self.get_size()
    }
}

impl SectionSizeTrait for ELFPLTSection {
    fn get_section_size(&self) -> usize {
        self.get_size() + self.get_alignment().get_padding() as usize
    }

    fn get_section_data_size(&self) -> usize {
        self.get_size()
    }
}

impl SectionSizeTrait for GOTSection {
    fn get_section_size(&self) -> usize {
        self.get_section_data_size() + self.get_alignment().get_padding() as usize
    }

    fn get_section_data_size(&self) -> usize {
        self.get_size()
    }
}

impl SectionSizeTrait for ELFDynamicStructure {
    fn get_section_size(&self) -> usize {
        self.get_dynamic_size()   // dynamic entry list size
        + self.get_alignment().get_padding() as usize  // plus padding
    }

    fn get_section_data_size(&self) -> usize {
        self.get_dynamic_size()
    }
}

impl SectionSizeTrait for GnuHashSection {
    fn get_section_size(&self) -> usize {
        self.get_section_data_size() + self.get_alignment().get_padding() as usize
    }

    fn get_section_data_size(&self) -> usize {
        self.get_size()
    }
}

impl SectionSizeTrait for TextSection {
    fn get_section_size(&self) -> usize {
        self.get_section_data_size()
    }

    fn get_section_data_size(&self) -> usize {
        self.get_code().iter().map(|mc| mc.get_size()).sum()
    }
}

impl SectionSizeTrait for DataSection {
    fn get_section_size(&self) -> usize {
        self.get_section_data_size()
    }

    fn get_section_data_size(&self) -> usize {
        self.get_code().iter().map(|mc| mc.get_size()).sum()
    }
}

impl SectionSizeTrait for ROSection {
    fn get_section_size(&self) -> usize {
        self.get_section_data_size()
    }

    fn get_section_data_size(&self) -> usize {
        self.get_code().iter().map(|mc| mc.get_size()).sum()
    }
}


impl SectionSizeTrait for GnuVersionSection {
    fn get_section_size(&self) -> usize {
        self.get_section_data_size() + self.get_alignment().get_padding() as usize
    }

    fn get_section_data_size(&self) -> usize {
        self.get_size_in_bytes()
    }
}

impl SectionSizeTrait for GnuVersionRequiredSection {
    fn get_section_size(&self) -> usize {
        self.get_section_data_size() + self.get_alignment().get_padding() as usize
    }

    fn get_section_data_size(&self) -> usize {
        self.get_size_in_bytes()
    }
}

impl SectionSizeTrait for NoteSection {
    fn get_section_size(&self) -> usize {
        self.get_section_data_size() + self.get_alignment().get_padding() as usize
    }

    fn get_section_data_size(&self) -> usize {
        self.get_size_in_bytes()
    }
}