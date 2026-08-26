use chrono::offset;
use parser_lib::mermaid_packet::*;
use parser_lib::common::ParsingError;

pub (crate) fn get_offsets_from_packet(file_path:&str) -> Result<Vec<(usize, String)>, ParsingError> {
    let packet_file = parse(file_path)
                        .map_err(|e| ParsingError::PacketErr(e))?;
    
    let name = packet_file.get_name().trim_start_matches("\"")
                                .trim_end_matches("\"")
                                .to_string();
    let package_name = if let Some(name2) = packet_file.get_name2() {
        format!("{name}.{name2}") }
        else { name };

    let mut r = Vec::default();
    let mut offset = 0;
    for entry in packet_file.get_entries() {
        let (value, name) = entry.into();
        r.push((offset, format!("{package_name}_{name}")));
        offset += value;
    }

    Ok(r)
}

/// generate equal statements from 'get_offsets_from_packet' result, for example:
/// 0:   .equ packet_name_field1, 0
pub (crate) fn generate_equ_statements_from_packet(file_path:&str) -> Result<Vec<String>, ParsingError> {
    let offsets = get_offsets_from_packet(file_path)?;
    let mut r = Vec::default();
    for (offset, name) in offsets {
        r.push(format!(".equ {name}, {offset}"));
    }

    Ok(r)
}