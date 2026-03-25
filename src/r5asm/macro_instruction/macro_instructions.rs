use std::collections::HashMap;

use super::super::instruction::Instruction;
use super::super::asm_error::*;
use super::*;
use core_utils::debug::*;
use parser_lib::common::ParsingError as ParserLibError;
use parser_lib::markdown_lang::load_md_file_from_str;

fn md_table_object_to_hash(table:&parser_lib::markdown_lang::Table) -> Result<HashMap<String, String>, ParserLibError> {
    let mut output = HashMap::default();
    let col_number = table.get_col_names()?.len();
    if col_number != 2 {
        return Err(ParserLibError::NoFound(
            (file!().to_string(), line!()).into(),
            format!("table has {col_number} columns, but expected 2"),
        ));
    }

    let rows = table.data_rows();
    for row in rows.iter() {
        let key = &row[0];
        let value = &row[1];
        let key_value = key.get_text().trim().to_string();
        let value_text = value.get_text().trim().to_string();
        if !value_text.is_empty() {
            output.insert(key_value, value_text);
        } else {
            let inlines = value.get_inline_items();
            if let Some(link_text) = inlines.first().and_then(|x| x.get_link().map(|l| l.get_link_text())) {
                output.insert(key_value, link_text.trim().to_string());
            } else {
                output.insert(key_value, String::new());
            }
        }
    }

    Ok(output)
}

#[derive(Debug, Clone)]
pub struct MacroInstructionHashMap(pub HashMap<String, MacroInstructionList>);

impl MacroInstructionHashMap {
    pub fn get(&self, key: &str) -> Option<&MacroInstructionList> {
        self.0.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut MacroInstructionList> {
        self.0.get_mut(key)
    }

    pub fn get_incs(&self, key: &str, parameters: &Vec<String>) -> Result<Vec<Instruction>, AsmError> {
        if let Some(macro_instruction_list) = self.get(key) {
            let mut mil = macro_instruction_list.clone();
            mil.replace_parameters(parameters);
            let r = mil.to_instructions()?;
            Ok(r)
        } else {
            Err(AsmError::NoFound((file!(), line!()).into(), format!("Macro instruction with key '{}' not found", key)))
        }
    }
}

impl From<HashMap<String, MacroInstructionList>> for MacroInstructionHashMap {
    fn from(value: HashMap<String, MacroInstructionList>) -> Self {
        MacroInstructionHashMap(value)
    }
}

impl Default for MacroInstructionHashMap {
    fn default() -> Self {
        MacroInstructionHashMap(HashMap::new())
    }
}

pub (crate) fn parse(input:&str) -> Result<MacroInstructionList, ParserLibError> {
    let mut parsed = MacroInstructionParser::parse(Rule::macro_snippet, input)
        .map_err(|e| ParserLibError::NoFound((file!().to_string(), line!()).into(), format!("Failed to parse macro instructions: {}", e)))?;

    let marco_instruction_pair = parsed.next().unwrap();

    let mut r = Vec::new();

    for macro_instruction_pair in marco_instruction_pair.into_inner().into_iter() {
        match macro_instruction_pair.as_rule() {
            Rule::marco_instruction => {
                let macro_instruction = MacroInstruction::from_pair(&macro_instruction_pair)?;
                r.push(macro_instruction);
            },
            Rule::EOI => {},
            _ => {
                let str = format!("unexpected rule {:?} in macro instruction parsing", macro_instruction_pair.as_rule());
                error_string(str.to_string());
                return Err(ParserLibError::NoFound((file!().to_string(), line!()).into(), str));
            }
        }
    }

    Ok(r.into())
}

pub (crate) fn parse_macro_instructions(file_content: &str) -> Result<MacroInstructionHashMap, ParserLibError> {
    let md = load_md_file_from_str(file_content)?;
    
    let tables = md.get_tables();
    if tables.is_empty() {
        return Ok(HashMap::default().into());
    }

    let hash = md_table_object_to_hash(tables.first().unwrap())?;
    
    let mut r = HashMap::new();
    for ((key, header_text), _index) in hash.iter().zip(0..) {
        let header_level = 2;
        let headers = md.get_headers_with_level(header_level);
        if let Some(_header) = headers.iter().find(|x| & x.get_text() == header_text) {
            if let Some(code) = md.find_code_after_header(header_text) {
                let code_text = code.get_code();
                match parse(&code_text) {
                    Ok(asm_code) => {
                        r.insert(key.to_string(), asm_code);
                    }
                    Err(e) => {
                        // err output cannot parse code block after header
                        error_string(format!("cannot parse code block '{header_text}' with key = '{key}'"));
                        error_string(format!("{e:?}"));
                        Err(ParserLibError::NoFound((file!().to_string(), line!()).into(), format!("cannot parse code block with header: {}", header_text)))?;
                    }
                }
            }
            else {
                // errr output cannot find code block after header
                error_string(format!("cannot find code block after header: {}", header_text));
                Err(ParserLibError::NoFound((file!().to_string(), line!()).into(), format!("cannot find code block after header: {}", header_text)))?;
            }
        }
        else {
            //err output cannot find header
            error_string(format!("cannot find header: {}", header_text));
            Err(ParserLibError::NoFound((file!().to_string(), line!()).into(), format!("cannot find header: {}", header_text)))?;
        }
    }

    Ok(r.into())
}