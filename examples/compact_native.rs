use std::{env, fs, path::PathBuf, process::ExitCode};
use riscv_asm_lib::r5asm::{assembler::build_asm, code_gen_config::CodeGenConfiguration, OPTIMIZE_TO_COMPACT_CODE};

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 || !matches!(args[3].as_str(), "on" | "off") {
        return Err("usage: compact_native <input.s> <new-output.elf> <on|off> (RV64 only)".into());
    }
    let input = fs::canonicalize(&args[1]).map_err(|error| format!("input: {error}"))?;
    let output = PathBuf::from(&args[2]);
    if output.exists() {
        return Err(format!("refusing to overwrite {}", output.display()));
    }
    let parent = output.parent().filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."));
    let parent = fs::canonicalize(parent).map_err(|error| format!("output directory: {error}"))?;
    let output = parent.join(output.file_name().ok_or("missing output filename")?);
    env::set_current_dir(&parent).map_err(|error| format!("working directory: {error}"))?;
    let mut config = CodeGenConfiguration::default();
    // The existing opt-in is process-global; this example builds on one thread.
    unsafe { OPTIMIZE_TO_COMPACT_CODE = args[3] == "on"; }
    build_asm(input.to_str().ok_or("input path is not UTF-8")?,
        output.to_str().ok_or("output path is not UTF-8")?, &mut config)
        .map_err(|error| format!("assembly failed: {error:?}"))
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
