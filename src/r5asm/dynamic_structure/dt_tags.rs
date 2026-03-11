use rust_macro_internal::*;
use rust_macro::*;

#[ini_enum("src/r5asm/dynamic_structure/dt_tags.ini", repr = u64)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[derive(GenIsEnumVariant)]
#[allow(non_camel_case_types)]
pub enum DTTags { 
}