use std::{collections::HashMap, fmt::{Display, Formatter}};

pub use core_utils::file_object::FileObject;

pub struct ASMSolution {
    container_path: Option<String>,
    main_file : FileObject, // FileObject is a struct that contains the file path and the file content
    data_file : Option<FileObject>,
    folder_name : HashMap<String, Vec<FileObject>>,
}

impl ASMSolution {
    pub fn new(main_file: FileObject) -> Self {
        Self {
            container_path: None,
            main_file,
            data_file: None,
            folder_name: HashMap::new(),
        }
    }

    pub fn set_container_path(&mut self, path: &str) {
        self.container_path = Some(path.to_string());
    }

    pub fn add_file(&mut self, folder_name: &str, file: FileObject) {
        if !self.folder_name.contains_key(folder_name) {
            self.folder_name.insert(folder_name.to_string(), vec![]);
        }
        self.folder_name.get_mut(folder_name).unwrap().push(file);
    }

    pub fn get_main_file(&self) -> &FileObject {
        &self.main_file
    }

    pub fn get_main_file_name(&self) -> String {
        if self.container_path.is_none() {
            format!("{}.{}", self.main_file.get_file_name(), self.main_file.get_file_extension())
        }
        else {
            format!("{}/{}.{}", self.container_path.as_ref().unwrap(), self.main_file.get_file_name(), self.main_file.get_file_extension())
        }
    }

    pub fn get_output_file_name(&self) -> String {
        if self.container_path.is_none() {
            format!("{}.elf", self.main_file.get_file_name())
        }
        else {
            format!("{}/{}.elf", self.container_path.as_ref().unwrap(), self.main_file.get_file_name())
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

        self.folder_name.get_mut(&data_folder_name).unwrap().push(file);
    }

    /// add ini file to data folder
    pub fn add_ini_file(&mut self, var_name: &str, ini_data: &str) {
        let main_name = self.main_file.get_file_name();
        let file = FileObject::new(var_name, "ini", ini_data);
        let data_folder_name = format!("{}.data", main_name);
        self.create_folder(&data_folder_name);

        self.folder_name.get_mut(&data_folder_name).unwrap().push(file);
    }

    /// add data file to data folder
    pub fn add_data_file(&mut self, data_file: FileObject) {
        let main_name = self.main_file.get_file_name();
        let data_folder_name = format!("{}.data", main_name);
        self.create_folder(&data_folder_name);

        self.folder_name.get_mut(&data_folder_name).unwrap().push(data_file);
    }

    /// save to hard disk
    pub fn save_to_disk(&mut self, path: &str) {
        self.set_container_path(path);
        
        // create folder
        std::fs::create_dir_all(path).unwrap();

        // save main file
        let main_file_path = format!("{}/{}.{}", path, self.main_file.get_file_name(), self.main_file.get_file_extension());
        std::fs::write(main_file_path, self.main_file.get_content()).unwrap();

        // save data file
        if let Some(data_file) = &self.data_file {
            let data_file_path = format!("{}/{}.{}", path, data_file.get_file_name(), data_file.get_file_extension());
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
                let file_path = format!("{}/{}.{}", folder_path, file.get_file_name(), file.get_file_extension());
                std::fs::write(file_path, file.get_content()).unwrap();
            }
        }
    }
}

impl Display for ASMSolution {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut r = String::new();
        r.push_str(&format!("Main File:\r\n{}\r\n", self.main_file));

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