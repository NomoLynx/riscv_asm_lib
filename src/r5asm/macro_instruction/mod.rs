use pest::Parser;
use pest_derive::Parser;

pub(crate) mod marco_instruction_pest;
pub(crate) mod macro_instructions;
pub(crate) mod macro_instruction_list;
pub(crate) mod macro_overload;

pub use marco_instruction_pest::*;
pub use macro_instruction_list::*;
pub use macro_instructions::*;
pub use macro_overload::*;

#[derive(Parser)]
#[grammar = "../riscv_asm_lib/src/r5asm/macro_instruction/marco_instruction.pest"]
pub (crate) struct MacroInstructionParser;

