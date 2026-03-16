use core_utils::traits::generate_code::GenerateCode;
use rust_macro_internal::*;

#[csv2enum_lookup("src/r5asm/elf_section/section_type/section_type.csv",
    variant, "src/r5asm/elf_section/section_type/section_type_col.csv")]
#[derive(Debug, Clone, PartialEq)]
pub enum SectionType {
    
}

impl SectionType {
    pub(crate) fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            ".TEXT" => Some(SectionType::Text),
            ".BSS" => Some(SectionType::Bss),
            ".DATA" => Some(SectionType::Data),
            ".RODATA" => Some(SectionType::Readonlydata),
            ".DYNAMIC" => Some(SectionType::Dynamic),
            ".INTERP" => Some(SectionType::Interp),
            ".PHDR" => Some(SectionType::Phdr),
            ".PHDRSEGMENT" => Some(SectionType::Phdrsegment),
            ".NOTE" => Some(SectionType::Note),
            _ => None,
        }
    }

    pub (crate) fn to_segment_flag(&self) -> u32 {
        self.lookup_segment_flag() as u32
    }

    /// Convert SectionType to a load type for program segments
    pub (crate) fn to_load_type(&self) -> u32 {
        self.lookup_load_type() as u32
    }
}

impl GenerateCode for SectionType {
    fn generate_code_string(&self) -> String {
        self.get_code()
    }
}