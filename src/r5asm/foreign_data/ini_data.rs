use core_utils::filesystem::read_file_content;
use parser_lib::ini::{InitFile, InitFileError, get_ini_properties};
use core_utils::debug::*;
use core_utils::number::*;

fn get_sorted_ini_properties(ini_file: &InitFile) -> Vec<(String, String)> {
    let mut items = get_ini_properties(ini_file).into_iter().collect::<Vec<_>>();
    items.sort_by(|left, right| left.0.cmp(&right.0));
    items
}

fn escape_asm_string(value: &str) -> String {
    core_utils::string::generate_escape_string(value)
}

fn parse_integer_literal(value: &str) -> Option<i128> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let normalized = trimmed.strip_prefix('+').unwrap_or(trimmed);
    let normalized = if let Some(rest) = normalized.strip_prefix("0X") {
        format!("0x{rest}")
    } else if let Some(rest) = normalized.strip_prefix("0B") {
        format!("0b{rest}")
    } else if let Some(rest) = normalized.strip_prefix("0O") {
        format!("0o{rest}")
    } else {
        normalized.to_string()
    };

    get_i128_from_str(&normalized).ok()
}

fn normalize_numeric_literal(value: &str) -> String {
    value.trim().replace('_', "")
}

fn infer_asm_directive(value: &str) -> (&'static str, String) {
    let lowered = value.trim().to_ascii_lowercase();

    if lowered == "true" {
        return (".byte", "1".to_string());
    }

    if lowered == "false" {
        return (".byte", "0".to_string());
    }

    if let Some(number) = parse_integer_literal(value) {
        if (0..=u8::MAX as i128).contains(&number) {
            return (".byte", number.to_string());
        }

        if (i32::MIN as i128..=i32::MAX as i128).contains(&number) {
            return (".word", number.to_string());
        }

        return (".dword", number.to_string());
    }

    if get_f32_from_str(value).map(|number| number.is_finite()).unwrap_or(false) {
        return (".float", normalize_numeric_literal(value));
    }

    if get_f64_from_str(value).map(|number| number.is_finite()).unwrap_or(false) {
        return (".double", normalize_numeric_literal(value));
    }

    (".string", format!("\"{}\"", escape_asm_string(value)))
}

/// Get the assembly data code from an ini file as a string.
pub fn get_asm_data_code_from_ini(ini_file : &InitFile) -> String {
    let asm_statements = get_sorted_ini_properties(ini_file)
        .iter()
        .map(|(name, value)| {
            let (directive, formatted_value) = infer_asm_directive(value);
            format!("{name}:   {directive} {formatted_value}")
        })
        .collect::<Vec<_>>();

    let r = format!(".data\r\n\r\n{}", asm_statements.join("\r\n"));
    r
}

pub fn ini_file_to_asm_data_code(file_path:&str) -> Result<String, InitFileError> {
    match read_file_content(file_path) {
        Ok(input) => {
            let ini_file = InitFile::parse(& input)?;
            let r = get_asm_data_code_from_ini(& ini_file);
            Ok(r)
        }
        Err(e) => {
            error_string(format!("ini file to asm data error: {e:?}"));
            Err(InitFileError::FileError)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infers_directives_from_scalar_values() {
        let ini = InitFile::parse(
            "plain=hello\nflag=true\nneg=-1\nsmall=255\nword=256\nhex=0x10\nlarge=4294967296\nsmall_float=3e-5\nsmall_float2=-3e4\nlarge_double=1e100\n"
        ).expect("valid ini");

        let asm = get_asm_data_code_from_ini(&ini);

        assert_eq!(
            asm,
            ".data\r\n\r\nflag:   .byte 1\r\nhex:   .byte 16\r\nlarge:   .dword 4294967296\r\nlarge_double:   .double 1e100\r\nneg:   .word -1\r\nplain:   .string \"hello\"\r\nsmall:   .byte 255\r\nsmall_float:   .float 3e-5\r\nsmall_float2:   .float -3e4\r\nword:   .word 256"
        );
    }

    #[test]
    fn escapes_string_values() {
        let escaped = escape_asm_string("line1\nline2\t\\end");

        assert_eq!(escaped, "line1\\nline2\\t\\\\end");
    }
}