use super::super::instruction::Instruction;
use super::super::asm_error::*;
use super::*;

#[derive(Debug, Clone)]
pub struct MacroInstructionList(Vec<MacroInstruction>);

impl MacroInstructionList {
    /// get internal vector
    pub fn get_list(&self) -> &Vec<MacroInstruction> {
        &self.0
    }

    /// get highest parameter number in all macro instructions
    pub fn get_highest_parameter_number(&self) -> usize {
        let mut highest = 0;
        for mi in &self.0 {
            let mi_highest = mi.get_highest_parameter_number();
            if mi_highest > highest {
                highest = mi_highest;
            }
        }
        highest
    }

    /// replace parameters in all macro instructions
    pub fn replace_parameters(&mut self, parameters: &Vec<String>) {
        // parameter number must equal to highest parameter number
        assert_eq!(parameters.len(), self.get_highest_parameter_number(), "parameter length (left) does not match highest parameter number (right)");
        
        for mi in &mut self.0 {
            mi.replace_parameters(parameters);
        }
    }

    pub fn to_instructions(&self) -> Result<Vec<Instruction>, AsmError> {
        let mut r = Vec::new();
        for mi in &self.0 {
            let mut incs = mi.to_instructions()?;
            r.append(&mut incs);
        }
        Ok(r)
    }
}

impl From<Vec<MacroInstruction>> for MacroInstructionList {
    fn from(v: Vec<MacroInstruction>) -> Self {
        MacroInstructionList(v)
    }
}