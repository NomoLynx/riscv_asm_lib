use super::elf_section::NoteSection;
use super::linker_config::LinkerConfig;
use super::macro_instruction::{self, *};
use super::dynamic_structure::*;
use rust_macro::*;

pub (crate) type BuildTarget = u8;

#[derive(Debug, Clone, Accessors)]
pub struct CodeGenConfiguration {
    replace_pseudo_code : bool,
    generate_bin_and_code: bool,
    build_target : BuildTarget,
    linker_config : LinkerConfig,
    note_section : NoteSection,
    marco_instruction_archive : MacroInstructionHashMap,
    external_function_versions: ExternalFunctionVersions,
}

impl CodeGenConfiguration {
    pub fn new(replace_pseudo_code:bool) -> Self {
        let mut r = Self::default();
        r.set_replace_pseudo_code(replace_pseudo_code);
        r
    }

    /// set generate bin and code to false
    pub fn reset_generate_bin_and_code(&mut self) {
        self.set_generate_bin_and_code(false);
    }

    /// set replace pseudo code to true
    pub fn reset_replace_pseudo_code(&mut self) {
        self.set_replace_pseudo_code(true);
    }

    pub fn get_marco_instruction_archive_mut(&mut self) -> &mut MacroInstructionHashMap {
        &mut self.marco_instruction_archive
    }

    pub fn get_linker_config_mut(&mut self) -> &mut LinkerConfig {
        &mut self.linker_config
    }

}

impl Default for CodeGenConfiguration {
    /// create a new CodeGenConfiguration with default values
    /// replace_pseudo_code is true
    /// generate_bin_and_code is false
    fn default() -> Self {
        let macro_instrution_md_content = include_str!("macro_instruction/macro_instruction.md");
        let marco_instruction_archive = macro_instruction::parse_macro_instructions(macro_instrution_md_content).unwrap_or_default();
        let mut r = Self { 
            replace_pseudo_code: true, 
            generate_bin_and_code : false, 
            build_target : 8, 
            note_section : NoteSection::new_tspt(),
            linker_config : LinkerConfig::default(),
            marco_instruction_archive,
            external_function_versions: ExternalFunctionVersions::default(),
        };

        r.reset_replace_pseudo_code();
        r.reset_generate_bin_and_code();
        r
    }
}