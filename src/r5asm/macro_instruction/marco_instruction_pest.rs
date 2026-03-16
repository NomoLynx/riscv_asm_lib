use std::fmt::Display;
use std::ops::{Index, IndexMut};

use pest::iterators::Pair;

use super::super::asm_error::AsmError;
use core_utils::debug::*;
use parser_lib::common::ParsingError as ParserLibError;
use super::super::{instruction::Instruction, macro_instruction::Rule};

#[derive(Debug, Clone)]
pub struct MacroInstruction {
    inc: String,
    items: Vec<String>,
}

impl MacroInstruction {
    pub fn from_pair(pair: &Pair<Rule>) -> Result<Self, ParserLibError> {
        assert_eq!(pair.as_rule(), Rule::marco_instruction); //Ensure the parsed rule
        let inner = pair.to_owned().into_inner().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
        match inner.as_slice() {
            [(Rule::inc, p)] => Ok(Self::new(p.as_str().to_string(), Vec::default())),
            [(Rule::inc, p), rest@..] => {
                let inc = p.as_str().to_string();
                let items = rest.iter().map(|(_, pair)| pair.as_str().to_string()).collect::<Vec<_>>();
                Ok(Self::new(inc, items))
            }
            _ => {
                let str = format!("cannot find {inner:?} in jump statement processing logic");
                error_string(str.to_string());
                Err(ParserLibError::NoFound((file!().to_string(), line!()).into(), str))
            }
        }
    }

    pub fn new(inc: String, items: Vec<String>) -> Self {
        Self { inc, items }
    }

    pub fn get_inc(&self) -> &str {
        &self.inc
    }

    /// get items
    pub fn get_items(&self) -> &Vec<String> {
        &self.items
    }

    /// check if items has something like $1, $2, etc.
    pub fn has_parameters(&self) -> bool {
        for item in &self.items {
            if item.starts_with('$') {
                return true;
            }
        }
        false
    }

    /// get parameter highest number in items
    pub fn get_highest_parameter_number(&self) -> usize {
        let mut highest = 0;
        for item in &self.items {
            if item.starts_with('$') {
                if let Ok(num) = item[1..].parse::<usize>() {
                    if num > highest {
                        highest = num;
                    }
                }
            }
        }
        highest
    }

    /// replace parameters with given values, the index is 1-based
    pub fn replace_parameters(&mut self, values: &Vec<String>) {
        if self.has_parameters() == false {
            return;
        }

        for item in &mut self.items {
            if item.starts_with('$') {
                if let Ok(num) = item[1..].parse::<usize>() {
                    if num >= 1 && num <= values.len() {
                        *item = values[num - 1].clone();
                    }
                }
            }
        }
    }

    pub fn to_instructions(&self) -> Result<Vec<Instruction>, AsmError> {
        let inc_str = format!("{self}");
        let mut asm_config = super::super::code_gen_config::CodeGenConfiguration::default();
        asm_config.reset_generate_bin_and_code();
        asm_config.set_replace_pseudo_code(true);

        if let Ok(inc) = super::super::instruction::Instruction::from_string(&inc_str, &mut asm_config) {
            Ok(inc)
        }
        else {
            Err(AsmError::GeneralError((file!(), line!()).into(), format!("cannot convert internal macro instruction to basic instruction: {self}")))
        }
    }
}

// Allow indexing: macro_inst[0] -> &String (can be used as &str)
impl Index<usize> for MacroInstruction {
    type Output = String;
    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

// Allow mutable indexing: &mut macro_inst[0]
impl IndexMut<usize> for MacroInstruction {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.items[index]
    }
}

impl Display for MacroInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inc = self.get_inc();
        let items_str = self.get_items().join(", ");
        write!(f, "{inc} {items_str}")
    }
}

impl From<MacroInstruction> for Instruction {
    fn from(macro_inst: MacroInstruction) -> Self {
        (&macro_inst).into()
    }
}

impl From<&MacroInstruction> for Instruction {
    fn from(macro_inst: &MacroInstruction) -> Self {
        macro_inst.to_instructions()
            .and_then(|mut v| {
            if v.len() == 1 {
                Ok(v.remove(0))
            } else {
                panic!("{}", format!("internal macro instruction should be converted to a single basic instruction, but got {}", v.len()))
            }
        })
        .unwrap()
    }
}