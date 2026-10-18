use std::path::PathBuf;

use rust_macro::*;

use crate::r5asm::dynamic_structure::ExternalFunctionVersions;

#[derive(Debug, Clone, Accessors)]
pub struct ExternalFileFunctions {
    file_name : String, 
    external_functions : ExternalFunctionVersions
}

impl From<&PathBuf> for ExternalFileFunctions {
    fn from(value: &PathBuf) -> Self {
        let file_name = value.file_name()
                                            .and_then(|x| x.to_str())
                                            .unwrap_or("");
        let mut lvalue = value.clone();
        lvalue.set_extension("lnk");
        let external_functions = lvalue.into();

        Self {
            file_name: file_name.to_string(),
            external_functions,
        }        
    }
}

impl From<PathBuf> for ExternalFileFunctions {
    fn from(value: PathBuf) -> Self {
        Self::from(&value)
    }
}

#[derive(Debug, Clone, Accessors, GetMut)]
pub struct LinkerConfig {
    virutual_address_start: u64,
    is_build_lib : bool,
    soname : Option<String>,
    needed_libs: Vec<String>,
    external_file_functions : Vec<ExternalFileFunctions>,
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

    /// Effective virtual-address base used for ELF layout.
    ///
    /// Executables keep the configured base address (default 0x8100_0000),
    /// while shared libraries are position-independent and use a zero base.
    pub fn get_effective_virtual_address_start(&self) -> u64 {
        if self.is_build_lib {
            0
        } else {
            self.virutual_address_start
        }
    }

    /// Processes external functions for all needed libraries.
    /// 
    /// # Arguments
    ///
    /// * `base_path` - The base path where the external library files are located.
    pub fn process_exteranl_function(&mut self, base_path:&PathBuf) {
        let mut external_file_functions = vec![];
        for file in self.get_needed_libs() {
            let mut base_path_clone = base_path.clone();
            base_path_clone.push(file);
            let v = base_path_clone.into();
            external_file_functions.push(v);
        }

        self.set_external_file_functions(external_file_functions);
    }
}

impl Default for LinkerConfig {
    fn default() -> Self {
        Self {
            virutual_address_start : 0x8100_0000,
            is_build_lib : false,
            soname : None,
            needed_libs : Vec::default(),
            external_file_functions : Vec::default(),
        }
    }
}
