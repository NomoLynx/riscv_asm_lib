
use core_utils::filesystem::{folder_exists, get_file_containing_folder, get_file_name_without_extension, get_files_in_folder, path_file_exists, read_file_to_string};
use pest::Parser;

use core_utils::debug::*;
use parser_lib::markdown_lang::*;
use super::asm_error::AsmError;
use super::build_snippet_parameters::BuildSnippetParameters;
use super::register::Register;
use super::elf_section::*;
use super::asm_program::*;
use parser_lib::common::ParsingError;
use super::{asm_solution::ASMSolution, code_gen_config::CodeGenConfiguration, r5asm_pest::{R5AsmParser, Rule}};

/// parse asm input string with default configuration
pub fn parse_asm_use_default_config(input:&str) -> Result<AsmProgram, AsmError> {
    let mut config = CodeGenConfiguration::default();
    parse_asm(input, &mut config)
}

pub fn parse_asm(input:&str, config:&mut CodeGenConfiguration) -> Result<AsmProgram, AsmError> {
    let mut pairs = R5AsmParser::parse(Rule::asm_prog, input).map_err(|e| {
        let err_str = format!("Assembler Parsing error: {} @ {}", e, e.line());
        error_string(err_str.clone());
        AsmError::GeneralError((file!(), line!()).into(), format!("error: {err_str} @ {:?}", &e.line_col))
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

pub (crate) fn read_data_md(file_path:&str, recalcuate_file_name:bool) -> Result<String, ParsingError> {
    let data_file = if recalcuate_file_name { get_related_data_file(file_path).ok_or(ParsingError::NoFound((file!().to_string(), line!()).into(), "data file not found".to_string()))? }
                                            else { file_path.to_string() };
    let md_file = load_md_file(&data_file)?;
    let tables = md_file.get_tables();
    
    let mut r = Vec::default();
    r.push(".data".to_string());
    for table in tables {
        let inc_strings = super::md_data::md_table_to_asm_data_section(table, &md_file)?;
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

/// build asm solution which contains input, output file name, and data file
pub fn build_asm_solution(asm_solution:&ASMSolution, config:&mut CodeGenConfiguration) -> Result<(), AsmError> {
    build_asm(&asm_solution.get_main_file_name(), &asm_solution.get_output_file_name(), config)
}

/// build the asm file to get output file, file_path is the input file, output_file_name is the output file name
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
        let files = get_files_in_folder(&folder, ".md");
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
        let files = get_files_in_folder(&folder, ".ini");
        for file in files {
            let data = parser_lib::ini::ini_file_to_asm_data_code(&file)
                .map_err(|_| AsmError::GeneralError((file!(), line!()).into(), format!("ini file to asm code wrong")))?;
            input = format!("{input}\r\n\r\n{}", data);
            let mut part1 = parse_asm(&data, config)?;
            part0.merge(&mut part1);
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
            let labels = ast.get_labels();

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
