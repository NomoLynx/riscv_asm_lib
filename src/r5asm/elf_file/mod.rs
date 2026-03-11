pub mod elf_file;
pub mod code_section;
pub mod data_section;
pub mod traits;
pub mod elf_header;

pub use elf_file::ElfFile;
pub use elf_header::*;

use std::fmt::Write;

use core_utils::debug::*;

/// Convert bytes to formatted hex string with 16 bytes per line
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len() * 3);
    for (i, chunk) in bytes.chunks(16).enumerate() {
        if i > 0 {
            hex.push('\n');
        }
        for byte in chunk {
            write!(&mut hex, "{:02x} ", byte).unwrap();
        }
    }
    hex.trim_end().to_string()
}

/// Convert hex string back to bytes
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, &'static str> {
    let mut bytes = Vec::new();
    let mut byte_str = String::new();
    
    for c in hex.chars() {
        if c.is_ascii_hexdigit() {
            byte_str.push(c);
            if byte_str.len() == 2 {
                bytes.push(u8::from_str_radix(&byte_str, 16)
                    .map_err(|_| "Invalid hex digit")?);
                byte_str.clear();
            }
        }
    }
    
    if !byte_str.is_empty() {
        Err("Odd number of hex digits")
    } else {
        Ok(bytes)
    }
}

// Alignment helper
pub fn align_up(size: usize, alignment: usize) -> usize {
    (size + alignment - 1) & !(alignment - 1)
}

/// Convert bytes to ASCII string, non-printable characters are replaced with '.'
pub (crate) fn bytes_to_ascii(bytes: &Vec<u8>) -> String {
    let mut out = String::new();

    for (i, &b) in bytes.iter().enumerate() {
        let c = if b.is_ascii_graphic() || b == b' ' {
            b as char
        } else {
            '.'
        };
        out.push(c);

        // insert newline every 16 characters
        if (i + 1) % 16 == 0 {
            out.push('\n');
        }
    }

    out
}

/// link the file in MD format
pub fn link(file_name: &str, output_file_name: &str) -> Result<(), String>{
    let loaded_md_file = ElfFile::from_markdown_file(file_name)?;
    loaded_md_file.save(output_file_name)?;
    output_string(format!("Linked file '{file_name}' saved to '{output_file_name}'"));
    Ok(())
}