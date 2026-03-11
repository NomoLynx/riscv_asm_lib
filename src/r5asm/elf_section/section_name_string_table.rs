#[derive(Clone, Debug)]
pub struct SectionNameStringTable {
    names: Vec<String>,
}

impl SectionNameStringTable {
    pub fn new() -> Self {
        SectionNameStringTable { names: Vec::new() }
    }

    /// insert name and get offset (not index) as return
    pub fn add_name(&mut self, name: &str) -> usize {
        let offset = self.names.iter().map(|s| s.len() + 1).sum(); // +1 for null terminator
        self.names.push(name.to_string());
        offset        
    }

    pub fn get_name(&self, index: usize) -> Option<&str> {
        self.names.get(index).map(|s| s.as_str())
    }

    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.names.iter()
    }

    pub fn get_name_by_offset(&self, offset: usize) -> Option<&str> {
        let mut current_offset = 0;
        for name in self.iter() {
            if current_offset == offset {
                return Some(name.as_str());
            }
            current_offset += name.len() + 1; // +1 for null terminator
        }
        None
    }

    /// convert to bytes with null terminators, suitable for writing to ELF file
    /// Note: does not include the initial null byte for the first entry
    /// the super is because this is only used in the parent module and not public, non-loaded data are sharing the alignment in the parent module
    pub (super) fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for name in &self.names {
            bytes.extend(name.as_bytes());
            bytes.push(0); // null terminator
        }
        bytes
    }
}

impl Default for SectionNameStringTable {
    fn default() -> Self {
        Self::new()
    }    
}