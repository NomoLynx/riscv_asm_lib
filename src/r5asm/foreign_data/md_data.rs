use parser_lib::markdown_lang::{markdown_pest::File, MarkdownPestError, Table};
use parser_lib::common::ParsingError;
use core_utils::number::u32_to_base26;

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
			let col_id = if col_names[col_number as usize].len()==0 { u32_to_base26(col_number) } 
								 else { col_names[col_number as usize].to_string() };
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
