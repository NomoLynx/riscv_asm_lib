use std::collections::HashMap;

use pest::iterators::Pair;

use core_utils::debug::*;
use core_utils::traits::generate_code::GenerateCode;
use parser_lib::expr_lang::*;
use super::super::{asm_error::AsmError, code_gen_config::CodeGenConfiguration, label_offset::LabelOffsetTable, r5asm_pest::{EquTable, Rule, SectionItem, SectionItem2, from_pair_template}};

use super::super::calculate_padding;
use super::super::elf_section::section_type::*;
use super::super::instruction::SourceRange;

/// Section in the assembly program, which contains a list of instructions or data directives, 
/// and also contains the label and offset table for the section
#[derive(Debug, Clone)]
pub struct Section {
    section_tag: SectionType,
    section_tag_range: Option<SourceRange>,
    section_items : Vec<SectionItem2>,
    section_length : usize,
    sectino_offset : usize,
    label_offset_table : LabelOffsetTable,
}

impl Section {
    pub (crate) fn from_pair(pair:&Pair<Rule>, config:&mut CodeGenConfiguration) -> Result<Option<Self>, AsmError> {
        if pair.as_rule() == Rule::EOI {
            Ok(None)
        }
        else {
            from_pair_template(pair, Rule::section, config, |rule:Vec<(Rule, Pair<Rule>)>, config| {
                match rule.as_slice() {
                    [(Rule::section_tag, p), others @ ..] => {
                        let section_type = SectionType::from_str(p.as_str()).ok_or(AsmError::IncompatibleType((file!(), line!()).into()))?;
                        let mut items = Vec::default();
                        for (_m, n) in others {
                            let rr = SectionItem::from_pair(n, config);
                            match rr 
                            {
                                Ok(r0) => {
                                    for item in r0 {
                                        let r1 = SectionItem2::new(0, item);
                                        items.push(r1)
                                    }
                                }
                                Err(e) => {
                                    error_string(format!("error when parsing {n:?}"));
                                    error_string(format!("Error = {e:?}"));
                                    return Err(e)
                                }
                            }
                        }
                        
                        Ok(Some(Self { 
                            section_tag : section_type,
                            section_tag_range: Some(SourceRange::from_pair(p)),
                            section_items : items,
                            section_length : 0,
                            sectino_offset : 0,
                            label_offset_table : LabelOffsetTable::new(),
                        }))
                    }
                    _ => Err(AsmError::MissingCase((file!(), line!()).into(), Rule::section)),
                }
            })
        }
    }

    pub (crate) fn update_item_offset(&mut self) -> Result<(), AsmError> {
        let mut offset = 0;
        for n in self.section_items.iter_mut() {
            n.set_offset(offset);
            offset = offset + n.get_item().get_size()?;
        }

        self.section_length = offset;

        Ok(())
    }

    /// get labels and the next item which is an instruction
    pub (crate) fn get_labels_and_next_items(&self) -> Vec<(&SectionItem2, Option<&SectionItem2>)> {
        let mut r = Vec::default();
        for n in self.section_items.iter() {
            if n.is_label() {
                if let Some(_label) = n.get_label() {
                    if let Some(next_item) = self.section_items.iter().find(|x| x.is_inc() && x.get_offset() > n.get_offset()) {
                        r.push((n, Some(next_item)));
                    } else {
                        r.push((n, None));
                    }
                }
            }
        }

        r
    }
    
    pub (crate) fn get_equ_list(&self) -> EquTable {
        let mut r = HashMap::default();
        for n in self.section_items.iter() {
            if let Some(m) = n.get_directive() {
                if let Some((key, value)) = m.get_equ() {
                    r.insert(key.to_string(), value.to_string());
                }
            }
        }

        r
    }

    pub (crate) fn get_section_type(&self) -> SectionType {
        self.section_tag.clone()
    }

    pub fn get_section_tag_range(&self) -> Option<&SourceRange> {
        self.section_tag_range.as_ref()
    }

    pub (crate) fn set_offset(&mut self, offset:usize) {
        self.sectino_offset = offset
    }

    pub (crate) fn get_section_length(&self) -> usize {
        self.section_length
    }

    /// update the offset of an instruction or data directive
    /// also need to expand .align to meet the alignment requirement
    pub (crate) fn update_offset(&mut self) {
        let mut padding = 0;
        for item in self.section_items.iter_mut() {
            item.set_offset(item.get_offset() + self.sectino_offset + padding);
            if let Some(align) = item.get_align() {
                let item_offset = item.get_offset();
                let padding_size = calculate_padding(item_offset as u64, align);
                padding += padding_size as usize;
                item.set_padding_data(padding_size as u32);
            }
        }

        self.section_length += padding;
    }

    pub fn get_label_offset_table(&self) -> &LabelOffsetTable {
        &self.label_offset_table
    }

    /// generate the label and offset table
    pub fn generate_label_and_offset(&self) -> LabelOffsetTable {
        let mut r = LabelOffsetTable::default();
        let mut seq = 0;
        for n in self.section_items.iter() {
            if n.is_label() {
                if let Some(s) = n.get_label() {
                    let related_label = n.get_related_label().unwrap_or_default();
                    r.add_entry(s.to_string(), n.get_offset(), seq, &related_label);
                    seq += 1;
                }
            }
            else if n.is_directive() {
                let directive = n.get_directive().unwrap();
                if let Some(s2) = directive.get_extern_label() {
                    r.add_entry(s2.to_string(), n.get_offset(), seq, "");
                    seq += 1;
                }
            }
        }

        r
    }

    /// create mutable label and offset table object in the section
    pub fn create_label_and_offset_table(&mut self) {
        let mut table = self.generate_label_and_offset();

        // enrich symbols, like global label requires to set to global, and set its size
        // process the global directive
        for item in self.get_all_items() {
            if let Some(name) = item.get_global_label() {
                table.set_label_global(name);
            }
        }

        // set the size of the label if possible
        for item in self.get_all_items() {
            if let Some((label_name, size_expr_str)) = item.get_size_directive() {
                let context = SizeDirectiveComputeContext {
                    section_item : item,
                    label_offset_table : &table,
                };

                let v = expr_to_clrobj(&size_expr_str, Some(&context)).unwrap()
                                .get_usize().unwrap();
                table.set_label_size(&label_name, v);
            }
        }

        self.label_offset_table = table;
    }

    /// get all items in the section, SectionItem2 type, use get_section_items() to get SectionItem type
    pub fn get_all_items(&self) -> &Vec<SectionItem2> {
        &self.section_items
    }

    /// get all items mut in the section, SectionItem2 type, use get_section_items() to get SectionItem type
    pub fn get_all_items_mut(&mut self) -> &mut Vec<SectionItem2> {
        &mut self.section_items
    }

    /// set all items in the section, SectionItem2 type
    pub fn set_all_items(&mut self, items:Vec<SectionItem2>) {
        self.section_items = items;
    }

    /// get all items in the section, SectionItem type, use get_all_items() to get SectionItem2 type
    pub fn get_section_items(&self) -> Vec<&SectionItem> {
        let r = self.get_all_items().iter()
            .map(|x| x.get_item())
            .collect::<Vec<_>>();

        r
    }

    /// get inc type SectionItem2
    pub fn get_instructions(&self) -> Vec<&SectionItem2> {
        self.section_items.iter()
            .filter(|x| x.is_inc())
            .collect::<Vec<_>>()
    }

    pub fn contains_external_symbol(&self) -> bool {
        self.section_items.iter()
            .any(|x| x.get_external_label().is_some())
    }

    pub fn contains_global_directive(&self) -> bool {
        self.section_items.iter()
            .any(|x| x.get_global_label().is_some())
    }

    /// append label to the section, mainly for build snippet function
    pub fn append_label(&mut self, offset:usize, label: &str) {
        let entry = SectionItem2::new(offset, SectionItem::new_label(label.into()));
        self.section_items.push(entry);
    }

    /// is text section
    pub fn is_text_section(&self) -> bool {
        self.section_tag == SectionType::Text
    }

    /// is data section
    pub fn is_data_section(&self) -> bool {
        self.section_tag == SectionType::Data
    }

    /// new empty data section
    pub fn new_empty_data_section() -> Self {
        let mut section = Section::default();
        section.set_section_tag(SectionType::Data);
        section
    }

    /// set section tag
    pub fn set_section_tag(&mut self, section_type:SectionType) {
        self.section_tag = section_type;
    }

    /// get directives in the section
    pub fn get_directives(&self) -> Vec<&SectionItem2> {
        self.section_items.iter()
            .filter(|x| x.is_directive())
            .collect::<Vec<_>>()
    }
}

impl GenerateCode for Section {
    fn generate_code_string(&self) -> String {
        let section_type_str = self.section_tag.generate_code_string();
        let code_list = self.section_items.iter()
                        .map(|x| x.generate_code_string())
                        .collect::<Vec<_>>();
        let code = code_list.join("\r\n");
        let r = format!("{section_type_str}\r\n\r\n{code}");
        r
    }
}

impl Default for Section {
    fn default() -> Self {
        Section { 
            section_tag: SectionType::Text,
            section_tag_range: None,
            section_items: Vec::default(), 
            section_length: 0, 
            sectino_offset: 0,
            label_offset_table : LabelOffsetTable::new(), 
        }
    }
}

pub (self) struct SizeDirectiveComputeContext<'a> {
    pub section_item : &'a SectionItem2,
    pub label_offset_table : &'a LabelOffsetTable,
}

impl ExpressionContextTrait for SizeDirectiveComputeContext<'_> {
    fn get_value(&self, identifier:&str) -> Option<ExprValue> {
        if identifier == "." {
            Some(ExprValue::from_i64(self.section_item.get_offset() as i64))
        }
        else {
            self.label_offset_table.get_single_label_offset(identifier, None)
                .map(|x| ExprValue::from_i64(x as i64))
        }
    }
}