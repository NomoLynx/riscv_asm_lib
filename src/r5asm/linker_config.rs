use rust_macro::*;

#[derive(Debug, Clone, Accessors)]
pub struct LinkerConfig {
    virutual_address_start: u64,
    is_build_lib : bool,
    soname : Option<String>,
    needed_libs: Vec<String>,
}

impl LinkerConfig {
    pub fn new(virutual_address_start: u64) -> Self {
        Self {
            virutual_address_start,
            ..Default::default()
        }
    }

    pub fn add_needed_lib(&mut self, lib_name:&str) {
        let lib_name = lib_name.trim();
        if lib_name.is_empty() {
            return;
        }

        if !self.needed_libs.iter().any(|x| x == lib_name) {
            self.needed_libs.push(lib_name.to_string());
        }
    }

    pub fn set_needed_libraries_from_csv(&mut self, libs:&str) {
        for lib in libs.split(',') {
            self.add_needed_lib(lib);
        }
    }
}

impl Default for LinkerConfig {
    fn default() -> Self {
        Self {
            virutual_address_start : 0x8100_0000,
            is_build_lib : false,
            soname : None,
            needed_libs : Vec::default(),
        }
    }
}
