use super::ELFStringTable;
use std::fmt::Debug;

/// const for dynamic string table name 
pub const DYNAMIC_STRING_TABLE_NAME: &str = ".dynstr";

#[derive(Clone)]
pub struct DynamicStringTable(pub ELFStringTable);

impl DynamicStringTable { 
    /// check if contains the string
    pub fn contains(&self, s: &str) -> bool {
        self.0.contains(s)
    }
}

impl Default for DynamicStringTable {
    fn default() -> Self {
        Self(ELFStringTable::default())
    }
}
impl Debug for DynamicStringTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
} 