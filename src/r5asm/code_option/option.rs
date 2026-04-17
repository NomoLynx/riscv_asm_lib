use rust_macro_internal::ini_enum_str;

#[ini_enum_str("src/r5asm/code_option/option.ini")]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CodeOption {
}