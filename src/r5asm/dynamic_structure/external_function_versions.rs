use parser_lib::markdown_lang::*;

fn md_table_to_string_row_table(file_name: &str) -> Result<StringValueTable, parser_lib::common::ParsingError> {
    let md_file = load_md_file(file_name)?;
    let tables = md_file.get_tables();
    let table =
        if tables.is_empty() {
            return Err(parser_lib::common::ParsingError::NoFound(
                (file!().to_string(), line!()).into(),
                format!("MD file has 0 table, The file is located at {file_name}"),
            ));
        } else {
            tables.first().unwrap()
        };

    let mut rows = Vec::default();
    for row in table.data_rows().to_owned().into_iter() {
        let mut output_row = Vec::default();
        for cell in row.iter() {
            output_row.push(cell.get_text().trim().to_string());
        }
        rows.push(output_row);
    }

    Ok(StringValueTable::new(rows))
}

pub type VersionEntry = (String, String, String); // (function name, library file name, version)

#[derive(Debug, Clone)]
pub struct ExternalFunctionVersions {
    values : Vec<VersionEntry>,
}

impl ExternalFunctionVersions {
    pub fn find_version(&self, function_name: &str) -> Option<(String, Vec<String>)> {
        let mut results: Vec<String> = Vec::new();
        let mut lib_name = String::new();
        for (func, lib, ver) in &self.values {
            if func == function_name {
                results.push(ver.to_string());

                if lib_name.is_empty() {
                    lib_name = lib.to_string();
                }
                else {
                    if lib_name != *lib {
                        // different library name found, return None
                        return None;
                    }
                }
            }
        }

        if lib_name.is_empty() {
            return None;
        }
        
        Some(( lib_name, results ))      
    }
}

/// implement iterator for ExternalFunctionVersions
impl IntoIterator for ExternalFunctionVersions {
    type Item = VersionEntry;
    type IntoIter = std::vec::IntoIter<VersionEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

/// implement iterator for &ExternalFunctionVersions not consume it
/// so that we can iterate multiple times
impl<'a> IntoIterator for &'a ExternalFunctionVersions {
    type Item = VersionEntry;
    type IntoIter = std::vec::IntoIter<VersionEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.clone().into_iter()
    }
}

impl Default for ExternalFunctionVersions {
    fn default() -> Self {
        let string_value_table = md_table_to_string_row_table("r5asm_data/external_linux_functions.md").unwrap();
        string_value_table.into()
    }
}

impl From<&StringValueTable> for ExternalFunctionVersions {
    fn from(table: &StringValueTable) -> Self {
        let mut values: Vec<VersionEntry> = Vec::new();
        for row in table.get_table() {
            if row.len() >= 3 {
                let function_name = row[0].clone();
                let library_name = row[1].clone();
                let version = row[2].clone();
                values.push((function_name, library_name, version));
            }
        }
        ExternalFunctionVersions { values }
    }
}

impl From<StringValueTable> for ExternalFunctionVersions {
    fn from(table: StringValueTable) -> Self {
        ExternalFunctionVersions::from(&table)
    }
}