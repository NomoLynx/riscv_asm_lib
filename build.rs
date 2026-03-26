use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::Path;

use chrono::Local;
use parser_lib::markdown_lang::load_md_file_from_str;
use parser_lib::pest_parser::{PestAtom, PestExpression};
use parser_lib::pest_parser::pest_pest::pest_parse;

fn print_time(label: &str) {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    println!("cargo:warning=[riscv_asm_lib build] {label} at {now}");
}

fn parse_macro_instruction_names(markdown: &str) -> Vec<String> {
    let md = load_md_file_from_str(markdown).expect("failed to parse macro_instruction.md");
    let tables = md.get_tables();
    let table = tables.first().expect("no table found in macro_instruction.md");
    table
        .data_rows()
        .iter()
        .map(|row| row[0].get_text().trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn extract_string_literals(expression: &PestExpression) -> Vec<String> {
    let mut values = Vec::new();

    for atom in expression.get_all_atoms() {
        match atom {
            PestAtom::String(_, value, _) => {
                values.push(value.trim_matches('"').to_string());
            }
            PestAtom::Expression(_, nested, _) => {
                values.extend(extract_string_literals(nested));
            }
            _ => {}
        }
    }

    values
}

fn normalize_pest_source_for_parser(pest_source: &str) -> String {
    pest_source.replace("#inc_name = ", "")
}

fn parse_all_pest_string_literals(pest_source: &str) -> Vec<String> {
    let normalized_pest_source = normalize_pest_source_for_parser(pest_source);
    let rules = pest_parse(&normalized_pest_source).expect("failed to parse r5asm.pest");
    let mut instruction_names = BTreeSet::new();

    for rule in rules {
        for value in extract_string_literals(rule.get_expression()) {
            if !value.is_empty() {
                instruction_names.insert(value);
            }
        }
    }

    instruction_names.into_iter().collect()
}

fn validate_markdown_instructions_are_defined(markdown_names: &[String], pest_source: &str) {
    let pest_names: HashSet<String> = parse_all_pest_string_literals(pest_source)
        .into_iter()
        .collect();

    for missing_name in markdown_names
        .iter()
        .filter(|name| !pest_names.contains(name.as_str()))
    {
        println!(
            "cargo:warning=[riscv_asm_lib build] macro instruction '{}' is not defined in r5asm.pest",
            missing_name
        );
    }
}

fn main() {
    print_time("start");

    let asm_dir = Path::new("src/data");
    println!("cargo:rerun-if-changed={}", asm_dir.display());

    let macro_instruction_md = Path::new("src/r5asm/macro_instruction/macro_instruction.md");
    println!("cargo:rerun-if-changed={}", macro_instruction_md.display());
    let r5asm_pest = Path::new("src/r5asm/r5asm.pest");
    println!("cargo:rerun-if-changed={}", r5asm_pest.display());

    let markdown = fs::read_to_string(macro_instruction_md)
        .expect("failed to read macro_instruction.md");
    let macro_instruction_names = parse_macro_instruction_names(&markdown);
    let pest_source = fs::read_to_string(r5asm_pest)
        .expect("failed to read r5asm.pest");

    validate_markdown_instructions_are_defined(&macro_instruction_names, &pest_source);

    println!(
        "cargo:warning=[riscv_asm_lib build] parsed {} macro instructions",
        macro_instruction_names.len()
    );

    print_time("end");
}
