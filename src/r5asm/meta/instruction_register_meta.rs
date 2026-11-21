use rust_macro_internal::csv_struct;
use parser_lib::csv::*;

use crate::r5asm::instruction::Instruction;
use core_utils::indexed_store::IndexedStore;

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
            let r2_changed = record.get_register2_changed().map(|x| *x).unwrap_or(false);
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

    pub fn get_inc_extension(inc:&Instruction) -> Vec<String> {
        let inc_name = inc.get_name();

        let db = Self::to_indexed_typed_vec();
        let inc_record = db.get_by_key(&inc_name.to_lowercase());
        if let Some(record) = inc_record {
            let extension = record.get_extension();
            
            extension.split("|")
                    .map(|s| s.to_string())
                    .collect()
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

    #[test]
    fn test_half_precision_arithmetic_uses_zfh_extension() {
        let mut config = CodeGenConfiguration::default();

        for code in [
            "fadd.h fa0, fa1, fa2",
            "fsub.h fa0, fa1, fa2",
            "fmul.h fa0, fa1, fa2",
            "fdiv.h fa0, fa1, fa2",
            "fsqrt.h fa0, fa1",
            "fmadd.h fa0, fa1, fa2, fa3",
            "fmsub.h fa0, fa1, fa2, fa3",
            "fnmsub.h fa0, fa1, fa2, fa3",
            "fnmadd.h fa0, fa1, fa2, fa3",
            "fsgnj.h fa0, fa1, fa2",
            "fsgnjn.h fa0, fa1, fa2",
            "fsgnjx.h fa0, fa1, fa2",
            "fmin.h fa0, fa1, fa2",
            "fmax.h fa0, fa1, fa2",
            "feq.h a0, fa1, fa2",
            "flt.h a0, fa1, fa2",
            "fle.h a0, fa1, fa2",
            "fclass.h a0, fa1",
            "fcvt.s.h fa0, fa1",
            "fcvt.h.s fa0, fa1",
            "fcvt.d.h fa0, fa1",
            "fcvt.h.d fa0, fa1",
            "fcvt.q.h fa0, fa1",
            "fcvt.h.q fa0, fa1",
            "fcvt.h.w fa0, a1",
            "fcvt.h.wu fa0, a1",
            "fcvt.h.l fa0, a1",
            "fcvt.h.lu fa0, a1",
            "fcvt.w.h a0, fa1",
            "fcvt.wu.h a0, fa1",
            "fcvt.l.h a0, fa1",
            "fcvt.lu.h a0, fa1",
            "flh fa0, 0(a0)",
            "fsh fa0, 0(a0)",
            "fmv.x.h a0, fa1",
            "fmv.h.x fa0, a1",
        ] {
            let instructions = Instruction::from_string(code, &mut config)
                .expect("half-precision instruction should parse");
            let inc = instructions
                .first()
                .expect("one parsed half-precision instruction should be returned");

            assert_eq!(
                InstructionRegisterMeta::get_inc_extension(inc),
                vec!["zfh".to_string()],
                "unexpected extension for `{code}`"
            );
        }
    }

    #[test]
    fn test_half_precision_conversion_selectors() {
        let cases = [
            ("fcvt.s.h fa0, fa1", 2),
            ("fcvt.h.s fa0, fa1", 0),
            ("fcvt.d.h fa0, fa1", 2),
            ("fcvt.h.d fa0, fa1", 1),
            ("fcvt.q.h fa0, fa1", 2),
            ("fcvt.h.q fa0, fa1", 3),
            ("fcvt.h.w fa0, a1", 0),
            ("fcvt.h.wu fa0, a1", 1),
            ("fcvt.h.l fa0, a1", 2),
            ("fcvt.h.lu fa0, a1", 3),
            ("fcvt.w.h a0, fa1", 0),
            ("fcvt.wu.h a0, fa1", 1),
            ("fcvt.l.h a0, fa1", 2),
            ("fcvt.lu.h a0, fa1", 3),
        ];
        let mut config = CodeGenConfiguration::default();

        for (code, expected) in cases {
            let instructions = Instruction::from_string(code, &mut config)
                .expect("half-precision conversion should parse");
            let inc = instructions.first().expect("one parsed conversion expected");

            assert_eq!(
                inc.get_alternative_r2_value(),
                Some(expected),
                "unexpected rs2 selector for `{code}`"
            );
        }
    }
}

