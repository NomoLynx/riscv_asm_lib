use std::fs;
use std::path::Path;

use chrono::Local;
use parser_lib::markdown_lang::load_md_file_from_str;

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

fn main() {
    print_time("start");

    let asm_dir = Path::new("src/data");
    println!("cargo:rerun-if-changed={}", asm_dir.display());

    let macro_instruction_md = Path::new("src/r5asm/macro_instruction/macro_instruction.md");
    println!("cargo:rerun-if-changed={}", macro_instruction_md.display());

    let markdown = fs::read_to_string(macro_instruction_md)
        .expect("failed to read macro_instruction.md");
    let macro_instruction_names = parse_macro_instruction_names(&markdown);

    println!(
        "cargo:warning=[riscv_asm_lib build] parsed {} macro instructions",
        macro_instruction_names.len()
    );

    print_time("end");
}
