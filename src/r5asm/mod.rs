use std::{fs::File, io::Write};

use self::asm_error::AsmError;

pub mod r5asm_pest;
pub mod opcode;
pub mod instruction;
mod machinecode;
mod basic_instruction_extensions;
mod register;
pub mod directive;
mod compact_inc;
pub mod code_gen_config;
pub mod asm_solution;
pub mod assembler;
pub mod elf_file;
pub mod traits;
pub mod label_offset;
pub mod imm_macro;
pub mod imm;
pub mod section;
pub mod dynamic_structure;
pub mod build_snippet_parameters;
pub mod external_label;
pub mod alignment;
pub mod elf_section;
pub mod linker_config;
pub mod macro_instruction;
pub mod asm_error;
pub mod asm_program;
pub mod r5inc;
pub mod md_data;

pub static mut OPTIMIZE_CODE_GEN : bool = false;
pub static mut OPTIMIZE_TO_COMPACT_CODE : bool = false;

pub type ExprValue = parser_lib::expr_lang::ExprValue;
pub type ExprError = parser_lib::common::ParsingError;


pub (self) fn reverse_string(input:&str) -> String {
    input.chars().rev().collect::<String>()
}

pub (self) fn write_to_file(path:&str, data:&str) -> Result<(), AsmError> {
    let mut file = match File::create(path) {
        Err(_why) => return Err(AsmError::IOError),
        Ok(file) => file,
    };

    match file.write_all(data.as_bytes()) {
        Err(_why) => Err(AsmError::IOError),
        Ok(_) => Ok(()),
    }
}

pub (crate) fn round_to_usize(v:usize, round_to:usize) -> usize {
    if v % round_to == 0 {
        v
    }
    else {
        ((v/round_to) + 1) * round_to
    }
}

/// calculate padding based on current address and alignment power
pub fn calculate_padding(current_address: u64, alignment_power: u32) -> u64 {
    let alignment = 1 << alignment_power; // 2^n
    let mask = alignment - 1;
    let r = if current_address & mask == 0 {
        0
    } else {
        alignment - (current_address & mask)
    };

    r
}

#[cfg(test)]
pub (self) mod tests {
    use super::*;

    #[test]
    fn test_reverse_string() {
        assert_eq!(reverse_string("hello"), "olleh");
        assert_eq!(reverse_string("world"), "dlrow");
        assert_eq!(reverse_string(""), "");
    }

    #[test]
    fn test_calculate_padding() {
        assert_eq!(calculate_padding(0, 2), 0);
        assert_eq!(calculate_padding(1, 2), 3);
        assert_eq!(calculate_padding(2, 2), 2);
        assert_eq!(calculate_padding(3, 2), 1);
        assert_eq!(calculate_padding(0x201c, 3), 4);
        assert_eq!(calculate_padding(0x203c, 3), 4);
    }
}