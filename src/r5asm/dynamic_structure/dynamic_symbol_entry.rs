use rust_macro::*;
use rust_macro_internal::*;
use std::fmt::{self, Debug, Formatter};

use crate::r5asm::{dynamic_structure::DynamicSymbolInfo, traits::ToMarkdownTableRow};

#[packet_struct("src/r5asm/dynamic_structure/dynamic_symbol_entry.mermaid")]
#[derive(Accessors)]
#[derive(Clone)]
pub struct DynamicSymbolEntry {

}

impl DynamicSymbolEntry {
    pub fn get_name_offset(&self) -> u32 {
        self.st_name
    }

    /// set the value (address) of the symbol, e.g., function address or offset
    pub fn set_value(&mut self, value:u64) {
        self.st_value = value;
    }

    /// set the size of the symbol
    pub fn set_symbol_size(&mut self, size:u64) {
        self.st_size = size;
    }

    pub fn get_symbol_size(&self) -> u64 {
        self.st_size
    }

    /// create a new external global function entry
    pub fn new_extern_global_function_entry(st_name:u32) -> Self {
        let st_info = DynamicSymbolInfo::GlobalFunction.into();
        Self::new(st_name, st_info, // STB_GLOBAL, STT_FUNC which is 0x12
                  0, // default visibility
                  0, // SHN_UNDEF (0, since it’s external)
                  0, // 0 (resolved at runtime)
                  0) // size is 0 for now
    }

    /// create a new global no type entry
    pub fn new_global_notype_entry(st_name:u32, st_value:u64) -> Self {
        let st_info = DynamicSymbolInfo::GlobalObject.into();
        Self::new(st_name, st_info, // STB_GLOBAL, STT_NOTYPE which is 0x10
                  0, // default visibility
                  0, // SHN_UNDEF (0, since it’s external)
                  st_value, // virtual address of the global pointer
                  0) // size of the symbol
    }

    /// create a new local section entry
    pub fn new_local_section_entry(st_name:u32, st_value:u64) -> Self {
        let st_info = DynamicSymbolInfo::LocalSection.into();
        Self::new(st_name, st_info, // STB_LOCAL, STT_SECTION which is 0x03
                  0, // default visibility
                  1, // SHN_ABS (1, absolute section)
                  st_value, // virtual address of the section
                  0) // size of the symbol
    }

    /// create a new global function entry
    pub fn new_global_function_entry(st_name:u32, st_value:u64) -> Self {
        let st_info = DynamicSymbolInfo::GlobalFunction.into();
        Self::new(st_name, st_info, // STB_GLOBAL, STT_FUNC which is 0x12
                  0, // default visibility
                  1, // SHN_ABS (1, absolute section)
                  st_value, // virtual address of the function
                  0) // size of the symbol
    }

    /// get (bind, type) from st_info
    fn get_info(&self) -> (u8, u8) {
        let bind = self.st_info >> 4;
        let typ = self.st_info & 0x0F;
        (bind, typ)
    }

    pub fn get_dynamic_symbol_info(&self) -> DynamicSymbolInfo {
        let (bind, typ) = self.get_info();
        (bind, typ).into()
    }

    pub fn is_global(&self) -> bool {
        self.get_dynamic_symbol_info().is_global()
    }

    pub fn get_size(&self) -> usize {
        Self::SERIALIZED_SIZE
    }
}

impl Debug for DynamicSymbolEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let symbol_section_index = match self.st_shndx { 
            0 => "SHN_UNDEF(0)",
            1 => "SHN_ABS(1)",
            2 => "SHN_COMMON(2)",
            n if n >= 0xFF00 => &format!("SHN_LORESERVE(0x{:X})", n),
            n => &format!("SectionIndex(0x{:X})", n),
        };

        let symbol_size = if self.st_size == 0 { "UnknownSize(0)" } else { &format!("size = 0x{:X} bytes", self.st_size) };
        let (bind, typ) = self.get_info();
        let dyn_symbol_info: DynamicSymbolInfo = (bind, typ).into();
        let info = self.st_info;
        let visibility = match self.st_other {
            0 => format!("Default({})", self.st_other),
            1 => format!("Internal({})", self.st_other),
            2 => format!("Hidden({})", self.st_other),
            3 => format!("Protected({})", self.st_other),
            _ => format!("Unknown({})", self.st_other),
        };

        write!(f, "DynSymbolEntry {{ name @ 0x{:X}, {dyn_symbol_info:?}(0x{info:X}), Visibility = {visibility}, {symbol_section_index}, value = 0x{:X}, {symbol_size} }}", 
            self.st_name, self.st_value)
    }
}

impl Default for DynamicSymbolEntry {
    fn default() -> Self {
        Self { st_name: 0, st_info: 0, st_other: 0, st_shndx: 0, st_value: 0, st_size: 0 }
    }
}

impl ToMarkdownTableRow for DynamicSymbolEntry {
    fn get_markdown_header(&self) -> String {
        "| st_name | st_info | st_other | st_shndx | st_value | st_size |
|---------|---------|---------|---------|---------|---------|".to_string()
    }
    
    fn to_markdown(&self) -> String {
        format!(
            "| {} (0x{:X}) | {} (0x{:X}) | {} (0x{:X}) | {} (0x{:X}) | {} (0x{:X}) | {} (0x{:X}) |",
            self.st_name, self.st_name,
            self.st_info, self.st_info,
            self.st_other, self.st_other,
            self.st_shndx, self.st_shndx,
            self.st_value, self.st_value,
            self.st_size, self.st_size
        )
    }
}
