use parser_lib::markdown_lang::{markdown_pest::File, MarkdownPestError, RichText, Table};
use parser_lib::common::ParsingError;
use core_utils::number::u32_to_base26;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MDTableOffsetOrder {
	RowFirst,
	ColumnFirst,
}

fn get_col_id(col_names: &[String], col_idx: usize) -> String {
	match col_names.get(col_idx) {
		Some(name) if !name.trim().is_empty() => name.to_string(),
		_ => u32_to_base26(col_idx as u32),
	}
}

fn get_data_item_size(data_item: &str) -> Result<usize, ParsingError> {
	let trimmed = data_item.trim();
	if trimmed.is_empty() {
		return Ok(0);
	}

	let mut parts = trimmed.split_whitespace();
	let directive = parts.next().unwrap_or_default().to_ascii_lowercase();
	let remainder = parts.collect::<Vec<_>>().join(" ");

	let value_count = if remainder.trim().is_empty() {
		1
	} else {
		remainder
			.split(',')
			.filter(|x| !x.trim().is_empty())
			.count()
			.max(1)
	};

	let bytes_per_value = match directive.as_str() {
		".byte" => 1,
		".2byte" | ".half" | ".short" => 2,
		".4byte" | ".word" | ".long" | ".float" => 4,
		".8byte" | ".dword" | ".quad" | ".double" => 8,
		_ => {
			return Err(ParsingError::NoFound(
				(file!().to_string(), line!()).into(),
				format!("unsupported data directive '{directive}' from '{trimmed}'"),
			))
		}
	};

	Ok(bytes_per_value * value_count)
}

fn append_symbols_at_offset(
	out: &mut Vec<(usize, String)>,
	md_file: &File,
	col_names: &[String],
	header_text_no_space: &str,
	row_idx: usize,
	col_idx: usize,
	cell: &RichText,
	offset: usize,
) {
	let col_id = get_col_id(col_names, col_idx);

	out.push((
		offset,
		format!("array_{}_{}_{}", header_text_no_space, col_idx, row_idx),
	));
	out.push((offset, format!("{}_{}", col_id, row_idx)));

	for footnote_id in cell.get_footnotes_archer_id() {
		let trimmed = footnote_id.trim();
		if trimmed.is_empty() {
			continue;
		}

		if let Some(footnote_text) = md_file.get_footnote_text_from_archor_id(trimmed) {
			let footnote_name = footnote_text.trim();
			if !footnote_name.is_empty() {
				out.push((offset, footnote_name.to_string()));
			}
			out.push((offset, trimmed.to_string()));
		}
	}
}

/// calculate symbol offsets generated from a markdown data table.
/// symbols include:
/// - array_<header>_<col>_<row>
/// - <col_name_or_letter>_<row>
/// - footnote anchors used by a cell (when defined in the markdown file)
pub (crate) fn get_symbol_offsets_from_md_table(
	table: &Table,
	md_file: &File,
	order: MDTableOffsetOrder,
) -> Result<Vec<(usize, String)>, ParsingError> {
	let headers = md_file.get_headers_with_level(1);
	let header = headers.first();
	if header.is_none() {
		return Err(ParsingError::MarkdownPestError(MarkdownPestError::MissingTopLevelHeader));
	}

	let header_text = header.unwrap().get_text().replace(" ", "");
	let col_names = table.get_col_names()?;
	let rows = table.data_rows();

	let mut result = Vec::default();
	let mut offset = 0usize;

	match order {
		MDTableOffsetOrder::RowFirst => {
			for (row_idx, row) in rows.iter().enumerate() {
				for (col_idx, cell) in row.iter().enumerate() {
					append_symbols_at_offset(
						&mut result,
						md_file,
						&col_names,
						&header_text,
						row_idx,
						col_idx,
						cell,
						offset,
					);
					offset += get_data_item_size(&cell.get_text())?;
				}
			}
		}
		MDTableOffsetOrder::ColumnFirst => {
			let max_col = rows.iter().map(|row| row.len()).max().unwrap_or(0);
			for col_idx in 0..max_col {
				for (row_idx, row) in rows.iter().enumerate() {
					if let Some(cell) = row.get(col_idx) {
						append_symbols_at_offset(
							&mut result,
							md_file,
							&col_names,
							&header_text,
							row_idx,
							col_idx,
							cell,
							offset,
						);
						offset += get_data_item_size(&cell.get_text())?;
					}
				}
			}
		}
	}

	Ok(result)
}

/// convert table to multiple labeled data section in assembly
/// each element in the table will be labeled as
/// colname_rownumber, if colname is empty, use A, B, C... to replace
/// array_tablename_colnumber_rownumber, tablename is the top level header text and remove space
pub (crate) fn md_table_to_asm_data_section(table:&Table, md_file:&File) -> Result<Vec<String>, ParsingError> {
	let mut r = Vec::default();
    
	let headers = md_file.get_headers_with_level(1);
	let header = headers.first();
	if header.is_none() {
		return Err(ParsingError::MarkdownPestError(MarkdownPestError::MissingTopLevelHeader));
	}

	let header_text = header.unwrap().get_text();
	r.push(format!("\r\n`{header_text}`:"));

	let col_names = table.get_col_names()?;
	let mut row_number = 0;
	for row in table.data_rows() {
		let mut col_number = 0;
		for cell in row.iter() {
			let col_id = get_col_id(&col_names, col_number as usize);
			let label = format!("{col_id}_{row_number}:");
			let label2 = format!("array_{}_{col_number}_{row_number}:", header_text.replace(" ", ""));
			let data_item = cell.get_text();
			let footnotes = cell.get_footnotes_archer_id();
			let footnote_texts = footnotes.iter()
													.filter_map(|x| {
														let txt_option = md_file.get_footnote_text_from_archor_id(x);
														if let Some(txt) = txt_option {
															Some((x, txt))
														}
														else {
															None
														}
													})
													.map(|(id, x)| format!("{}: # {}", id.trim(), x.trim()))
													.collect::<Vec<_>>();

			r.extend_from_slice(&footnote_texts);
			r.push(label2);
			r.push(label);

			r.push(data_item);           

			col_number = col_number + 1;
		}

		row_number = row_number + 1;
	}

	Ok(r)
}

#[cfg(test)]
mod tests {
	use super::*;
	use parser_lib::markdown_lang::load_md_file_from_str;

	const TEST_MD: &str = "# Struct A\n\n| C0       | C1       |\n|----------|----------|\n| .word 1  | .dword 2 |\n| .word 3  | .word 4  |\n";

	#[test]
	fn symbol_offsets_row_first() {
		let file = load_md_file_from_str(TEST_MD).expect("md parse should work");
		let tables = file.get_tables();
		let table = tables.first().expect("table should exist");

		let offsets = get_symbol_offsets_from_md_table(&table, &file, MDTableOffsetOrder::RowFirst)
			.expect("offset calc should work");

		assert!(offsets.contains(&(0, "C0_0".to_string())));
		assert!(offsets.contains(&(4, "C1_0".to_string())));
		assert!(offsets.contains(&(12, "C0_1".to_string())));
		assert!(offsets.contains(&(16, "C1_1".to_string())));
	}

	#[test]
	fn symbol_offsets_column_first() {
		let file = load_md_file_from_str(TEST_MD).expect("md parse should work");
		let tables = file.get_tables();
		let table = tables.first().expect("table should exist");

		let offsets = get_symbol_offsets_from_md_table(&table, &file, MDTableOffsetOrder::ColumnFirst)
			.expect("offset calc should work");

		assert!(offsets.contains(&(0, "C0_0".to_string())));
		assert!(offsets.contains(&(4, "C0_1".to_string())));
		assert!(offsets.contains(&(8, "C1_0".to_string())));
		assert!(offsets.contains(&(16, "C1_1".to_string())));
	}

	#[test]
	fn symbol_offsets_empty_column_header_fallbacks_to_base26() {
		let md = "# Struct A\n\n|          | C1       |\n|----------|----------|\n| .word 1  | .word 2  |\n| .word 3  | .word 4  |\n";
		let file = load_md_file_from_str(md).expect("md parse should work");
		let tables = file.get_tables();
		let table = tables.first().expect("table should exist");

		let offsets = get_symbol_offsets_from_md_table(&table, &file, MDTableOffsetOrder::RowFirst)
			.expect("offset calc should work");

		assert!(offsets.contains(&(0, "A_0".to_string())));
		assert!(offsets.contains(&(4, "C1_0".to_string())));
		assert!(offsets.contains(&(8, "A_1".to_string())));
	}

	#[test]
	fn symbol_offsets_include_footnote_aliases() {
		let md = "# Struct A\n\n| C0       |\n|----------|\n| .word 1[^var1] |\n\n[^var1]: alias0\n";
		let file = load_md_file_from_str(md).expect("md parse should work");
		let tables = file.get_tables();
		let table = tables.first().expect("table should exist");

		let offsets = get_symbol_offsets_from_md_table(&table, &file, MDTableOffsetOrder::RowFirst)
			.expect("offset calc should work");

		assert!(offsets.contains(&(0, "var1".to_string())));
		assert!(offsets.contains(&(0, "alias0".to_string())));
	}
}
