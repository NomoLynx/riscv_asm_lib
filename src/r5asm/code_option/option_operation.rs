use std::fmt::Display;

use rust_macro_internal::ini_enum_str;

#[ini_enum_str("src/r5asm/code_option/option_operation.ini")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptionOperation { }

impl Default for OptionOperation {
    fn default() -> Self {
        OptionOperation::Pop
    }
}

impl Display for OptionOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}