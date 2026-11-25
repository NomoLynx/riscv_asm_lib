use std::{
    collections::HashMap,
    fmt::{Display, Formatter}, path::{Path, PathBuf},
};

use crate::r5asm::{asm_error::AsmError, code_gen_config::CodeGenConfiguration};
pub use core_utils::file_object::FileObject;
use core_utils::string::*;
use core_utils::{filesystem::get_file_name_without_extension, number::get_u64_from_str};
use parser_lib::ini::{get_ini_properties, parse_ini_from_file};

use super::linker_config::LinkerConfig;

pub enum CompileFileAction {
    None,
    ProcessAsType,
}

#[derive(Default)]
pub struct FileAction {
    file_name: String,
    action: Vec<CompileFileAction>,
}

#[derive(Default)]
pub struct ASMSolution {
    container_path: Option<String>,
    main_file: FileObject, // FileObject is a struct that contains the file path and the file content
    source_files: Vec<FileObject>,
    data_file: Option<FileObject>,
    folder_name: HashMap<String, Vec<FileObject>>,
    output_folder: Option<String>,
    file_actions: Vec<FileAction>,
}

impl ASMSolution {
    pub fn new(main_file: FileObject) -> Self {
        let mut source_files = Vec::new();
        source_files.push(FileObject::new(
            main_file.get_file_name(),
            main_file.get_file_extension(),
            main_file.get_content(),
        ));

        Self {
            main_file,
            source_files,
            ..Default::default()
        }
    }

    pub fn set_container_path(&mut self, path: &str) {
        self.container_path = Some(path.to_string());
    }

    pub fn set_output_folder(&mut self, output_folder: &str) {
        self.output_folder = Some(output_folder.to_string());
    }

    pub fn add_file(&mut self, folder_name: &str, file: FileObject) {
        if !self.folder_name.contains_key(folder_name) {
            self.folder_name.insert(folder_name.to_string(), vec![]);
        }
        self.folder_name.get_mut(folder_name).unwrap().push(file);
    }

    pub fn add_source_file(&mut self, file: FileObject) {
        if !self.source_files.iter().any(|x| {
            x.get_file_name() == file.get_file_name()
                && x.get_file_extension() == file.get_file_extension()
        }) {
            self.source_files.push(file);
        }
    }

    pub fn get_source_files(&self) -> &[FileObject] {
        &self.source_files
    }

    pub fn get_combined_source(&self) -> String {
        let mut combined = String::new();
        for (idx, file) in self.source_files.iter().enumerate() {
            if idx > 0 {
                combined.push_str("\r\n\r\n");
            }
            combined.push_str(file.get_content());
        }
        combined
    }

    pub fn has_multiple_source_files(&self) -> bool {
        self.source_files.len() > 1
    }

    pub fn get_main_file(&self) -> &FileObject {
        &self.main_file
    }

    pub fn get_main_file_name(&self) -> String {
        if self.container_path.is_none() {
            format!(
                "{}.{}",
                self.main_file.get_file_name(),
                self.main_file.get_file_extension()
            )
        } else {
            format!(
                "{}/{}.{}",
                self.container_path.as_ref().unwrap(),
                self.main_file.get_file_name(),
                self.main_file.get_file_extension()
            )
        }
    }

    pub fn get_output_file_name(&self) -> String {
        self.get_output_file_name_with_extension("elf")
    }

    pub fn get_output_file_name_with_linker_config(&self, linker_config: &LinkerConfig) -> String {
        if linker_config.get_is_build_lib() {
            self.get_output_file_name_with_extension("so")
        } else {
            self.get_output_file_name_with_extension("elf")
        }
    }

    fn get_output_file_name_with_extension(&self, extension: &str) -> String {
        match (self.container_path.as_ref(), self.output_folder.as_ref()) {
            (Some(container_path), Some(output_folder)) => {
                let container = container_path.trim_end_matches("/");
                let output_folder = output_folder.trim_end_matches("/");
                return format!(
                    "{container}/{output_folder}/{}.{}",
                    self.main_file.get_file_name(),
                    extension
                );
            }
            (Some(container_path), None) => {
                let container = container_path.trim_end_matches("/");
                return format!(
                    "{container}/{}.{}",
                    self.main_file.get_file_name(),
                    extension
                );
            }
            (None, Some(output_folder)) => {
                let output_folder = output_folder.trim_end_matches("/");
                return format!(
                    "{output_folder}/{}.{}",
                    self.main_file.get_file_name(),
                    extension
                );
            }
            (None, None) => {
                return format!("{}.{}", self.main_file.get_file_name(), extension);
            }
        }
    }

    pub fn from_code_string(code: &str) -> Self {
        let main_file = FileObject::new("main", "s", code);
        Self::new(main_file)
    }

    fn is_folder_exists(&self, folder_name: &str) -> bool {
        self.folder_name.contains_key(folder_name)
    }

    fn create_folder(&mut self, folder_name: &str) {
        if !self.is_folder_exists(folder_name) {
            self.folder_name.insert(folder_name.to_string(), vec![]);
        }
    }

    fn get_folder_names(&self) -> Vec<String> {
        self.folder_name.keys().cloned().collect()
    }

    /// add md file to data folder
    pub fn add_md_file(&mut self, var_name: &str, md_data: &str) {
        let main_name = self.main_file.get_file_name();
        let file = FileObject::new(var_name, "md", md_data);
        let data_folder_name = format!("{}.data", main_name);
        self.create_folder(&data_folder_name);

        self.folder_name
            .get_mut(&data_folder_name)
            .unwrap()
            .push(file);
    }

    /// add ini file to data folder
    pub fn add_ini_file(&mut self, var_name: &str, ini_data: &str) {
        let main_name = self.main_file.get_file_name();
        let file = FileObject::new(var_name, "ini", ini_data);
        let data_folder_name = format!("{}.data", main_name);
        self.create_folder(&data_folder_name);

        self.folder_name
            .get_mut(&data_folder_name)
            .unwrap()
            .push(file);
    }

    /// add data file to data folder
    pub fn add_data_file(&mut self, data_file: FileObject) {
        let main_name = self.main_file.get_file_name();
        let data_folder_name = format!("{}.data", main_name);
        self.create_folder(&data_folder_name);

        self.folder_name
            .get_mut(&data_folder_name)
            .unwrap()
            .push(data_file);
    }

    /// save to hard disk
    pub fn save_to_disk(&mut self, path: &str) {
        self.set_container_path(path);

        // create folder
        std::fs::create_dir_all(path).unwrap();

        // save main file
        let main_file_path = format!(
            "{}/{}.{}",
            path,
            self.main_file.get_file_name(),
            self.main_file.get_file_extension()
        );
        std::fs::write(main_file_path, self.main_file.get_content()).unwrap();

        // save additional source files
        for file in self.source_files.iter().skip(1) {
            let file_path = format!(
                "{}/{}.{}",
                path,
                file.get_file_name(),
                file.get_file_extension()
            );
            std::fs::write(file_path, file.get_content()).unwrap();
        }

        // save data file
        if let Some(data_file) = &self.data_file {
            let data_file_path = format!(
                "{}/{}.{}",
                path,
                data_file.get_file_name(),
                data_file.get_file_extension()
            );
            std::fs::write(data_file_path, data_file.get_content()).unwrap();
        }

        // remove all folders
        for folder_name in self.get_folder_names() {
            let folder_path = format!("{}/{}", path, folder_name);
            if std::path::Path::new(&folder_path).exists() {
                std::fs::remove_dir_all(&folder_path).unwrap();
            }
        }

        // save data files
        for (folder_name, files) in &self.folder_name {
            let folder_path = format!("{}/{}", path, folder_name);
            std::fs::create_dir_all(&folder_path).unwrap();
            for file in files {
                let file_path = format!(
                    "{}/{}.{}",
                    folder_path,
                    file.get_file_name(),
                    file.get_file_extension()
                );
                std::fs::write(file_path, file.get_content()).unwrap();
            }
        }
    }

    fn resolve_ini_file_path(base_folder: &Path, raw_value: &str) -> String {
        let path_value = remove_prefix_suffix_double_quote(raw_value);
        let candidate = PathBuf::from(&path_value);
        if candidate.is_absolute() {
            path_value
        } else {
            base_folder.join(candidate).to_string_lossy().to_string()
        }
    }

    pub fn form_init_file(
        ini_file_path: &str,
    ) -> Result<(ASMSolution, CodeGenConfiguration), AsmError> {
        let ini = parse_ini_from_file(ini_file_path).map_err(|e| {
            AsmError::GeneralError(
                (file!(), line!()).into(),
                format!("failed to parse ini: {e:?}"),
            )
        })?;

        let properties = get_ini_properties(&ini);
        let ini_base_folder = Path::new(ini_file_path)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        let main_file_raw = properties
            .get("asm_main")
            .or_else(|| properties.get("main"))
            .or_else(|| properties.get("main_file"))
            .cloned()
            .unwrap_or_else(|| "main.s".to_string());
        let main_file_path = Self::resolve_ini_file_path(&ini_base_folder, &main_file_raw);

        let output_folder_raw = properties
            .get("asm_output_folder")
            .or_else(|| properties.get("output_folder"))
            .cloned();

        let source_paths = properties
            .get("asm_source")
            .or_else(|| properties.get("source"))
            .or_else(|| properties.get("source_files"))
            .map(|value| {
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|part| !part.is_empty())
                    .map(|part| Self::resolve_ini_file_path(&ini_base_folder, part))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let main_content = std::fs::read_to_string(&main_file_path).map_err(|e| {
            AsmError::GeneralError(
                (file!(), line!()).into(),
                format!("failed to read main asm file '{main_file_path}': {e}"),
            )
        })?;

        let main_stem = get_file_name_without_extension(&main_file_path)
            .unwrap_or("main".to_string())
            .to_string();

        let mut solution = ASMSolution::new(FileObject::new(&main_stem, "s", &main_content));

        let ini_base_folder_str = ini_base_folder.to_string_lossy().to_string();
        solution.set_container_path(&ini_base_folder_str);

        if let Some(output_folder) = output_folder_raw {
            solution.set_output_folder(&output_folder);
        }

        for item in source_paths {
            if item.is_empty() || item == main_file_path {
                continue;
            }

            let content = std::fs::read_to_string(&item).map_err(|e| {
                AsmError::GeneralError(
                    (file!(), line!()).into(),
                    format!("failed to read source file '{item}': {e}"),
                )
            })?;

            let file_stem = get_file_name_without_extension(&item)
                .unwrap_or("source".to_string())
                .to_string();
            let ext = Path::new(&item)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("s")
                .to_string();

            solution.add_source_file(FileObject::new(&file_stem, &ext, &content));
        }

        let mut config = CodeGenConfiguration::default();

        if let Some(value) = properties
            .get("replace_pseudo_code")
            .or_else(|| properties.get("codegen_replace_pseudo_code"))
        {
            config.set_replace_pseudo_code(string_to_bool(value));
        }

        if let Some(value) = properties
            .get("generate_bin_and_code")
            .or_else(|| properties.get("codegen_generate_bin_and_code"))
        {
            config.set_generate_bin_and_code(string_to_bool(value));
        }

        if let Some(value) = properties
            .get("build_target")
            .or_else(|| properties.get("codegen_build_target"))
        {
            config.set_build_target(value.trim().parse::<u8>().unwrap_or(8));
        }

        let virtual_address = properties
            .get("virtual_address_start")
            .or_else(|| properties.get("linker_virtual_address_start"))
            .or_else(|| properties.get("start_address"))
            .or_else(|| properties.get("linker_start_address"));

        if let Some(value) = virtual_address {
            config
                .get_linker_config_mut()
                .set_virutual_address_start(get_u64_from_str(value).unwrap_or(0));
        }

        if let Some(value) = properties
            .get("is_build_lib")
            .or_else(|| properties.get("linker_is_build_lib"))
        {
            config
                .get_linker_config_mut()
                .set_is_build_lib(string_to_bool(value));
        }

        if let Some(value) = properties
            .get("soname")
            .or_else(|| properties.get("linker_soname"))
        {
            config
                .get_linker_config_mut()
                .set_soname(Some(value.trim().to_string()));
        }

        if let Some(value) = properties
            .get("asm_needed_libs")
            .or_else(|| properties.get("asm_linker_needed_libs"))
            .or_else(|| properties.get("linker_needed_libs"))
            .or_else(|| properties.get("needed_libs"))
        {
            config
                .get_linker_config_mut()
                .set_needed_libraries_from_csv(value);
            config
                .get_linker_config_mut()
                .process_exteranl_function(&ini_base_folder);
        }

        Ok((solution, config))
    }
}

impl Display for ASMSolution {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut r = String::new();
        r.push_str(&format!("Main File:\r\n{}\r\n", self.main_file));

        if self.source_files.len() > 1 {
            r.push_str("Additional source files:\r\n");
            for file in self.source_files.iter().skip(1) {
                r.push_str(&format!("{}\r\n", file));
            }
        }

        if let Some(data_file) = &self.data_file {
            r.push_str(&format!("Data File:\r\n{}\r\n", data_file));
        }

        for (folder_name, files) in &self.folder_name {
            r.push_str(&format!("Folder: {}\r\n", folder_name));
            for file in files {
                r.push_str(&format!("{}\r\n", file));
            }
        }

        write!(f, "{}", r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::r5asm::linker_config::LinkerConfig;

    #[test]
    fn asm_solution_tracks_multiple_source_files() {
        let mut solution =
            ASMSolution::new(FileObject::new("main", "s", ".text\naddi x1, x2, 1\n"));
        solution.add_source_file(FileObject::new("helper", "s", ".text\naddi x3, x4, 2\n"));

        assert!(solution.has_multiple_source_files());
        assert_eq!(solution.get_source_files().len(), 2);
        assert!(solution.get_combined_source().contains("addi x1, x2, 1"));
        assert!(solution.get_combined_source().contains("addi x3, x4, 2"));
    }

    #[test]
    fn asm_solution_outputs_shared_library_name_when_linker_is_library_mode() {
        let solution = ASMSolution::new(FileObject::new("main", "s", ".text\nret\n"));
        let mut linker_config = LinkerConfig::default();
        linker_config.set_is_build_lib(true);

        assert_eq!(
            solution.get_output_file_name_with_linker_config(&linker_config),
            "main.so"
        );
    }
}
