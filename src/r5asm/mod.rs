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
pub mod vector_incs;
pub mod code_option;

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
    use std::convert::TryInto;

    use pest::Parser;

    use super::*;

    fn decode_u32_words(bytes: &[u8]) -> Vec<u32> {
        bytes
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
            .collect()
    }

    fn set_test_cwd_for_r5asm_data() {
        let dir = format!("{}/src/r5asm", env!("CARGO_MANIFEST_DIR"));
        let _ = std::env::set_current_dir(&dir);
    }

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

    #[test]
    fn test_zba_instruction_parser_acceptance() {
        let cases = [
            "add.uw x1, x2, x3",
            "sh1add x1, x2, x3",
            "sh2add x1, x2, x3",
            "sh3add x1, x2, x3",
            "slli.uw x1, x2, 3",
            "sh1add.uw x1, x2, x3",
            "sh2add.uw x1, x2, x3",
            "sh3add.uw x1, x2, x3",
        ];

        for case in cases {
            let parsed = r5asm_pest::R5AsmParser::parse(r5asm_pest::Rule::instruction, case);
            assert!(parsed.is_ok(), "failed to parse zba instruction: {case}");
        }
    }

    #[test]
    fn test_zba_instruction_encoding() {
        set_test_cwd_for_r5asm_data();

        let input = ".text\n\
add.uw x1, x2, x3\n\
sh1add x1, x2, x3\n\
sh2add x1, x2, x3\n\
sh3add x1, x2, x3\n\
slli.uw x1, x2, 3\n\
sh1add.uw x1, x2, x3\n\
sh2add.uw x1, x2, x3\n\
sh3add.uw x1, x2, x3\n";

        let params = build_snippet_parameters::BuildSnippetParameters::default();
        let bytes = assembler::build_asm_snippet(input, &params).expect("zba snippet should build");
        let words = decode_u32_words(&bytes);

        let expected = vec![
            0x0831_00BB, // add.uw x1, x2, x3
            0x2031_20B3, // sh1add x1, x2, x3
            0x2031_40B3, // sh2add x1, x2, x3
            0x2031_60B3, // sh3add x1, x2, x3
            0x0831_109B, // slli.uw x1, x2, 3
            0x2031_20BB, // sh1add.uw x1, x2, x3
            0x2031_40BB, // sh2add.uw x1, x2, x3
            0x2031_60BB, // sh3add.uw x1, x2, x3
        ];

        assert_eq!(words, expected);
    }

    #[test]
    fn test_zbs_instruction_parser_acceptance() {
        let cases = [
            "bclr x1, x2, x3",
            "bclri x1, x2, 3",
            "bext x1, x2, x3",
            "bexti x1, x2, 3",
            "binv x1, x2, x3",
            "binvi x1, x2, 3",
            "bset x1, x2, x3",
            "bseti x1, x2, 3",
        ];

        for case in cases {
            let parsed = r5asm_pest::R5AsmParser::parse(r5asm_pest::Rule::instruction, case);
            assert!(parsed.is_ok(), "failed to parse zbs instruction: {case}");
        }
    }

    #[test]
    fn test_zbs_instruction_encoding() {
        set_test_cwd_for_r5asm_data();

        let input = ".text\n\
bclr x1, x2, x3\n\
bclri x1, x2, 3\n\
bext x1, x2, x3\n\
bexti x1, x2, 3\n\
binv x1, x2, x3\n\
binvi x1, x2, 3\n\
bset x1, x2, x3\n\
bseti x1, x2, 3\n";

        let params = build_snippet_parameters::BuildSnippetParameters::default();
        let bytes = assembler::build_asm_snippet(input, &params).expect("zbs snippet should build");
        let words = decode_u32_words(&bytes);

        let expected = vec![
            0x4831_10B3, // bclr x1, x2, x3
            0x4831_1093, // bclri x1, x2, 3
            0x4831_50B3, // bext x1, x2, x3
            0x4831_5093, // bexti x1, x2, 3
            0x6831_10B3, // binv x1, x2, x3
            0x6831_1093, // binvi x1, x2, 3
            0x2831_10B3, // bset x1, x2, x3
            0x2831_1093, // bseti x1, x2, 3
        ];

        assert_eq!(words, expected);
    }

    #[test]
    fn test_zbb_instruction_parser_acceptance_core() {
        let cases = [
            "andn x1, x2, x3",
            "orn x1, x2, x3",
            "xnor x1, x2, x3",
            "rol x1, x2, x3",
            "ror x1, x2, x3",
            "rori x1, x2, 3",
            "clz x1, x2",
            "ctz x1, x2",
            "cpop x1, x2",
            "sext.b x1, x2",
            "sext.h x1, x2",
            "orc.b x1, x2",
            "rev8 x1, x2",
            "zext.h x1, x2",
            "min x1, x2, x3",
            "minu x1, x2, x3",
            "max x1, x2, x3",
            "maxu x1, x2, x3",
        ];

        for case in cases {
            let parsed = r5asm_pest::R5AsmParser::parse(r5asm_pest::Rule::instruction, case);
            assert!(parsed.is_ok(), "failed to parse zbb core instruction: {case}");
        }
    }

    #[test]
    fn test_zbb_instruction_encoding_core() {
        set_test_cwd_for_r5asm_data();

        let input = ".text\n\
andn x1, x2, x3\n\
orn x1, x2, x3\n\
xnor x1, x2, x3\n\
rol x1, x2, x3\n\
ror x1, x2, x3\n\
rori x1, x2, 3\n\
clz x1, x2\n\
ctz x1, x2\n\
cpop x1, x2\n\
sext.b x1, x2\n\
sext.h x1, x2\n\
orc.b x1, x2\n\
rev8 x1, x2\n\
zext.h x1, x2\n\
min x1, x2, x3\n\
minu x1, x2, x3\n\
max x1, x2, x3\n\
maxu x1, x2, x3\n";

        let params = build_snippet_parameters::BuildSnippetParameters::default();
        let bytes = assembler::build_asm_snippet(input, &params).expect("zbb core snippet should build");
        let words = decode_u32_words(&bytes);

        let expected = vec![
            0x4031_70B3, // andn x1, x2, x3
            0x4031_60B3, // orn x1, x2, x3
            0x4031_40B3, // xnor x1, x2, x3
            0x6031_10B3, // rol x1, x2, x3
            0x6031_50B3, // ror x1, x2, x3
            0x6031_5093, // rori x1, x2, 3
            0x6001_1093, // clz x1, x2
            0x6011_1093, // ctz x1, x2
            0x6021_1093, // cpop x1, x2
            0x6041_1093, // sext.b x1, x2
            0x6051_1093, // sext.h x1, x2
            0x2871_5093, // orc.b x1, x2
            0x6B81_5093, // rev8 x1, x2
            0x0801_40BB, // zext.h x1, x2 (RV64 alias to packw rd, rs1, x0)
            0x0A31_40B3, // min x1, x2, x3
            0x0A31_50B3, // minu x1, x2, x3
            0x0A31_60B3, // max x1, x2, x3
            0x0A31_70B3, // maxu x1, x2, x3
        ];

        assert_eq!(words, expected);
    }

    #[test]
    fn test_zbb_instruction_parser_acceptance_w() {
        let cases = [
            "rolw x1, x2, x3",
            "rorw x1, x2, x3",
            "roriw x1, x2, 3",
            "clzw x1, x2",
            "ctzw x1, x2",
            "cpopw x1, x2",
        ];

        for case in cases {
            let parsed = r5asm_pest::R5AsmParser::parse(r5asm_pest::Rule::instruction, case);
            assert!(parsed.is_ok(), "failed to parse zbb word instruction: {case}");
        }
    }

    #[test]
    fn test_zbb_instruction_encoding_w() {
        set_test_cwd_for_r5asm_data();

        let input = ".text\n\
rolw x1, x2, x3\n\
rorw x1, x2, x3\n\
roriw x1, x2, 3\n\
clzw x1, x2\n\
ctzw x1, x2\n\
cpopw x1, x2\n";

        let params = build_snippet_parameters::BuildSnippetParameters::default();
        let bytes = assembler::build_asm_snippet(input, &params).expect("zbb word snippet should build");
        let words = decode_u32_words(&bytes);

        let expected = vec![
            0x6031_10BB, // rolw x1, x2, x3
            0x6031_50BB, // rorw x1, x2, x3
            0x6031_509B, // roriw x1, x2, 3
            0x6001_109B, // clzw x1, x2
            0x6011_109B, // ctzw x1, x2
            0x6021_109B, // cpopw x1, x2
        ];

        assert_eq!(words, expected);
    }

    #[test]
    fn test_shift_right_arithmetic_immediate_encoding() {
        set_test_cwd_for_r5asm_data();

        let input = ".text\n\
srai x1, x2, 3\n\
sraiw x1, x2, 3\n";

        let params = build_snippet_parameters::BuildSnippetParameters::default();
        let bytes = assembler::build_asm_snippet(input, &params).expect("srai snippet should build");
        let words = decode_u32_words(&bytes);

        let expected = vec![
            0x4031_5093, // srai x1, x2, 3
            0x4031_509B, // sraiw x1, x2, 3
        ];

        assert_eq!(words, expected);
    }

    #[test]
    fn test_rvv_vset_instruction_parser_acceptance() {
        let cases = [
            "vsetvli x5, x6, e32, m1, ta, ma",
            "vsetivli x7, 9, e16, mf2, tu, mu",
            "vsetvl x8, x9, x10",
        ];

        for case in cases {
            let parsed = r5asm_pest::R5AsmParser::parse(r5asm_pest::Rule::instruction, case);
            assert!(parsed.is_ok(), "failed to parse RVV vset instruction: {case}");
        }
    }

    #[test]
    fn test_rvv_unit_stride_load_store_acceptance() {
        let cases = [
            "vle32.v v1, (x2)",
            "vse32.v v1, (x2)",
        ];

        for case in cases {
            let parsed = r5asm_pest::R5AsmParser::parse(r5asm_pest::Rule::instruction, case);
            assert!(parsed.is_ok(), "failed to parse RVV unit-stride load/store: {case}");
        }
    }

    #[test]
    fn test_rvv_mask_and_reduction_parser_acceptance() {
        let cases = [
            "vand.mm v1, v2, v3",
            "vxor.mm v4, v5, v6",
            "vnot.m v7, v8",
            "vredsum.vs v9, v10, v11",
            "vadd.vv v2, v0, v1, v0.t",
        ];

        for case in cases {
            let parsed = r5asm_pest::R5AsmParser::parse(r5asm_pest::Rule::instruction, case);
            assert!(parsed.is_ok(), "failed to parse RVV mask/reduction instruction: {case}");
        }
    }

    #[test]
    fn test_rvv_vset_instruction_encoding() {
        set_test_cwd_for_r5asm_data();

        let input = ".text\n\
vsetvli x5, x6, e32, m1, ta, ma\n\
vsetivli x7, 9, e16, mf2, tu, mu\n\
vsetvl x8, x9, x10\n";

        let params = build_snippet_parameters::BuildSnippetParameters::default();
        let bytes = assembler::build_asm_snippet(input, &params).expect("rvv vset snippet should build");
        let words = decode_u32_words(&bytes);

        let expected = vec![
            0x0D03_72D7, // vsetvli x5, x6, e32, m1, ta, ma
            0xC0F4_F3D7, // vsetivli x7, 9, e16, mf2, tu, mu
            0x80A4_F457, // vsetvl x8, x9, x10
        ];

        assert_eq!(words, expected);
    }

    #[test]
    fn test_rvv_unit_stride_load_store_encoding() {
        set_test_cwd_for_r5asm_data();

        let input = ".text\n\
vle32.v v1, (x2)\n\
vse32.v v1, (x2)\n";

        let params = build_snippet_parameters::BuildSnippetParameters::default();
        let bytes = assembler::build_asm_snippet(input, &params).expect("rvv unit-stride load/store snippet should build");
        let words = decode_u32_words(&bytes);

        let expected = vec![
            0x0201_6087, // vle32.v v1, (x2)
            0x0201_60A7, // vse32.v v1, (x2)
        ];

        assert_eq!(words, expected);
    }

    #[test]
    fn test_rvv_div_encoding_is_distinct_from_add() {
        set_test_cwd_for_r5asm_data();

        let input = ".text\n\
vadd.vv v2, v0, v1\n\
vdiv.vv v2, v0, v1\n";

        let params = build_snippet_parameters::BuildSnippetParameters::default();
        let bytes = assembler::build_asm_snippet(input, &params).expect("rvv add/div snippet should build");
        let words = decode_u32_words(&bytes);

        let expected = vec![
            0x0200_8157, // vadd.vv v2, v0, v1
            0x8600_A157, // vdiv.vv v2, v0, v1
        ];

        assert_eq!(words, expected);
        assert_ne!(words[0], words[1], "vdiv.vv must not alias vadd.vv");
    }

    #[test]
    fn test_rvv_divrem_encoding_reference() {
        set_test_cwd_for_r5asm_data();

        let input = ".text\n\
vdiv.vv v2, v0, v1\n\
vdivu.vv v3, v0, v1\n\
vrem.vv v4, v0, v1\n\
vremu.vv v5, v0, v1\n\
vdiv.vx v6, v0, x7\n\
vrem.vx v8, v0, x9\n";

        let params = build_snippet_parameters::BuildSnippetParameters::default();
        let bytes = assembler::build_asm_snippet(input, &params).expect("rvv div/rem reference snippet should build");
        let words = decode_u32_words(&bytes);

        let expected = vec![
            0x8600_A157, // vdiv.vv v2, v0, v1
            0x8200_A1D7, // vdivu.vv v3, v0, v1
            0x8E00_A257, // vrem.vv v4, v0, v1
            0x8A00_A2D7, // vremu.vv v5, v0, v1
            0x8603_E357, // vdiv.vx v6, v0, x7
            0x8E04_E457, // vrem.vx v8, v0, x9
        ];

        assert_eq!(words, expected);
    }

    #[test]
    fn test_rvv_value_operation_snippet_builds() {
        set_test_cwd_for_r5asm_data();

        let input = ".text\n\
vsetvli t0, a0, e32, m1, ta, ma\n\
vadd.vv v2, v0, v1\n\
vadd.vx v3, v2, t1\n\
vsub.vv v4, v1, v0\n";

        let params = build_snippet_parameters::BuildSnippetParameters::default();
        let bytes = assembler::build_asm_snippet(input, &params).expect("rvv value operation snippet should build");
        let words = decode_u32_words(&bytes);

        assert_eq!(words.len(), 4);
    }

    #[test]
    fn test_rvv_additional_value_operation_builds() {
        set_test_cwd_for_r5asm_data();

        let input = ".text\n\
vsetvli t0, a0, e32, m1, ta, ma\n\
vsub.vx v5, v3, t1\n";

        let params = build_snippet_parameters::BuildSnippetParameters::default();
        let bytes = assembler::build_asm_snippet(input, &params).expect("additional rvv value operation should build");
        let words = decode_u32_words(&bytes);

        assert_eq!(words.len(), 2);
    }

    #[test]
    fn test_rvv_wider_load_store_builds() {
        set_test_cwd_for_r5asm_data();

        let input = ".text\n\
vle8.v v1, (x2)\n\
vle16.v v2, (x3)\n\
vle64.v v3, (x4)\n\
vse8.v v1, (x2)\n\
vse16.v v2, (x3)\n\
vse64.v v3, (x4)\n";

        let params = build_snippet_parameters::BuildSnippetParameters::default();
        let bytes = assembler::build_asm_snippet(input, &params).expect("rvv wider load/store snippet should build");
        let words = decode_u32_words(&bytes);

        assert_eq!(words.len(), 6);
    }

    #[test]
    fn test_rvv_mask_immediate_and_reduction_builds() {
        set_test_cwd_for_r5asm_data();

        let input = ".text\n\
vsetvli t0, a0, e32, m1, ta, ma\n\
vadd.vi v1, v2, 7\n\
vand.mm v3, v4, v5\n\
vredsum.vs v6, v7, v8\n";

        let params = build_snippet_parameters::BuildSnippetParameters::default();
        let bytes = assembler::build_asm_snippet(input, &params).expect("rvv mask/immediate/reduction snippet should build");
        let words = decode_u32_words(&bytes);

        assert_eq!(words.len(), 4);
    }

    #[test]
    fn test_rvv_masked_value_and_indexed_strided_builds() {
        set_test_cwd_for_r5asm_data();

        let input = ".text\n\
vsetvli t0, a0, e32, m1, ta, ma\n\
vadd.vv v2, v0, v1, v0.t\n\
vlse32.v v3, x4, x5\n\
vsse32.v v3, x4, x5\n\
vluxei32.v v6, x7, x8\n\
vsuxei32.v v6, x7, x8\n";

        let params = build_snippet_parameters::BuildSnippetParameters::default();
        let bytes = assembler::build_asm_snippet(input, &params).expect("rvv masked/indexed/strided snippet should build");
        let words = decode_u32_words(&bytes);

        assert_eq!(words.len(), 6);
    }

    #[test]
    fn test_rvv_extended_value_operation_builds() {
        set_test_cwd_for_r5asm_data();

        let input = ".text\n\
vsetvli t0, a0, e32, m1, ta, ma\n\
vmul.vv v1, v2, v3\n\
vdivu.vx v4, v5, t1\n\
vminu.vv v6, v7, v8\n\
vmaxu.vx v9, v10, t2\n\
vor.vi v11, v12, 3\n\
vsra.vi v13, v14, 1\n";

        let params = build_snippet_parameters::BuildSnippetParameters::default();
        let bytes = assembler::build_asm_snippet(input, &params).expect("extended rvv value operations should build");
        let words = decode_u32_words(&bytes);

        assert_eq!(words.len(), 7);
    }

    #[test]
    fn test_rvv_alternate_mask_reduction_and_ordered_indexed_builds() {
        set_test_cwd_for_r5asm_data();

        let input = ".text\n\
vsetvli t0, a0, e32, m1, ta, ma\n\
vor.mm v1, v2, v3\n\
vnot.m v4, v5\n\
vredmin.vs v6, v7, v8\n\
vredmax.vs v9, v10, v11\n\
vloxei32.v v12, x13, x14\n\
vsoxei32.v v12, x13, x14\n";

        let params = build_snippet_parameters::BuildSnippetParameters::default();
        let bytes = assembler::build_asm_snippet(input, &params).expect("alternate rvv mask/reduction/indexed operations should build");
        let words = decode_u32_words(&bytes);

        assert_eq!(words.len(), 7);
    }

    #[test]
    fn test_rvv_vector_sample_files_build() {
        set_test_cwd_for_r5asm_data();

        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../test_data/test_r5asm");
        let cases = [
            "vector_divrem_ops.s",
            "vector_general.s",
            "vector_logic_more_ops.s",
            "vector_logic_ops.s",
            "vector_minmax_more_ops.s",
            "vector_minmax_ops.s",
            "vector_shift_more_ops.s",
            "vector_shift_ops.s",
            "vector_vi_ops.s",
            "vector_vx_ops.s",
        ];

        let params = build_snippet_parameters::BuildSnippetParameters::default();
        for case in cases {
            let path = base.join(case);
            let input = std::fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("failed to read RVV sample file {}", path.display()));
            let bytes = assembler::build_asm_snippet(&input, &params)
                .unwrap_or_else(|_| panic!("rvv sample file should build: {}", path.display()));
            assert!(!bytes.is_empty(), "rvv sample file produced no bytes: {}", path.display());
        }
    }
}