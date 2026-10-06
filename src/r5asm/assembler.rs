use std::path::{Path, PathBuf};

use core_utils::file_object::FileObject;
use core_utils::string::string_to_bool;
use core_utils::number::get_u64_from_str;
use core_utils::filesystem::{folder_exists, get_file_containing_folder, get_file_name_without_extension, get_files_in_folder, path_file_exists, read_file_content, read_file_to_string};
use parser_lib::mermaid_type::MermaidType;
use pest::Parser;

use core_utils::debug::*;
use parser_lib::ini::{get_ini_properties, parse_ini_from_file};
use parser_lib::markdown_lang::*;
use pest::error::Error;
use pest::error::LineColLocation;
use crate::r5asm::foreign_data::*;
use crate::r5asm::state_machine_code::from_state1_to_asm;

use super::asm_error::AsmError;
use super::build_snippet_parameters::BuildSnippetParameters;
use super::register::Register;
use super::elf_section::*;
use super::asm_program::*;
use parser_lib::common::ParsingError;
use super::{asm_solution::ASMSolution, code_gen_config::CodeGenConfiguration, r5asm_pest::{R5AsmParser, Rule}};

fn pest_error_to_line<R: pest::RuleType>(err: &Error<R>) -> usize {
    match err.line_col {
        LineColLocation::Pos((line, _column)) => line,
        LineColLocation::Span((line, _column), _) => line,
    }
}

/// parse asm input string with default configuration, mainly for 
/// source code simulator, use 100 as base PC value for .text section
pub fn parse_asm_use_default_config(input:&str) -> Result<AsmProgram, AsmError> {
    let mut config = CodeGenConfiguration::default();
    let mut program = parse_asm(input, &mut config)?;

    // Same passes used by assembler build flow
    program.second_round(&mut config)?;
    program.third_round()?;

    // Optional: apply a base PC to .text so returned offsets are relocated
    program.update_section(SectionType::Text, 100 as usize);
    program.update_label_virtual_address(None)?;

    Ok(program)
}

pub fn parse_asm(input:&str, config:&mut CodeGenConfiguration) -> Result<AsmProgram, AsmError> {
    let mut pairs = R5AsmParser::parse(Rule::asm_prog, input).map_err(|e| {
        let line = pest_error_to_line(&e) as u32;
        let err_str = format!("Assembler Parsing error: {} @ {}", e, line);
        error_string(err_str.clone());
        AsmError::GeneralError((file!(), line).into(), format!("error: {err_str}"))
    })?;

    if let Some(pair) = pairs.find(|n| n.as_str().len() == input.len()) {
        let prog_r = AsmProgram::from_pair(&pair, config);
        if prog_r.is_err() {
            error_str("cannot get program from Rule::START");
            Err(AsmError::ParsingConversionError((file!(), line!()).into(), format!("cannot get program from Rule::START")) )
        }
        else { 
            let prog = prog_r.unwrap();
            Ok(prog)
        }
    }
    else {
        error_string(format!("Error: {} at {}", "does not catch all string", input.to_owned()));
        debug_string(format!("input: {}\r\nParsed: {:#?}", input, pairs));
        let count = pairs.count();
        debug_string(format!("Pairs count = {count}\r\n"));                
        Err(AsmError::ParsingConversionError((file!(), line!()).into(), format!("does not catch all string")) )
    }
}

const ASM_DATA_FILE_EXTENSION:&str = ".data.md";
const ASM_DATA_FOLDER_EXTENSION:&str = ".data";

const ASM_INI_FILE_EXTENSION:&str = ".ini";
const ASM_MERMAID_FILE_EXTENSION:&str = ".mermaid";
const ASM_MARKDOWN_FILE_EXTENSION:&str = ".md";
const ASM_MARKDOWN_FILE_EXTENSION2:&str = ".mkd";

pub (crate) fn read_data_md(file_path:&str, recalcuate_file_name:bool) -> Result<String, ParsingError> {
    let data_file = if recalcuate_file_name { get_related_data_file(file_path).ok_or(ParsingError::NoFound((file!().to_string(), line!()).into(), "data file not found".to_string()))? }
                                            else { file_path.to_string() };
    let md_file = load_md_file(&data_file)?;
    let tables = md_file.get_tables();
    
    let mut r = Vec::default();
    r.push(".data".to_string());
    for table in tables {
        let inc_strings = md_table_to_asm_data_section(table, &md_file)?;
        let incs = inc_strings.join("\r\n");
        r.push(incs);
    }

    let rr = r.iter().fold(String::default(), |acc, s| { format!("{acc}{s}") });
    Ok(rr)
}

fn get_related_data_file(file_path:&str) -> Option<String> {
    let file_name = get_file_name_without_extension(file_path);
    let folder = get_file_containing_folder(file_path);
    match (folder, file_name) {
        (Some(folder), Some(file)) => {
            let extension = ASM_DATA_FILE_EXTENSION;
            let full_path = std::path::Path::new(&folder).join(format!("{file}{extension}"));
            if full_path.exists() {
                full_path.as_os_str().to_str().map(|x| x.to_string())
            }
            else {
                None
            }
        }
        _ => None
    }
}

pub fn get_additional_file_and_folder(file_path:&str) -> Option<(String, String)> {
    let separator = std::path::MAIN_SEPARATOR;
    let file_name_without_extension_option = get_file_name_without_extension(file_path);
    let containing_folder_option = get_file_containing_folder(file_path);
    match (file_name_without_extension_option, containing_folder_option) {
        (Some(file_name_without_extension), Some(containing_folder)) => {
            let file = format!("{file_name_without_extension}{ASM_DATA_FILE_EXTENSION}");
            let folder = format!("{containing_folder}{separator}{file_name_without_extension}{ASM_DATA_FOLDER_EXTENSION}{separator}");
            Some((folder, file))
        }
        _ => None,
    }
}

fn parse_ini_u64(value: &str) -> u64 {
    let r = get_u64_from_str(value).unwrap_or(0);
    r
}

fn trim_ini_value(value: &str) -> String {
    value.trim().trim_matches('"').to_string()
}

fn resolve_ini_file_path(base_folder: &Path, raw_value: &str) -> String {
    let path_value = trim_ini_value(raw_value);
    let candidate = PathBuf::from(&path_value);
    if candidate.is_absolute() {
        path_value
    } else {
        base_folder.join(candidate).to_string_lossy().to_string()
    }
}

pub fn load_asm_solution_from_ini(ini_file_path: &str) -> Result<(ASMSolution, CodeGenConfiguration), AsmError> {
    let ini = parse_ini_from_file(ini_file_path)
        .map_err(|e| AsmError::GeneralError((file!(), line!()).into(), format!("failed to parse ini: {e:?}")))?;

    let properties = get_ini_properties(&ini);
    let ini_base_folder = Path::new(ini_file_path)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    let main_file_raw = properties
        .get("asm_main")
        .or_else(|| properties.get("main"))
        .or_else(|| properties.get("main_file"))
        .cloned()
        .unwrap_or_else(|| "main.s".to_string());
    let main_file_path = resolve_ini_file_path(&ini_base_folder, &main_file_raw);

    let output_folder_raw = properties
        .get("asm_output_folder")
        .or_else(|| properties.get("output_folder"))
        .cloned();

    let source_paths = properties
        .get("asm_source")
        .or_else(|| properties.get("source"))
        .or_else(|| properties.get("source_files"))
        .map(|value| {
            value.split(',')
                .map(str::trim)
                .filter(|part| !part.is_empty())
                .map(|part| resolve_ini_file_path(&ini_base_folder, part))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let main_content = std::fs::read_to_string(&main_file_path)
        .map_err(|e| AsmError::GeneralError((file!(), line!()).into(), format!("failed to read main asm file '{main_file_path}': {e}")))?;

    let main_stem = get_file_name_without_extension(&main_file_path)
        .unwrap_or("main".to_string())
        .to_string();

    let mut solution = ASMSolution::new(FileObject::new(&main_stem, "s", &main_content));

    let ini_base_folder_str = ini_base_folder.to_string_lossy().to_string();
    solution.set_container_path(&ini_base_folder_str);

    if let Some(output_folder) = output_folder_raw {
        solution.set_output_folder(&output_folder);
    }
    
    for item in source_paths {
        if item.is_empty() || item == main_file_path {
            continue;
        }

        let content = std::fs::read_to_string(&item)
            .map_err(|e| AsmError::GeneralError((file!(), line!()).into(), format!("failed to read source file '{item}': {e}")))?;

        let file_stem = get_file_name_without_extension(&item)
            .unwrap_or("source".to_string())
            .to_string();
        let ext = Path::new(&item)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("s")
            .to_string();

        solution.add_source_file(FileObject::new(&file_stem, &ext, &content));
    }

    let mut config = CodeGenConfiguration::default();

    if let Some(value) = properties.get("replace_pseudo_code").or_else(|| properties.get("codegen_replace_pseudo_code")) {
        config.set_replace_pseudo_code(string_to_bool(value));
    }

    if let Some(value) = properties.get("generate_bin_and_code").or_else(|| properties.get("codegen_generate_bin_and_code")) {
        config.set_generate_bin_and_code(string_to_bool(value));
    }

    if let Some(value) = properties.get("build_target").or_else(|| properties.get("codegen_build_target")) {
        config.set_build_target(value.trim().parse::<u8>().unwrap_or(8));
    }

    let virtual_address = properties
        .get("virtual_address_start")
        .or_else(|| properties.get("linker_virtual_address_start"))
        .or_else(|| properties.get("start_address"))
        .or_else(|| properties.get("linker_start_address"));

    if let Some(value) = virtual_address {
        config.get_linker_config_mut().set_virutual_address_start(parse_ini_u64(value));
    }

    if let Some(value) = properties.get("is_build_lib").or_else(|| properties.get("linker_is_build_lib")) {
        config.get_linker_config_mut().set_is_build_lib(string_to_bool(value));
    }

    if let Some(value) = properties.get("soname").or_else(|| properties.get("linker_soname")) {
        config.get_linker_config_mut().set_soname(Some(value.trim().to_string()));
    }

    Ok((solution, config))
}

/// build asm solution which contains one or more source files plus optional data files
pub fn build_asm_solution(asm_solution:&ASMSolution, config:&mut CodeGenConfiguration) -> Result<(), AsmError> {
    if asm_solution.has_multiple_source_files() {
        let input = asm_solution.get_combined_source();
        let mut ast = parse_asm(&input, config)?;
        ast.second_round(config)?;
        ast.third_round()?;
        ast.link_to_bin(&asm_solution.get_output_file_name(), config)?;
    }
    else {
        build_asm(&asm_solution.get_main_file_name(), &asm_solution.get_output_file_name(), config)?;
    }

    output_string(format!("built asm file: {}", &asm_solution.get_output_file_name()));
    Ok(())
}

/// build asm directly from an ini file. The ini drives both ASMSolution and CodeGenConfiguration.
pub fn build_asm_solution_from_ini(ini_file_path:&str) -> Result<(), AsmError> {
    let (solution, mut config) = load_asm_solution_from_ini(ini_file_path)?;
    build_asm_solution(&solution, &mut config)
}

/// build asm from ini and optionally copy resulting elf to a requested output file name.
pub fn build_asm_from_ini(ini_file_path:&str, output_file_name:&str) -> Result<(), AsmError> {
    let (solution, mut config) = load_asm_solution_from_ini(ini_file_path)?;
    build_asm_solution(&solution, &mut config)?;

    let generated_output = solution.get_output_file_name();
    if !output_file_name.is_empty() && generated_output != output_file_name {
        std::fs::copy(&generated_output, output_file_name).map_err(|e| {
            AsmError::GeneralError(
                (file!(), line!()).into(),
                format!(
                    "failed to copy generated output '{generated_output}' to '{output_file_name}': {e}"
                ),
            )
        })?;
    }

    Ok(())
}

/// build the asm file to get output file, file_path is the input file, output_file_name is the output file name
/// for general build, please use [`build_asm_solution`] instead, which will handle data file and data folder automatically
pub fn build_asm(file_path:&str, output_file_name:&str, config:&mut CodeGenConfiguration) -> Result<(), AsmError> {
    //check if has md data file or data folder
    let is_data_md_file_exist = if let Some((folder, file)) = get_additional_file_and_folder(file_path) {
            let containing_folder = get_file_containing_folder(file_path).unwrap();
            let data_file_exists = path_file_exists(&containing_folder, &file);
            let folder_exists = folder_exists(&folder);
            data_file_exists || folder_exists
        }
        else {
            false
        };

    let mut ast = if is_data_md_file_exist {
        let mut input = String::default();

        //merge the .md file
        let mut part0 = parse_asm(&read_file_to_string(file_path), config)?;

        if let Ok(data) = read_data_md(file_path, true) {
            input = format!("{}\r\n\r\n{}", read_file_to_string(file_path), data);
            
            let mut part1 = parse_asm(&data, config)?;
            part0.merge(&mut part1);
        }

        //merge files in the data folder
        let folder = get_additional_file_and_folder(file_path).unwrap().0;
        let files = get_files_in_folder(&folder, ASM_MARKDOWN_FILE_EXTENSION);
        for file in files.iter() {
            match read_data_md(file, false)
                    .map_err(|x| AsmError::GeneralError((file!(), line!()).into(), format!("cannot read data md file {file}: {x:?}"))) {
                Ok(data) => {
                    input = format!("{input}\r\n\r\n{}", data);

                    let mut part1 = parse_asm(&data, config)?;
                    part0.merge(&mut part1);
                }
                Err(ex) => {
                    error_string(format!("error: {ex:?}"));
                    return Err(ex)
                }
            }
        }

        //merge ini files in the data folder
        let folder = get_additional_file_and_folder(file_path).unwrap().0;
        let files = get_files_in_folder(&folder, ASM_INI_FILE_EXTENSION);
        for file in files {
            let data = parser_lib::ini::ini_file_to_asm_data_code(&file)
                .map_err(|_| AsmError::GeneralError((file!(), line!()).into(), format!("ini file to asm code wrong")))?;
            input = format!("{input}\r\n\r\n{}", data);
            let mut part1 = parse_asm(&data, config)?;
            part0.merge(&mut part1);
        }

        // merge mermaid diagram packet files as type template in the data folder
        let folder = get_additional_file_and_folder(file_path).unwrap().0;
        let files = get_files_in_folder(&folder, ASM_MERMAID_FILE_EXTENSION);
        for file in files {
            let file_content = read_file_content(&file)
                                        .map_err(|_| AsmError::IOError)?;
            let mermaid = MermaidType::get_mermaid_type_from_string_content(&file_content);
            match mermaid {
                Some(MermaidType::Packet(_)) => {
                    let data = generate_equ_statements_from_packet(&file)
                        .map_err(|_| AsmError::GeneralError((file!(), line!()).into(), format!("packet file to asm code wrong")))?;
                    let data_str = format!(".data\r\n{}", data.join("\r\n"));
                    input = format!("{input}\r\n\r\n{}", data_str);
                    let mut part1 = parse_asm(&data_str, config)?;
                    part0.merge(&mut part1);
                }
                Some(MermaidType::State(state)) => {
                    let data = from_state1_to_asm(&state)?;
                    debug_string(data);
                }
                _ => {}
            }
        }

        // merge mkd file in the data folder as type template
        let folder = get_additional_file_and_folder(file_path).unwrap().0;
        let files = get_files_in_folder(&folder, ASM_MARKDOWN_FILE_EXTENSION2);
        for file in files {
            let md_file = load_md_file(&file)
                                    .map_err(|x| AsmError::GeneralError((file!(), line!()).into(), format!("{x:?}")))?;
            let tables = md_file.get_tables();
            for table in tables {
                let data = get_equ_asm_statements_from_md_table(&table, &md_file, MDTableOffsetOrder::RowFirst)
                                                .map_err(|_| AsmError::GeneralError((file!(), line!()).into(), format!("mkd file to asm code wrong")))?;
                let data_str = format!(".data\r\n{}", data.join("\r\n"));
                input = format!("{input}\r\n\r\n{}", data_str);
                let mut part1 = parse_asm(&data_str, config)?;
                part0.merge(&mut part1);
            }
        }

        super::write_to_file("temp.s", &input)?;

        part0
    }
    else {
            let input = read_file_to_string(file_path);
            parse_asm(input.as_str(), config)?
    };

    ast.second_round(config)?;
    ast.third_round()?;
    ast.link_to_bin(output_file_name, config)
}

/// build asm snippet from input string, it will parse the input and generate binary code
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pest_error_to_line_reports_source_line_not_offset() {
        let err = R5AsmParser::parse(Rule::asm_prog, "\n\nfn efg {")
            .expect_err("input should fail to parse on line 3");

        assert_eq!(pest_error_to_line(&err), 3);
    }

    #[test]
    fn load_asm_solution_from_ini_sets_linker_config_and_sources() {
        let dir = std::env::temp_dir().join("r5asm_ini_test");
        let _ = std::fs::create_dir_all(&dir);

        let main_path = dir.join("main.s");
        let helper_path = dir.join("helper.s");
        let ini_path = dir.join("build.ini");

        std::fs::write(&main_path, ".text\naddi x1, x2, 1\n").unwrap();
        std::fs::write(&helper_path, ".text\naddi x3, x4, 2\n").unwrap();

        let ini = format!(
            "[asm]\nmain = main.s\nsource = helper.s\n\n[linker]\nvirtual_address_start = 0x81000000\n\n[codegen]\nreplace_pseudo_code = true\ngenerate_bin_and_code = false\nbuild_target = 8\n"
        );
        std::fs::write(&ini_path, ini).unwrap();

        let (solution, config) = load_asm_solution_from_ini(ini_path.to_str().unwrap()).unwrap();

        assert_eq!(solution.get_source_files().len(), 2);
        assert!(solution.get_combined_source().contains("addi x1, x2, 1"));
        assert!(solution.get_combined_source().contains("addi x3, x4, 2"));
        assert_eq!(config.get_linker_config().get_virutual_address_start(), 0x8100_0000);

        let _ = std::fs::remove_dir_all(&dir);
    }
}

pub fn build_asm_snippet(input:&str, parameters:&BuildSnippetParameters) -> Result<Vec<u8>, AsmError> {
    debug_str("Build asm snippet...");
    debug_string(format!("Parameters: {:?}", parameters));
    let mut config = CodeGenConfiguration::default();
    match parse_asm(input, &mut config) {
        Ok(mut ast) => {
            ast.second_round(&mut config)?;
            ast.third_round()?;

            let pc = parameters.get_pc().unwrap_or(0);
            ast.update_section(SectionType::Text, pc as usize);
            
            // add label to text section
            let mut txt_sections_mut = ast.get_txt_sections_mut();
            let text_section = txt_sections_mut.get_mut(0)
                                .ok_or(AsmError::CannotRetrieveValue((file!(), line!()).into()))?;
            for (label, offset) in parameters.get_u64_parameters() {
                text_section.append_label(offset as usize, &label);
            }
            
            ast.update_label_virtual_address(None)?;
            let regs = Register::new();
            let labels = ast.get_labels()?;

            // generate machine code for all txt sections
            let mut code_bin = Vec::default();         
            for section in ast.get_txt_sections() {
                for inc in section.get_instructions() {                
                    let machine_codes = ast.get_machine_code_list(inc, &regs, &labels)?;
                    let bin = machine_codes.iter().map(|x| x.to_vec()).flatten().collect::<Vec<_>>();
                    code_bin.extend(bin);
                }
            }
            
            Ok(code_bin)
        }
        Err(e) => {
            error_string(format!("asm snippet build error: {e:?}"));
            Err(e)
        }
    }
}
