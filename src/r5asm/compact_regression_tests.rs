use super::code_gen_config::CodeGenConfiguration;
use super::compact_inc::get_machine_code_from_compact_inc;
use super::instruction::Instruction;
use super::register::Register;
use core_utils::traits::generate_code::GenerateCode;

fn parse(source: &str, target: u8) -> Instruction {
    let mut config = CodeGenConfiguration::default();
    config.set_build_target(target);
    config.get_linker_config_mut().set_xlen(target * 8);
    let mut instructions = Instruction::from_string(source, &mut config)
        .unwrap_or_else(|error| panic!("{source}: {error:?}"));
    assert_eq!(instructions.len(), 1, "{source}");
    instructions.remove(0)
}

#[test]
fn compact_conversion_emits_all_current_c_forms_without_losing_operands() {
    let cases = [
        (8, "lw x31, 252(x2)", "c.lwsp", 0x5ffe),
        (8, "ld x31, 504(x2)", "c.ldsp", 0x7ffe),
        (4, "flw f31, 252(x2)", "c.flwsp", 0x7ffe),
        (8, "fld f0, 504(x2)", "c.fldsp", 0x307e),
        (8, "sw x0, 252(x2)", "c.swsp", 0xdf82),
        (8, "sd x31, 504(x2)", "c.sdsp", 0xfffe),
        (4, "fsw f0, 252(x2)", "c.fswsp", 0xff82),
        (8, "fsd f31, 504(x2)", "c.fsdsp", 0xbffe),
        (8, "lw x8, 124(x15)", "c.lw", 0x5fe0),
        (8, "ld x15, 248(x8)", "c.ld", 0x7c7c),
        (4, "flw f8, 124(x15)", "c.flw", 0x7fe0),
        (8, "fld f8, 248(x15)", "c.fld", 0x3fe0),
        (8, "sw x15, 124(x8)", "c.sw", 0xdc7c),
        (8, "sd x15, 248(x8)", "c.sd", 0xfc7c),
        (4, "fsw f15, 124(x8)", "c.fsw", 0xfc7c),
        (8, "fsd f15, 248(x8)", "c.fsd", 0xbc7c),
        (8, "jal x0, -2048", "c.j", 0xb001),
        (4, "jal x1, 2046", "c.jal", 0x2ffd),
        (8, "jalr x0, x31, 0", "c.jr", 0x8f82),
        (8, "jalr x1, x31, 0", "c.jalr", 0x9f82),
        (8, "beq x15, x0, -256", "c.beqz", 0xd381),
        (8, "bne x0, x15, 254", "c.bnez", 0xeffd),
        (8, "addi x31, x0, -1", "c.li", 0x5ffd),
        (8, "lui x31, 0xfffff", "c.lui", 0x7ffd),
        (8, "addi x31, x31, -32", "c.addi", 0x1f81),
        (8, "addiw x31, x31, 0", "c.addiw", 0x2f81),
        (8, "addi x2, x2, -512", "c.addi16sp", 0x7101),
        (8, "addi x15, x2, 1020", "c.addi4spn", 0x1ffc),
        (8, "slli x31, x31, 63", "c.slli", 0x1ffe),
        (8, "srli x15, x15, 63", "c.srli", 0x93fd),
        (8, "srai x15, x15, 63", "c.srai", 0x97fd),
        (8, "andi x15, x15, -32", "c.andi", 0x9b81),
        (8, "add x31, x0, x30", "c.mv", 0x8ffa),
        (8, "add x31, x31, x30", "c.add", 0x9ffa),
        (8, "and x15, x15, x8", "c.and", 0x8fe1),
        (8, "or x15, x15, x8", "c.or", 0x8fc1),
        (8, "xor x15, x15, x8", "c.xor", 0x8fa1),
        (8, "sub x15, x15, x8", "c.sub", 0x8f81),
        (8, "addw x15, x15, x8", "c.addw", 0x9fa1),
        (8, "subw x15, x15, x8", "c.subw", 0x9f81),
        (8, "addi x0, x0, 0", "c.nop", 0x0001),
        (8, "ebreak", "c.ebreak", 0x9002),
    ];
    for (target, source, name, expected) in cases {
        let original = parse(source, target);
        let compact = original.convert_to_compact_for_target(target)
            .unwrap_or_else(|| panic!("{source} should compress to {name}"));
        assert_eq!(compact.name, name, "{source}");
        let machine = get_machine_code_from_compact_inc(&compact, 2, &Register::new(), target)
            .unwrap_or_else(|error| panic!("{source}: {error:?}"));
        assert_eq!(machine.get_size(), 2, "{source}");
        assert_eq!(machine.to_vec(), (expected as u16).to_le_bytes(), "{source}");
        let rendered = compact.generate_code_string();
        let reparsed = parse(&rendered, target);
        let rebuilt = get_machine_code_from_compact_inc(&reparsed, 2, &Register::new(), target).unwrap();
        assert_eq!(rebuilt.to_vec(), machine.to_vec(), "{rendered}");
    }
}

    #[test]
    fn compact_signed_ci_fields_keep_full_register_ids_and_legal_hints() {
        for target in [4, 8] {
            for rd in [1, 2, 15, 31] {
                for imm in -32..=31 {
                    let instruction = parse(&format!("c.addi x{rd}, {imm}"), target);
                    let machine = get_machine_code_from_compact_inc(&instruction, 2, &Register::new(), target).unwrap();
                    let bytes: [u8; 2] = machine.to_vec().try_into().unwrap();
                    let word = u16::from_le_bytes(bytes);
                    assert_eq!((word >> 7) & 31, rd);
                    let field = ((word >> 12) & 1) << 5 | (word >> 2) & 31;
                    let decoded = (field as i16) << 10 >> 10;
                    assert_eq!(decoded, imm);
                }
            }
        }
    }

    #[test]
    fn compact_conversion_preserves_aliases_commutativity_and_source_ranges() {
        let original = parse("and s0, x8, a1", 8);
        let compact = original.convert_to_compact().unwrap();
        assert_eq!(compact.name, "c.and");
        assert_eq!(compact.r0_name.as_deref(), Some("s0"));
        assert_eq!(compact.r1_name.as_deref(), Some("a1"));
        assert_eq!(compact.get_r0_location(), original.get_r0_location());
        assert_eq!(compact.get_r1_location(), original.get_r2_location());
        assert_eq!(compact.get_name_location(), original.get_name_location());
        assert!(compact.get_r2_location().is_none());
        let machine = get_machine_code_from_compact_inc(&compact, 0, &Register::new(), 8).unwrap();
        assert_eq!(machine.to_vec(), 0x8c6du16.to_le_bytes());

        let original = parse("add a0, a1, x10", 8);
        let compact = original.convert_to_compact().unwrap();
        assert_eq!(compact.r1_name.as_deref(), Some("a1"));
        assert_eq!(compact.get_r1_location(), original.get_r1_location());
        assert_eq!(get_machine_code_from_compact_inc(&compact, 0, &Register::new(), 8).unwrap().to_vec(),
            0x952eu16.to_le_bytes());
    }

    #[test]
    fn compact_conversion_respects_width_and_relocation_boundaries() {
        for source in ["ld x8, 0(x9)", "sd x8, 0(x9)", "addiw x8, x8, 1",
            "addw x8, x8, x9", "subw x8, x8, x9",
            "slli x8, x8, 32", "srli x8, x8, 32", "srai x8, x8, 32"]
        {
            assert!(parse(source, 4).convert_to_compact_for_target(4).is_none(), "{source}");
        }
        for source in ["flw f8, 0(x9)", "fsw f8, 0(x9)", "jal x1, 6",
            "slli x8, x8, 0", "srli x8, x8, 0", "srai x8, x8, 0",
            "addi x8, x8, %lo(symbol)", "beq x8, x0, symbol", "jal x0, symbol"]
        {
            assert!(parse(source, 8).convert_to_compact_for_target(8).is_none(), "{source}");
        }
    }

    #[test]
    fn compact_encoder_rejects_reserved_fields_wrong_classes_and_wrong_widths() {
        for (target, source) in [
            (8, "c.lwsp x0, 0"), (8, "c.lwsp x8, 4(x3)"), (8, "c.lw x8, 2(x9)"),
            (8, "c.ldsp x31, 512"), (8, "c.addiw x0, 0"),
            (8, "c.addi4spn x8, 0"), (8, "c.addi16sp x2, 0"),
            (8, "c.lui x8, 0"), (8, "c.lui x2, 1"),
            (8, "c.jr x0"), (8, "c.mv x1, x0"), (8, "c.add x1, x0"),
            (8, "c.slli x31, 64"), (8, "c.beqz x8, 256"), (8, "c.bnez x8, 3"),
            (8, "c.j -2050"), (8, "c.lw f8, 0(x9)"), (8, "c.fld x8, 0(x9)"),
            (8, "c.lw x8, 0(f9)"), (8, "c.add v8, x9"),
            (4, "c.ld x8, 0(x9)"), (4, "c.sd x8, 0(x9)"),
            (4, "c.addiw x1, 1"), (4, "c.addw x8, x9"), (4, "c.subw x8, x9"),
            (4, "c.srli x8, 32"), (8, "c.flw f8, 0(x9)"), (8, "c.jal 2"),
        ] {
            let instruction = parse(source, target);
            assert!(get_machine_code_from_compact_inc(&instruction, 0, &Register::new(), target).is_err(), "{source}");
        }
    }

#[test]
fn compact_conversion_keeps_non_equivalent_and_unsupported_forms() {
    for source in [
        "lw x0, 0(x2)", "lw x8, 2(x9)", "lw x8, -4(x9)",
        "lw x8, 128(x9)", "lw x31, 256(x2)", "ld x31, 512(x2)",
        "addi x31, x0, 32", "addi x8, x9, 1", "addi x2, x2, -1024",
        "andi x7, x7, -1", "srli x7, x7, 1", "slli x0, x0, 1",
        "jalr x0, x0, 0", "jalr x2, x8, 0", "jalr x1, x8, 2",
        "beq x8, x9, 8", "beq x7, x0, 8", "beq x8, x0, 256",
        "beq x8, x0, -258", "bne x8, x0, 3", "jal x0, 2048",
        "jal x0, -2050", "lui x2, 1", "lui x8, 32", "lui x0, 1",
        "add x8, x9, x10", "sub x8, x9, x8", "lw x8, symbol(x9)",
        "lbu x8, 0(x9)", "lh x8, 0(x9)", "lhu x8, 0(x9)",
        "sb x8, 0(x9)", "sh x8, 0(x9)", "mul x8, x8, x9",
        "xori x8, x8, -1", "andi x8, x8, 255",
    ] {
        assert!(parse(source, 8).convert_to_compact().is_none(), "{source}");
    }
}
