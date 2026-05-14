use rust_macro_internal::csv_struct;
use parser_lib::csv::*;

use crate::r5asm::instruction::Instruction;

#[csv_struct("src/r5asm/meta/instruction_register.csv")]
pub struct InstructionRegisterMeta {

}

impl InstructionRegisterMeta {
    pub fn get_changed_registers(inc:&Instruction) -> Vec<String> {
        let inc_name = inc.get_name();

        let db = Self::to_typed_vec();
        let inc_record = db.iter()
                .find(|record| record.get_instruction_name().to_lowercase() == inc_name.to_lowercase());
        if let Some(record) = inc_record {
            let mut changed_regs = Vec::default();
            
            let r0_changed = record.get_register0_changed();
            let r1_changed = record.get_register1_changed();
            let r2_changed = record.get_register2_changed();
            let r3_changed = record.get_register3_changed().map(|x| *x).unwrap_or(false);

            if r0_changed { changed_regs.push(inc.get_r0().unwrap().to_string()); }
            if r1_changed { changed_regs.push(inc.get_r1().unwrap().to_string()); }
            if r2_changed { changed_regs.push(inc.get_r2().unwrap().to_string()); }
            if r3_changed { changed_regs.push(inc.get_r3().unwrap().to_string()); }

            changed_regs
        }
        else {
            Vec::default()
        }
    } 
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::r5asm::code_gen_config::CodeGenConfiguration;

    #[test]
    fn test_get_changed_registers_with_add_instruction() {
        let mut config = CodeGenConfiguration::default();
        let instructions = Instruction::from_string("add x1, x2, x3", &mut config)
            .expect("add instruction should parse");
        let inc = instructions
            .first()
            .expect("one parsed add instruction should be returned");

        let changed = InstructionRegisterMeta::get_changed_registers(inc);

        assert_eq!(changed, vec!["x1".to_string()]);
    }
}

