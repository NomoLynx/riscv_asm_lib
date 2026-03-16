use rust_macro::*;

#[derive(Debug, Clone, Accessors)]
pub struct LinkerConfig {
    virutual_address_start: u64,
    is_build_lib : bool,
    soname : Option<String>,
}

impl LinkerConfig {
    pub fn new(virutual_address_start: u64) -> Self {
        Self {
            virutual_address_start,
            is_build_lib : false,
            soname : None,
        }
    }
}

impl Default for LinkerConfig {
    fn default() -> Self {
        Self::new(0x8100_0000)
    }
}