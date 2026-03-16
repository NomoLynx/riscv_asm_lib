use super::*;
use rust_macro::*;
use rust_macro_internal::*;

#[packet_struct("src/r5asm/elf_section/section_header.mermaid")]
#[derive(Clone, PartialEq, Eq, Accessors)]
pub struct ElfSectionHeader {

}

impl ElfSectionHeader {
    pub fn is_all_zero(&self) -> bool {
        self.name == 0 &&
        self.section_type == 0 &&
        self.flags == 0 &&
        self.addr == 0 &&
        self.offset == 0 &&
        self.size == 0 &&
        self.link == 0 &&
        self.info == 0 &&
        self.addralign == 0 &&
        self.entsize == 0
    }

    /// deserialize from byte slice
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 64 {
            return None; // Not enough data for ELF64 section header
        }

        let name = u32::from_le_bytes(data[0..4].try_into().unwrap());
        let section_type = u32::from_le_bytes(data[4..8].try_into().unwrap());
        let flags = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let addr = u64::from_le_bytes(data[16..24].try_into().unwrap());
        let offset = u64::from_le_bytes(data[24..32].try_into().unwrap());
        let size = u64::from_le_bytes(data[32..40].try_into().unwrap());
        let link = u32::from_le_bytes(data[40..44].try_into().unwrap());
        let info = u32::from_le_bytes(data[44..48].try_into().unwrap());
        let addralign = u64::from_le_bytes(data[48..56].try_into().unwrap());
        let entsize = u64::from_le_bytes(data[56..64].try_into().unwrap());

        Some(Self {
            name,
            section_type,
            flags,
            addr,
            offset,
            size,
            link,
            info,
            addralign,
            entsize,
        })
    }
}

impl std::fmt::Debug for ElfSectionHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ElfSectionHeader")
            .field("name", &self.name)
            .field("section_type", &self.section_type)
            .field("flags", &self.flags)
            .field("addr", &self.addr)
            .field("offset", &self.offset)
            .field("size", &self.size)
            .field("link", &self.link)
            .field("info", &self.info)
            .field("addralign", &self.addralign)
            .field("entsize", &self.entsize)
            .finish()
    }
}

impl From<&RawBinarySection> for ElfSectionHeader {
    fn from(section: &RawBinarySection) -> Self {
        Self {
            name: section.name_offset() as u32,
            section_type: 0, // Placeholder, set appropriately
            flags: 0,        // Placeholder, set appropriately
            addr: 0,         // Placeholder, set appropriately
            offset: 0,       // Placeholder, set appropriately
            size: section.data().len() as u64,
            link: 0,         // Placeholder, set appropriately
            info: 0,         // Placeholder, set appropriately
            addralign: section.alignment().get_value() as u64,
            entsize: 0,      // Placeholder, set appropriately
        }
    }
}

impl Default for ElfSectionHeader {
    fn default() -> Self {
        Self {
            name: 0,
            section_type: 0,
            flags: 0,
            addr: 0,
            offset: 0,
            size: 0,
            link: 0,
            info: 0,
            addralign: 0,
            entsize: 0,
        }
    }
}