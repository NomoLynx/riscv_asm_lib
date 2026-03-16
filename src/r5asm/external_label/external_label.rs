use super::super::{asm_error::AsmError, dynamic_structure::{DynamicSymbolInfo, ELFDynamicStructure}};
use std::fmt::Debug;

use super::*;

#[derive(Clone)]
pub struct ExternalLabel {
    name: String,
    value: ExternalLabelValue,
}

impl ExternalLabel {
    pub fn new(name: String) -> Self {
        ExternalLabel { name, value: ExternalLabelValue::Undefined }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_value(&self) -> &ExternalLabelValue {
        &self.value
    }

    pub fn set_value(&mut self, value: ExternalLabelValue) {
        self.value = value;
    }

    pub fn update_value(&mut self, ds: &ELFDynamicStructure) -> Result<(), AsmError> {
        let dynamic_string_table = ds.get_dynamic_string_table();
        let symbol_name = self.get_name();
        let symbol_table = ds.get_dynamic_symbol_table();
        if let Some(dyn_symbol) = symbol_table.get_symbol_entry_by_name(symbol_name, dynamic_string_table) {
            match dyn_symbol.get_dynamic_symbol_info() { 
                DynamicSymbolInfo::GlobalFunction | 
                DynamicSymbolInfo::WeakFunction => {
                    let plt_address = ds.find_symbol_plt_virtual_address(symbol_name);
                    if let Some(v) = plt_address {
                        self.set_value(ExternalLabelValue::Address(v));
                        Ok(())
                    }
                    else {
                        Err(AsmError::NoFound((file!(), line!()).into(), format!("cannot find plt address for external function symbol '{symbol_name}'")))
                    }
                }
                _ => todo!("need to add logic to process other type of symbol"),
            }
        }
        else {
            Ok(())
        }
    }
}

impl From<&str> for ExternalLabel {
    fn from(name: &str) -> Self {
        name.to_string().into()
    }
}

impl From<String> for ExternalLabel {
    fn from(name: String) -> Self {
        ExternalLabel::new(name)
    }
}

impl From<&String> for ExternalLabel {
    fn from(name: &String) -> Self {
        name.clone().into()
    }
}

impl PartialEq for ExternalLabel {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Debug for ExternalLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExternLabel {{ name: {}, value: {:?} }}", self.name, self.value)
    }
}