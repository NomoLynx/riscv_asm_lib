use std::collections::HashMap;

use core_utils::filesystem::write_to_file;
use pest::iterators::Pair;

use core_utils::traits::generate_code::GenerateCode;
use crate::r5asm::code_option::option::CodeOption;

use super::basic_instruction_extensions::BasicInstructionExtensions;
use super::code_gen_config::CodeGenConfiguration;
use super::compact_inc::get_machine_code_from_compact_inc;
use super::directive::Directive;
use super::dynamic_structure::ELFDynamicStructure;
use super::external_label::ExternalLabel;
use super::imm_macro::ImmMacro;
use super::instruction::Instruction;
use super::label_offset::{LabelOffsetTable, LabelOffsetTableEntry, LabelTable};
use super::elf_file::code_section::CodeSection;
use super::elf_file::data_section::ReadOnlySection;
use super::elf_file::data_section::DataSection;
use super::elf_section::*;
use super::elf_file::*;
use super::machinecode::MachineCode;
use super::round_to_usize;
use super::opcode::OpCode;
use super::r5asm_pest::{EquTable, InstructionRegisterName, InstructionTypes, Rule, SectionItem, SectionItem2, from_pair_vec_null_fn_template};
use super::register::Register;
use super::section::Section;
use super::traits::{SectionSizeTrait, ToMarkdown};
use super::vector_incs::vector_emitter::emit_vector_instruction;
use super::{OPTIMIZE_CODE_GEN, OPTIMIZE_TO_COMPACT_CODE, asm_error::*, reverse_string};
use core_utils::debug::*;

#[derive(Debug, Clone)]
pub struct AsmProgram {
    sections : Vec<Section>,
    options: Vec<CodeOption>, 
}

impl AsmProgram {
    pub (crate) fn from_pair(pair:&Pair<Rule>, config:&mut CodeGenConfiguration) -> Result<Self, AsmError> {
        let pairs = pair.to_owned().into_inner().into_iter().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
        
        let mut sections = Vec::default();
        let mut options = Vec::default();

        for (r, p) in pairs {
            match r {
                Rule::global_option => {
                    let option_pairs = p.into_inner()
                                        .filter(|x| x.as_rule() == Rule::option_directive_param)
                                        .collect::<Vec<_>>();
                    for option_pair in option_pairs {
                        let str = option_pair.as_str().to_string();
                        let option = CodeOption::from_str(&str)
                                            .ok_or(AsmError::NoFound((file!(), line!()).into(), format!("cannot find code option for '{str}'")))?;
                        options.push(option);
                    }
                },
                Rule::section => {
                    if let Some(section) = Section::from_pair(&p, config)? {
                        sections.push(section);
                    }
                },
                _ => {}
            }
        }

        Ok(Self {
            sections,
            options,
        })
    }

    fn contains_external_symbol(&self) -> bool {
        self.sections.iter()
            .any(|x| x.contains_external_symbol())
    }

    fn contains_global_directive(&self) -> bool {
        self.sections.iter()
            .any(|x| x.contains_global_directive())
    }

    /// perform replace equal value, pseudo code to real inc, and replace label in jal
    pub (crate) fn second_round(&mut self, config:&mut CodeGenConfiguration) -> Result<(), AsmError> {
        let equals = self.get_equals();
        for section in self.sections.iter_mut() {
            if section.get_section_type() == SectionType::Text {
                let mut new_item_list = Vec::default();
                let items = section.get_all_items_mut();
                while !items.is_empty() {
                    let mut item = items.remove(0);

                    //replace the equ value
                    Self::replace_equ(&mut item, &equals, config)?;

                    //make pseudo code to real inc
                    if item.is_pseodu_inc() {
                        let new_items = Self::get_inc_from_pseduo(&item)?;
                        new_item_list.extend(new_items);
                    }
                    else {
                        if unsafe { OPTIMIZE_TO_COMPACT_CODE } {
                            if let Some(inc) = item.get_inc() {
                                if let Some(new_inc) = inc.convert_to_compact() {
                                    item.set_inc(new_inc);
                                }
                            }
                        }

                        new_item_list.push(item);
                    }
                }

                section.set_all_items(new_item_list);
            }

            section.update_item_offset()?;  //update the offset for section item and length for the whole section
        }

        // replace psuedo inc call and possible for other instruciton which need label value
        for section in self.sections.iter_mut() {
            if section.get_section_type() == SectionType::Text {
                let labels = section.generate_label_and_offset();
                let mut new_item_list = Vec::default();
                let items = section.get_all_items_mut();
                while !items.is_empty() {
                    let item = items.remove(0);
                    if item.is_pseodu_inc() {
                        let new_items = Self::get_inc_from_pseudoinc_call(&item, &labels)?;
                        new_item_list.extend(new_items);
                    }
                    else {
                        new_item_list.push(item);
                    }
                }

                section.set_all_items(new_item_list);
            }

            section.update_item_offset()?;  //update the offset for section item and length for the whole section
        }

        //replace label in jal, this has to be after call because previous code (e.g. call) can modify existing instruction's offset
        for section in self.sections.iter_mut() {
            if section.get_section_type() == SectionType::Text {
                let labels = section.generate_label_and_offset();
                let mut new_item_list = Vec::default();
                let items = section.get_all_items_mut();
                while !items.is_empty() {
                    let item = items.remove(0);
                    if item.is_inc() {
                        let new_items = Self::get_offset_in_jal(&item, &labels)?;
                        if new_items.is_empty() {
                            new_item_list.push(item);
                        }
                        else {
                            new_item_list.extend(new_items);
                        }
                    }
                    else {
                        new_item_list.push(item);
                    }
                }

                section.set_all_items(new_item_list);
            }

            section.update_item_offset()?;
        }

        let labeled_items = self.get_labeled_directive_size()?;
        for section in self.sections.iter_mut() {
            if section.get_section_type() == SectionType::Text {
                //calculate relocation function
                for item in section.get_all_items_mut() {
                    Self::cal_rel_fun(item, &labeled_items)?;
                }
            }
        }

        // enrich the label table data (e.g. set global, set global label size)
        for section in self.sections.iter_mut() {
            section.create_label_and_offset_table();
        }

        Ok(())
    }

    /// mainly for optimization
    pub (crate) fn third_round(&mut self) -> Result<(), AsmError> {
        if unsafe { !OPTIMIZE_CODE_GEN } {
            return Ok(())
        }
        
        for section in self.sections.iter_mut() {
            if section.get_section_type() == SectionType::Text {
                for inc in section.get_all_items_mut() {
                    match inc.get_inc_mut() {
                        Some(n) => {
                            if n.get_op_code()? == OpCode::Addi && n.imm_to_u32()? == 0 {
                                n.set_is_generate(false)
                            }
                        }
                        None => {}
                    }
                }
            }
        }

        Ok(())
    }

    pub fn get_non_text_section_items(&self) -> Vec<&SectionItem2> {
        let mut r = Vec::default();
        for section in self.sections.iter() {
            if section.get_section_type() != SectionType::Text {
                for item in section.get_all_items() {
                    r.push(item);
                }
            }
        }

        r
    }

    pub fn get_text_section_items(&self) -> Vec<&SectionItem2> {
        let mut r = Vec::default();
        for section in self.sections.iter() {
            if section.get_section_type() == SectionType::Text {
                for item in section.get_all_items() {
                    r.push(item);
                }
            }
        }

        r
    }

    fn get_labeled_section_item(&self, lable:&str) -> Option<&SectionItem2> {
        let l = self.get_non_text_section_items();
        if let Some(pos) = l.iter().position(|x| x.is_label() && x.get_label().unwrap() == lable) {
            l.iter().nth(pos+1).copied()
        }
        else {
            None
        }
    }

    fn get_labeled_directive_size(&self) -> Result<HashMap<String, usize>, AsmError> {
        let l = self.get_labels();
        let labels = l.keys();
        let mut r = HashMap::default();
        for n in labels.into_iter() {
            let name = n.get_real_name().to_string();
            if let Some(item) = self.get_labeled_section_item(&name) {
                if let Some(directive) = item.get_directive() {
                    r.insert(name, directive.get_directive_size()?);
                }
            }
        }

        Ok(r)
    }

    /// calculate some rel functions like %len
    fn cal_rel_fun(item:&mut SectionItem2, item_list:&HashMap<String, usize>) -> Result<(), AsmError> {
        if let Some(inc) = item.get_inc_mut() {
            if let Some(rel) = &inc.rel_fun {
                if rel.to_lowercase().as_str() == "%len" {
                    if let Some(label_name) = inc.get_imm().and_then(|x| x.get_value()) {
                        if item_list.contains_key(&label_name) {
                            let size = item_list[&label_name];
                            inc.set_imm(Some(format!("{size}").into()));
                            inc.rel_fun = None; //clear the rel function so it won't cause more panic
                        }
                        else {
                            return Err(AsmError::NoFound((file!(), line!()).into(), format!("cannot find lable name = '{label_name}'")))
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    /// replace equ value in instruction or directive
    fn get_inc_from_pseduo(item:&SectionItem2) -> Result<Vec<SectionItem2>, AsmError> {
        if !item.is_pseodu_inc() {
            return Err(AsmError::WrongType((file!(), line!()).into(), format!("Can only convert a non-pesuedo code. The current code is {item:?}")))
        }

        let inc = item.get_inc().unwrap();
        match inc.name.to_lowercase().as_str() {
            "li" => {
                let rd = inc.r0_name.as_deref().unwrap_or("x0");
                let imm = inc.imm_to_i64()?;
                let incs = Instruction::li_from_imm(rd, imm)?;
                Ok(incs.into_iter().map(|i| SectionItem2::new(0, SectionItem::Instruction(i))).collect())
            }
            "call" => { 
                // don't process here and leave for next function
                Ok(vec![item.clone()])
            }
            _ => {
                error_string(format!(""));
                Err(AsmError::NoFound((file!(), line!()).into(), format!("need to add process logic for {inc:?} when convert psudeo code")))
            }
        }
    }

    /// get offset in branch and jump instructions, 
    /// if the offset is not label, return empty vec which means current function does not process the instruction
    fn get_offset_in_jal(item:&SectionItem2, label_offset:&LabelOffsetTable) -> Result<Vec<SectionItem2>, AsmError> { 
        if !item.is_inc() {
            return Err(AsmError::WrongType((file!(), line!()).into(), format!("Can only convert an inc. The current code is {item:?}")))
        }

        let inc = item.get_inc().unwrap();
        let current_offset = item.get_offset();
        let name = inc.name.to_lowercase();
        match name.as_str() {
            "blt" |
            "bge" |
            "bltu" |
            "bgeu" |
            "bne" |
            "beq" |
            "jal" => {
                if let Some(label) = inc.get_imm().and_then(|x| x.get_value()) {
                    if let Some(target_offset) = label_offset.get_single_label_offset(&label, Some(item.get_offset())) {
                        let offset = target_offset as i64 - current_offset as i64;
                        let (inc_name, inc_type) = Instruction::get_inc_and_inc_type(&name)?;
                        let mut r = Instruction::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions);
                        r.set_r0_value(inc.get_r0().unwrap());
                        if inc.get_r1().is_some() {
                            r.set_r1_value(inc.get_r1().unwrap());
                        }
                        r.set_imm_value(& offset.to_string());
                        Ok(vec![SectionItem2::new(0, SectionItem::Instruction(r))])
                    }
                    else {  //the imm can be a number which is a valid offset value
                        Ok(vec![])   //return empty means current function does not process the instruction.
                    }
                }
                else {
                    Err(AsmError::GeneralError((file!(), line!()).into(), format!("cannot find imm value in the instruction")))
                }
            }
            _ => {
                Ok(vec![])   //return empty means current function does not process the instruction.
            }
        }
    }

    fn get_inc_from_pseudoinc_call(item:&SectionItem2, label_offset:&LabelOffsetTable) -> Result<Vec<SectionItem2>, AsmError> {
        if !item.is_pseodu_inc() {
            return Err(AsmError::WrongType((file!(), line!()).into(), format!("Can only convert a non-pesuedo code. The current code is {item:?}")))
        }
        
        let inc = item.get_inc().unwrap();
        let current_offset = item.get_offset();
        match inc.name.to_lowercase().as_str() {
            "call" => {
                if let Some(label) = inc.get_imm().and_then(|x| x.get_value()) {
                    if let Some(target_offset) = label_offset.get_single_label_offset(&label, None) {
                        let offset = target_offset as i64 - current_offset as i64 - 4;
                        if Instruction::is_within_1m(offset) {
                            let (inc_name, inc_type) = Instruction::get_inc_and_inc_type("jal")?;
                            let mut r = Instruction::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions);
                            r.set_r0_value("ra");
                            r.set_imm_value(&label);
                            Ok(vec![SectionItem2::new(0, SectionItem::Instruction(r))])
                        }
                        else {
                            let (inc_name, inc_type) = Instruction::get_inc_and_inc_type("auipc")?;
                            let mut r = Instruction::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions);
                            r.r0_name = Some("ra".to_string());
                            r.set_imm(Some(label.as_str().into()));
                            r.rel_fun = Some("%pcrel_hi".to_string());

                            let (inc_name2, inc_type2) = Instruction::get_inc_and_inc_type("jalr")?;
                            let mut r2 = Instruction::new(inc_name2, inc_type2, BasicInstructionExtensions::BaseIntegerInstructions);
                            r2.r0_name = Some("ra".to_string());
                            r2.r1_name = Some("ra".to_string());
                            r2.set_imm(Some(label.as_str().into()));
                            r2.rel_fun = Some("%pcrel_lo".to_string());
                            Ok(vec![SectionItem2::new(0, SectionItem::Instruction(r)), SectionItem2::new(0, SectionItem::Instruction(r2))])
                        }
                    }
                    else {
                        // cannot find label, but this label can be external label
                        let (inc_name, inc_type) = Instruction::get_inc_and_inc_type("auipc")?;
                        let mut r = Instruction::new(inc_name, inc_type, BasicInstructionExtensions::BaseIntegerInstructions);
                        r.r0_name = Some("ra".to_string());
                        r.set_imm(Some(label.as_str().into()));
                        r.rel_fun = Some("%pcrel_hi".to_string());

                        let (inc_name2, inc_type2) = Instruction::get_inc_and_inc_type("jalr")?;
                        let mut r2 = Instruction::new(inc_name2, inc_type2, BasicInstructionExtensions::BaseIntegerInstructions);
                        r2.r0_name = Some("ra".to_string());
                        r2.r1_name = Some("ra".to_string());
                        r2.set_imm(Some(label.into()));
                        r2.rel_fun = Some("%pcrel_lo".to_string());
                        Ok(vec![SectionItem2::new(0, SectionItem::Instruction(r)), SectionItem2::new(0, SectionItem::Instruction(r2))])
                    }
                }
                else {
                    Err(AsmError::GeneralError((file!(), line!()).into(), format!("cannot find imm value in the instruction")))
                }
            }
            _ => {
                error_string(format!(""));
                Err(AsmError::NoFound((file!(), line!()).into(), format!("need to add process logic for {inc:?} when convert psudeo code")))
            }
        }
    }

    fn replace_equ(item:&mut SectionItem2, equals:&EquTable, config:&mut CodeGenConfiguration) -> Result<(), AsmError> {
        if let Some(inc) = item.get_inc_mut() {
            if let Some(imm_value) = inc.get_imm().and_then(|x| x.get_value()) {
                if equals.contains_key(&imm_value) {
                    let new_value = equals[&imm_value].to_string();
                    inc.set_imm(Some(new_value.into()));
                }
            }
            else if let Some(macro_value) = inc.get_imm().and_then(|x| x.get_macro_value()) {
                let imm = match macro_value { 
                    ImmMacro::PtrSize => *config.get_build_target(),
                };
                
                inc.set_imm(Some(imm.into()))
            }
        }

        Ok(())
    }

    /// update the section's offset value
    /// this function also process the alignment for the item in data section
    /// it returns the total length of the section
    pub (crate) fn update_section(&mut self, section_type:SectionType, offset: usize) -> usize {
        let mut length = 0;
        let sections = self.sections.iter_mut().filter(|x| x.get_section_type() == section_type);
        let mut current_section_offset = offset;
        for section in sections {
            section.set_offset(current_section_offset);
            section.update_offset();
            length += section.get_section_length();
            current_section_offset += section.get_section_length();
        }

        length
    }

    fn get_machine_code_from_data(&self, section_type:SectionType) -> Result<Vec<MachineCode>, AsmError> {
        let mut data_bin = Vec::default();
        let sections = self.sections.iter().filter(|x| x.get_section_type() == section_type);
        for section in sections {
            for item2 in section.get_all_items() {
                let item = item2.get_item();
                let offset = item2.get_offset();
                match item {
                    SectionItem::Lable(_n) => {
                        //debug_string(format!("label '{n}' @ {offset}/0x{offset:X}"));
                        // labels are not processed here, they are used for linking and debugging
                    },
                    SectionItem::Instruction(_) => return Err(AsmError::GeneralError((file!(), line!()).into(), format!("instruction cannot be processed here. It has other function."))),
                    SectionItem::Directive(n) => {
                        if let Some(data) = n.get_machine_code() {
                            //debug_string(format!("Directive Generation: {n:?} => {data:?} @ {offset}/0x{offset:X}"));
                            data_bin.push(data)
                        }
                        else {
                            warn_string(format!("ignore generating machine code from {n:?} @ {offset}/0x{offset:X}"));
                        }
                    }
                }
            }
        }

        Ok(data_bin)
    }

    /// this function is to generate binary and work as linker function
    pub (crate) fn link_to_bin(&mut self, file_path:&str, config:&mut CodeGenConfiguration) -> Result<(), AsmError> {
        let data = self.generate_binary(config)?;

        std::fs::write(file_path, data)
                .map_err(|_| { AsmError::GeneralError((file!(), line!()).into(), format!("cannot write to file")) })
    }

    /// update the virtual address for all incs in the text section, 
    /// the virtual address is real value after consider label, register and external symbols
    /// this value is used to generate the final binary code
    pub (crate) fn update_label_virtual_address(&mut self, dynamic_structure:Option<&ELFDynamicStructure>) -> Result<(), AsmError> {
        let mut external_symbols = self.get_external_symbols();

        // update external symbol virtual address
        if let Some(ds) = dynamic_structure {
            for ext in external_symbols.iter_mut() {
                ext.update_value(ds)?;                
            }
        }

        let regs = Register::new();
        let labels = self.get_labels().into();
        let all_sections = &mut self.sections;
        let sections = all_sections.iter_mut().filter(|x| x.get_section_type() == SectionType::Text);
        for section in sections {
            for inc in section.get_all_items_mut().iter_mut().filter(|x| x.is_inc()) {
                let virtual_offset = inc.get_imm_value(&labels, &regs, &external_symbols)?; 
                if let Some(instruction) = inc.get_inc_mut() {
                    instruction.set_virtual_address(virtual_offset);
                }
            }
        }

        Ok(())
    }

    /// get all external symbols defined in the program
    pub fn get_external_symbols(&self) -> Vec<ExternalLabel> {
        let mut r = Vec::default();
        for section in self.sections.iter() {
            for item in section.get_all_items() {
                if let SectionItem::Directive(n) = item.get_item() {
                    if let Some(imm) = n.get_extern_label() {
                        r.push(imm.into());
                    }
                }
            }
        }

        r
    }

    /// write debug asm code to file text_section_debug.s
    /// this function is mainly for debug purpose
    fn generate_debug_asm_code(&self, config:&mut CodeGenConfiguration) -> Result<(), AsmError> {
        let labels = self.get_labels();
        let regs = Register::new();

        if !config.get_generate_bin_and_code() {
            return Ok(())
        } 

        let mut code = Vec::default();
        let text_sections = self.sections.iter()
                                         .filter(|x| x.get_section_type() == SectionType::Text)
                                         .collect::<Vec<_>>();
        for section in &text_sections {
            for inc in section.get_instructions() {
                let machine_codes = self.get_machine_code_list(inc, &regs, &labels)?;
                let machine_code_str = machine_codes.iter().map(|x| format!("{v:?}", v=x.get_code_data())).collect::<Vec<_>>();
                let external_label_str = if inc.get_external_symbol().is_some() { inc.get_external_symbol().unwrap().to_string() } else { String::new() };
                let line = format!("0x{:X}: {}\t\t# {} {} imm/offset = {}", 
                            inc.get_offset(), 
                            inc.get_item().generate_code_string(), 
                            external_label_str, 
                            machine_code_str.join(" | "),
                            inc.get_virtual_address().map(|x| format!("0x{:X}", x)).unwrap_or_else(|| "N/A".to_string()));
                code.push(line);
            }
        }

        if !code.is_empty() {
            // add lablels to the end of the text section debug file
            code.push(format!("\r\n\r\n#####"));
            code.push(format!("# label list"));
            code.push(format!("#####\r\n"));
            for section in text_sections {
                for item in section.get_all_items() {
                    if item.is_label() {
                        let label = item.get_label().unwrap();
                        let line = format!("# {} @ 0x{:X}", label, item.get_offset());
                        code.push(line);
                    }
                }
            }
            code.push(format!("\r\n\r\n#####"));

            code.push("\n\n".to_string());
            code.push(labels.to_markdown());

            let _ = write_to_file("text_section_debug.s", & code.join("\r\n"));
        }

        Ok(())
    }

    /// generate segment headers and set the virtual address for each section
    /// the virtual address is used to generate the final binary code
    /// this function also process alignment for the item in data section
    fn generate_program_headers(&mut self, config:&mut CodeGenConfiguration) -> Result<SegmentHeaderList, AsmError> {
        let va_offset = config.get_linker_config().get_virutual_address_start() as u64;
        let mut segment_headers = SegmentHeaderList::default();

        //iterate all sections to mark section offset, in the order of 
        // all .text sections, data, bss, rodata.
        let offset_init = 0x1000;
        let mut offset = offset_init;   //this address must be equal to the virutal address defined below
        let virtual_address_unit = 0x1000;

        let segment_size = self.update_section(SectionType::Text, offset + va_offset as usize);
        let text_segment = ProgramHeader::new3(SectionType::Text, offset as u64, offset as u64 + va_offset, segment_size as u64);
        segment_headers.add((text_segment, SectionType::Text).into());

        offset = round_to_usize(offset + segment_size, virtual_address_unit);
        let segment_size = self.update_section(SectionType::Data, offset + va_offset as usize);
        let data_segment = ProgramHeader::new3(SectionType::Data, offset as u64, offset as u64 + va_offset, segment_size as u64);
        segment_headers.add((data_segment, SectionType::Data).into());

        offset = round_to_usize(offset + segment_size, virtual_address_unit);
        let segment_size = self.update_section(SectionType::Bss, offset + va_offset as usize);
        let bss_segment = ProgramHeader::new_bss(offset as u64, offset as u64 + va_offset, segment_size as u64);
        segment_headers.add((bss_segment, SectionType::Bss).into());

        offset = round_to_usize(offset + segment_size, virtual_address_unit);
        let segment_size = self.update_section(SectionType::Readonlydata, offset + va_offset as usize);
        let ro_segment = ProgramHeader::new3(SectionType::Readonlydata, offset as u64, offset as u64 + va_offset, segment_size as u64);
        segment_headers.add((ro_segment, SectionType::Readonlydata).into());

        // note header
        let note_segment = config.get_note_section();
        offset = segment_headers.len() * ProgramHeader::SERIALIZED_SIZE as usize + ELFHeader::SERIALIZED_SIZE; // +2 is for phdr and phdr for phdr segment
        let note_segment_header = ProgramHeader::new_note(offset as u64, offset as u64, note_segment.get_size_in_bytes() as u64, 0x4);
        segment_headers.add((note_segment_header, SectionType::Note).into());

        Ok(segment_headers)
    }

    /// this function performs two actions:
    /// - generate segment headers 
    /// - and set the virtual address for each section in the dynamic structure
    fn generate_dynamic_program_headers(&mut self, config:&mut CodeGenConfiguration, dynamic_struture:&mut ELFDynamicStructure) 
        -> Result<SegmentHeaderList, AsmError> 
    {
        let linker_config = config.get_linker_config();
        let va_offset = linker_config.get_virutual_address_start() as u64;
        let mut segment_headers = SegmentHeaderList::default();

        //iterate all sections to mark section offset, in the order of 
        // all .text sections, data, bss, rodata.
        let offset_init = 0x1000;
        let mut offset = offset_init;   //this address must be equal to the virutal address defined below
        let virtual_address_unit = 0x1000;

        // text segment contains code and plt section size
        let text_segment_size = self.update_section(SectionType::Text, offset + va_offset as usize); 
        dynamic_struture.set_text_segment_address((offset + text_segment_size) as u64, linker_config); // the address of text segment is after the text section
        let segment_size = text_segment_size + dynamic_struture.get_text_segment_size();
        let text_segment = ProgramHeader::new3(SectionType::Text, offset as u64, offset as u64 + va_offset, segment_size as u64);
        segment_headers.add(text_segment.into());

        // data segment contains data, dyn entry list size, and got section size
        offset = round_to_usize(offset + segment_size, virtual_address_unit);  
        let data_segment_size = self.update_section(SectionType::Data, offset + va_offset as usize);
        dynamic_struture.set_data_segment_address((offset + data_segment_size) as u64, linker_config); // the address of data segment is after the data section    
        let segment_size = data_segment_size + dynamic_struture.get_data_segment_size();
        let data_segment = ProgramHeader::new3(SectionType::Data, offset as u64, offset as u64 + va_offset, segment_size as u64);
        segment_headers.add((data_segment, SectionType::Data).into());

        offset = round_to_usize(offset + segment_size, virtual_address_unit);
        let segment_size = self.update_section(SectionType::Bss, offset + va_offset as usize);
        let bss_segment = ProgramHeader::new_bss(offset as u64, offset as u64 + va_offset, segment_size as u64);
        segment_headers.add((bss_segment, SectionType::Bss).into());

        // rodata segment contains read only data, dynamic symbol table, dynamic string table, rela.plt, interp is also embedded in rodata segment
        offset = round_to_usize(offset + segment_size, virtual_address_unit);
        let intrep_length = dynamic_struture.get_interp().unwrap_or_default().len() + 1;  // +1 for null terminator
        let rosegment_size = self.update_section(SectionType::Readonlydata, offset + va_offset as usize);
        dynamic_struture.set_rosections_address((offset + rosegment_size) as u64, linker_config); // the address of rodata segment is after the rodata section
        let segment_size = rosegment_size + dynamic_struture.get_readonly_segment_size();
        let ro_segment = ProgramHeader::new3(SectionType::Readonlydata, offset as u64, offset as u64 + va_offset, segment_size as u64);
        segment_headers.add((ro_segment, SectionType::Readonlydata).into());

        // generate interp segment
        let interp_offset = dynamic_struture.get_interp_offset(); 
        let interp_segment = ProgramHeader::new_interp(interp_offset as u64, interp_offset as u64 + va_offset, intrep_length as u64, intrep_length as u64);
        segment_headers.add((interp_segment, SectionType::Interp).into());

        //dynamic headers
        let segment_size = dynamic_struture.get_section_data_size() as u64;
        let dynamic_segment = ProgramHeader::new_dynamic(dynamic_struture.get_offset(), 
                        dynamic_struture.get_virtual_address(), 
                    segment_size, 
                            dynamic_struture.get_alignment().into());
        segment_headers.add((dynamic_segment, SectionType::Dynamic).into());

        // note header
        let note_segment = config.get_note_section();
        offset = segment_headers.len() + 2 * ProgramHeader::SERIALIZED_SIZE as usize + ELFHeader::SERIALIZED_SIZE; // +2 is for phdr and phdr for phdr segment
        let note_segment_header = ProgramHeader::new_note(offset as u64, offset as u64, note_segment.get_size_in_bytes() as u64, 0x4);
        segment_headers.add((note_segment_header, SectionType::Note).into());

        // add phdr tells loader where the program header table itself is located, mandatory for dynamic elf
        let segments_size = (segment_headers.len() + 1) * ProgramHeader::SERIALIZED_SIZE ; // +1 is the phdr segment itself
        let phdr_segment = ProgramHeader::new_phdr(segments_size as u64, va_offset);
        let segment_for_phdr = ProgramHeader::new_rodata_for_phdr(phdr_segment.get_file_size() + Elf64Header::SERIALIZED_SIZE as u64, va_offset);
        segment_headers.insert(0, (segment_for_phdr, SectionType::Phdrsegment).into());
        segment_headers.insert(0, (phdr_segment, SectionType::Phdr).into()); //phdr is always the first segment

        Ok(segment_headers)
    }

    /// get entry address, with assumption that the text segment is always the 1st segmen and entry address is from the segment offset
    /// if no entry address is defined, return 0
    pub (crate) fn get_entry_address2(&self) -> usize {
        let labels = self.get_labels();
        let entry_address = labels.get_entry_address()
                            .unwrap_or(0);
        entry_address
    }

    /// generate non-dynamic elf file
    fn generate_non_dynamic_elf(&self, code_bin:TextSection, segment_headers:&SegmentHeaderList, config:&mut CodeGenConfiguration) -> Result<ElfFile, AsmError> {
        type ELFDataSection = super::elf_section::DataSection;

        // generate data sections
        let data_bin_data : ELFDataSection = self.get_machine_code_from_data(SectionType::Data)?.into();
        let rodata_bin_data : ROSection = self.get_machine_code_from_data(SectionType::Readonlydata)?.into();

        // conslidate all data into byte vec
        let mut elf_file = ElfFile::new();
        elf_file.set_entry_point(self.get_entry_address2() as u64);
        let text_segment = & segment_headers[SectionType::Text];
        let data_segment = & segment_headers[SectionType::Data];
        let bss_segment = & segment_headers[SectionType::Bss];
        let ro_segment = & segment_headers[SectionType::Readonlydata];

         // add program headers
        if text_segment.get_memory_size() > 0 {
            let code: CodeSection = code_bin.into();
            elf_file.add_code_section(code, text_segment.get_virtual_address());
        }
        if data_segment.get_memory_size() > 0 {
            elf_file.add_data_section(data_bin_data.into(), data_segment.get_virtual_address());
        }
        if bss_segment.get_memory_size() > 0 {
            elf_file.add_bss_section(bss_segment.get_virtual_address(), bss_segment.get_memory_size());
        }
        if ro_segment.get_memory_size() > 0 {
            elf_file.add_read_only_section(rodata_bin_data.into(), ro_segment.get_virtual_address());
        }

        // add note section
        // note section is added to the beginning of the file
        // because loader won't load the note section into memory, so it won't cause problem for the virtual address of other section, 
        // and it can make sure the note section is always at a fixed location in the file which is easier for linker to find and update
        let offset = (segment_headers.len() * ProgramHeader::SERIALIZED_SIZE) as u64 + ELFHeader::SERIALIZED_SIZE as u64; // after all program headers
        elf_file.add_note_section(config.get_note_section().clone(), offset);

        elf_file.save_md_file("linker_debug.md").unwrap();
        Ok(elf_file)
    }

    /// generate and dynamic elf file based on program/segment headers
    /// this process also set the virtual address for each segement which combines all the sections for the segment
    fn generate_dynamic_elf(&self, dynamic_structure:&mut ELFDynamicStructure, code_bin :TextSection, segment_headers:&SegmentHeaderList, config:&mut CodeGenConfiguration) 
        -> Result<ElfFile, AsmError> 
    {
        type ELFDataSection = super::elf_section::DataSection;

        // generate data sections
        let data_bin_data : ELFDataSection = self.get_machine_code_from_data(SectionType::Data)?.into();
        let rodata_bin_data : ROSection = self.get_machine_code_from_data(SectionType::Readonlydata)?.into();

        // conslidate all data into byte vec
        let mut elf_file = ElfFile::new();
        let mut section_structure = ElfSectionStructure::default();

        elf_file.set_to_dynamic();
        elf_file.set_entry_point(self.get_entry_address2() as u64);
        let text_segment = & segment_headers[SectionType::Text];
        let data_segment = & segment_headers[SectionType::Data];
        let bss_segment = & segment_headers[SectionType::Bss];
        let ro_segment = & segment_headers[SectionType::Readonlydata];
        let interp_segment = & segment_headers[SectionType::Interp];
        let dynamic_section = & segment_headers[SectionType::Dynamic];
        let phdr_segment = & segment_headers[SectionType::Phdr];
        let phdr_readonly_segment = & segment_headers[SectionType::Phdrsegment];

        elf_file.add_phdr_section(phdr_segment);
        elf_file.add_segment_for_phdr(phdr_readonly_segment);

        if text_segment.get_memory_size() > 0 {
            let mut code: CodeSection = code_bin.into();
            section_structure.new_text_section_header(text_segment.get_file_offset(), text_segment.get_virtual_address(), code.get_size() as u64);
            dynamic_structure.generate_plt_code();
            code = code + dynamic_structure.to_text_segment_data();
            assert!(code.get_size() as u64 == text_segment.get_memory_size(), "code size {} must be equal to text segment size {}", code.get_size(), text_segment.get_memory_size());
            elf_file.add_code_section(code, text_segment.get_virtual_address());
        }
        if data_segment.get_memory_size() > 0 {
            let mut data :DataSection = data_bin_data.into();
            section_structure.new_data_section_header(data_segment.get_file_offset(), data_segment.get_virtual_address(), data.get_size() as u64);
            data = data + dynamic_structure.to_data_segment_data();
            assert!(data.get_size() as u64 == data_segment.get_memory_size(), "data size {} must be equal to data segment size {}", data.get_size(), data_segment.get_memory_size());
            elf_file.add_data_section(data, data_segment.get_virtual_address());            
        }
        if bss_segment.get_memory_size() > 0 {
            elf_file.add_bss_section(bss_segment.get_virtual_address(), bss_segment.get_memory_size());
        }
        if ro_segment.get_memory_size() > 0 {
            let mut code : ReadOnlySection = rodata_bin_data.into();
            section_structure.new_rodata_section_header(ro_segment.get_file_offset(), ro_segment.get_virtual_address(), code.get_size() as u64);
            code = code + dynamic_structure.to_readonly_segment_data();
            assert!(code.get_size() as u64 == ro_segment.get_memory_size(), "rodata size {} must be equal to rodata segment size {}", code.get_size(), ro_segment.get_memory_size());
            elf_file.add_read_only_section(code, ro_segment.get_virtual_address());
        }
        if interp_segment.get_memory_size() > 0 {
            elf_file.add_interpreter_section(&interp_segment);
        }
        if dynamic_section.get_memory_size() > 0 {
            elf_file.add_dynamic_section(dynamic_section);
        }

        let offset = (segment_headers.len() * ProgramHeader::SERIALIZED_SIZE) as u64 + ELFHeader::SERIALIZED_SIZE as u64; // after all program headers
        elf_file.add_note_section(config.get_note_section().clone(), offset);

        // add dynamic related section and then add section related data to elf file
        section_structure.set_section_offset(segment_headers.get_farest_offset() as usize);
        section_structure <<= dynamic_structure;
        section_structure.update();
        elf_file <<= &section_structure;

        elf_file.save_md_file("linker_debug.md").unwrap();
        Ok(elf_file)
    }

    /// get all directives defined in the program
    pub fn get_all_directives(&self) -> Vec<&Directive> {
        let mut r = Vec::default();
        for section in self.sections.iter() {
            for item in section.get_all_items() {
                if let SectionItem::Directive(n) = item.get_item() {
                    r.push(n);
                }
            }
        }

        r
    }

    /// get all global function symbol data defined in the program: name, offset, and size
    pub fn get_global_function_symbol_data(&self) -> Vec<(String, isize, usize)> {
        let mut r = Vec::default();
        for section in self.sections.iter() {
            let label_table = section.get_label_offset_table();
            for glb_item in label_table.get_global_labels() {
                let name = glb_item.get_label().name().to_string();
                let offset = glb_item.get_offset() as isize;
                let size = glb_item.get_symbol_size();
                r.push((name, offset, size));
            }
        }

        r
    }

    /// check if the program need to generate dynamic structure
    fn need_dynamic_structure(&self) -> bool {
        self.contains_external_symbol() ||
        self.contains_global_directive()
    }

    fn refresh_label_tables(&mut self) {
        for section in self.sections.iter_mut() {
            section.create_label_and_offset_table();
        }
    }

    fn generate_binary_with_dynamic_structure(&mut self, config:&mut CodeGenConfiguration) -> Result<Vec<u8>, AsmError> {
        let mut dynamic_structure = ELFDynamicStructure::default();
        dynamic_structure.insert_external_functions(self, config);
        dynamic_structure.add_global_functions(self);
        dynamic_structure.create_hash_section();

        let segment_headers = self.generate_dynamic_program_headers(config, &mut dynamic_structure)?;
        self.update_label_virtual_address(Some(&dynamic_structure))?;
        self.refresh_label_tables();

        for (symbol_name, offset, size) in self.get_global_function_symbol_data() {
            dynamic_structure.update_global_function_symbol(&symbol_name, offset as u64, size);
        }

        let code_bin = self.get_code_bin()?;
        self.generate_debug_asm_code(config)?;

        dynamic_structure.enrich_symbols(&segment_headers);
        let elf_file = self.generate_dynamic_elf(&mut dynamic_structure, code_bin, &segment_headers, config)?;
        debug_string(format!("Dynamic Structure: {dynamic_structure:?}\r\n"));

        Ok(elf_file.to_bytes())
    }

    fn generate_binary_without_dynamic_structure(&mut self, config:&mut CodeGenConfiguration) -> Result<Vec<u8>, AsmError> {
        let segment_headers = self.generate_program_headers(config)?;
        self.update_label_virtual_address(None)?;
        self.refresh_label_tables();

        let code_bin = self.get_code_bin()?;
        self.generate_debug_asm_code(config)?;

        let elf_file = self.generate_non_dynamic_elf(code_bin, &segment_headers, config)?;
        debug_string(format!("No external symbol found, generate a non-dynamic elf file\r\n"));

        Ok(elf_file.to_bytes())
    }

    /// this is the linker function and it generates the binary
    pub (crate) fn generate_binary(&mut self, config:&mut CodeGenConfiguration) -> Result<Vec<u8>, AsmError> {
        if self.need_dynamic_structure() {
            self.generate_binary_with_dynamic_structure(config)
        } else {
            self.generate_binary_without_dynamic_structure(config)
        }
    }

    fn get_code_bin(&self) -> Result<TextSection, AsmError> {
        let mut code_bin = Vec::default();
        let regs = Register::new();
        let labels = self.get_labels();

        for section in self.get_txt_sections() {
            for inc in section.get_instructions() {                
                let mut machine_codes = self.get_machine_code_list(inc, &regs, &labels)?;
                code_bin.append(&mut machine_codes)
            }
        }

        Ok(code_bin.into())
    }

    /// get all sections with type TEXT
    pub (crate) fn get_txt_sections(&self) -> Vec<&Section> {
        self.sections.iter()
            .filter(|x| x.get_section_type() == SectionType::Text)
            .collect::<Vec<_>>()
    }

    /// get all section with type TEXT mut 
    pub (crate) fn get_txt_sections_mut(&mut self) -> Vec<&mut Section> {
        self.sections.iter_mut()
            .filter(|x| x.get_section_type() == SectionType::Text)
            .collect::<Vec<_>>()
    }

    pub (crate) fn get_labels(&'_ self) -> LabelTable<'_> {
        let mut r = LabelTable::default();
        let mut id = 0;
        for section in self.sections.iter() {
            let lables = section.get_labels_and_next_items();
            for (label, inc) in lables.into_iter() {
                if let Some(s) = label.get_label() {
                    let related_label = inc.and_then(|x| x.get_related_label()).unwrap_or_default();
                    let key = LabelOffsetTableEntry::new(s.into(), label.get_offset(), id, &related_label);
                    id += 1;
                    r.add(key, label);
                } else if let Some(s2) = label.get_external_label() {
                    let key = LabelOffsetTableEntry::new(s2.into(), label.get_offset(), id, "");
                    id += 1;
                    r.add(key, label);
                }
            }
        }

        for section in self.get_txt_sections() {
            for item in section.get_all_items() {
                if let Some(inc) = item.get_inc() {
                    if inc.is_pcrel_hi() {
                        let label = inc.get_rel_fun().unwrap();
                        let related_label = inc.get_imm().unwrap().get_value().unwrap();
                        let key = LabelOffsetTableEntry::new(label.into(), item.get_offset(), id, &related_label);
                        id += 1;
                        r.add(key, item);
                    }
                }
            }
        }

        r
    }

    pub (crate) fn get_equals(&self) -> EquTable {
        let mut r = HashMap::default();

        for section in self.sections.iter() {
            let hash = section.get_equ_list();
            for key in hash.keys() {
                r.insert(key.to_string(), hash[key].to_string());
            }
        } 

        r
    }

    fn substring(input:&str, start_index:usize, length:usize) -> String {
        let end_index = start_index + length;
        let r = &input[start_index..end_index];
        r.to_string()
    }

    fn reverse(input:&str) -> String {
        reverse_string(input)
    }

    pub fn get_machine_code_list(&self, item:&SectionItem2, regs:&Register, labels:&LabelTable) -> Result<Vec<MachineCode>, AsmError> {
        let inc = item.get_inc().unwrap();
        let inc_offset = item.get_offset();

        if !inc.is_generate {
            return Ok(Vec::default());
        }

        if inc.inc_extensions_and_type == BasicInstructionExtensions::RvVInstructions {
            return emit_vector_instruction(inc, regs, inc_offset);
        }

        // debug_string(format!("Generate binary for {inc:?} @ 0x{:x}", item.offset));
        let op_code = inc.get_op_code()?;
        let inc_type = op_code.get_instruction_type();
        let op_code_str = format!("{:0>7b}", op_code.get_value());
        let rr = match inc_type {
            InstructionTypes::B => {
                let rs1 = inc.get_register_id_as_string(InstructionRegisterName::Rs0, regs)?;
                let rs2 = inc.get_register_id_as_string(InstructionRegisterName::Rs1, regs)?;

                assert!(inc.get_imm().is_some());  //branch instruction must have imm value

                //if imm string has been replaced as number, it means the offset is calcuated at previous stage
                // otherwise, lable value will be calculated here
                let imm = if let Some(inc_imm_value) = inc.get_imm_value_from_imm_string() {
                    assert!(inc.label_virtual_address as i64 == item.get_offset() as i64 + inc_imm_value as i64);  //label virtual address must be the same as the current offset + imm value
                    inc_imm_value as u32
                }
                else {
                    let label = inc.get_imm().and_then(|x| x.get_value()).unwrap();
                    let label_offset = labels.get(&label).unwrap().get_offset();
                    (label_offset as i64 - item.get_offset() as i64) as u32
                };

                let imm_len = 13;
                let mut imm_str = format!("{imm:0>imm_len$b}");
                if imm_str.len() > imm_len {
                    imm_str = Self::substring(&imm_str, imm_str.len()-imm_len, imm_len);
                }

                let imm11 = Self::substring(&imm_str, 1, 1);
                let imm4_1 = Self::substring(&imm_str, imm_str.len() - 5, 4); //imm_str length set to 13, that's why need to -5 not -4
                let funct3 = if let Some(n) = inc.get_option_value_str() { n } 
                                     else { op_code.get_funct3_str()? };
                let imm10_5 = Self::substring(&imm_str, 2, 6);
                let imm12 = Self::substring(&imm_str, 0, 1);
                let machine_code_str = format!("{imm12}{imm10_5}{rs2}{rs1}{funct3}{imm4_1}{imm11}{op_code_str}");
                if machine_code_str.len() > 32 {
                    return Err(AsmError::ConversionFailed((file!(), line!()).into(), format!("machine code {machine_code_str} is too long to convert in B inc type")));
                }
                let machine_code = MachineCode::from_string(&machine_code_str, inc_offset)?;
                Ok([machine_code].to_vec())
            }
            InstructionTypes::I => {
                let rd = inc.get_register_id_as_string(InstructionRegisterName::Rd, regs)?;
                let funct3 = if let Some(n) = inc.get_option_value_str() { n } 
                                     else { op_code.get_funct3_str()? };
                let imm_len = 12;

                if op_code.is_control_register_as_imm() {
                    let rs1 = 
                        if inc.r2_name.is_some() { 
                            inc.get_register_id_as_string(InstructionRegisterName::Rs2, regs)?  //reg in rs2 location will be treated as rs1
                        }
                        else {
                            format!("{:0>5b}", inc.imm_to_u32()?)
                        };
                    let value = u32::from_str_radix(&inc.get_register_id_as_string(InstructionRegisterName::Rs1, regs)?, 2).unwrap(); //reg in rs1 location is the control reg and will serve as imm
                    let mut imm_str = format!("{value:0>imm_len$b}");

                    if imm_str.len() > imm_len {
                        imm_str = Self::reverse(&imm_str);
                        imm_str = Self::reverse(&imm_str[0..imm_len].to_string());
                    }

                    let machine_code_str = format!("{imm_str}{rs1}{funct3}{rd}{op_code_str}");
                    if machine_code_str.len() > 32 {
                        debug_string(format!("imm_str = {imm_str} with len = {}", imm_str.len()));
                        return Err(AsmError::ConversionFailed((file!(), line!()).into(), format!("machine code {machine_code_str} with len = {} is too long to convert in I inc type", machine_code_str.len())));
                    }
                    let machine_code = MachineCode::from_string(&machine_code_str, inc_offset)?;                
                    let r = [machine_code].to_vec();
                    Ok(r)
                }
                else {                    
                    let rs1 = inc.get_register_id_as_string(InstructionRegisterName::Rs1, regs)?;
                    let value = inc.label_virtual_address;
                    let mut imm_str = format!("{value:0>imm_len$b}");
                    
                    if imm_str.len() > imm_len {
                        imm_str = Self::reverse(&imm_str);
                        imm_str = Self::reverse(&imm_str[0..imm_len].to_string());
                    }

                    let machine_code_str = format!("{imm_str}{rs1}{funct3}{rd}{op_code_str}");
                    if machine_code_str.len() > 32 {
                        debug_string(format!("imm_str = {imm_str} with len = {}", imm_str.len()));
                        return Err(AsmError::ConversionFailed((file!(), line!()).into(), format!("machine code {machine_code_str} with len = {} is too long to convert in I inc type", machine_code_str.len())));
                    }
                    let machine_code = MachineCode::from_string(&machine_code_str, inc_offset)?;                
                    let r = [machine_code].to_vec();
                    Ok(r)
                }
            }
            InstructionTypes::J => {
                let rd = inc.get_register_id_as_string(InstructionRegisterName::Rd, regs)?;
                let imm = inc.label_virtual_address;
                let mut imm_str = format!("{imm:0>21b}");
                if imm_str.len() > 21 {
                    imm_str = Self::substring(&imm_str, imm_str.len()-21, 21);
                }

                imm_str = imm_str.chars().rev().collect::<String>();
                let imm20 = Self::reverse(&Self::substring(&imm_str, 20, 1));
                let imm19_12 = Self::reverse(&Self::substring(&imm_str, 12, 8));
                let imm11 = Self::reverse(&Self::substring(&imm_str, 11, 1));
                let imm10_1 = Self::reverse(&Self::substring(&imm_str, 1, 10));          
                let machine_code_str = format!("{imm20}{imm10_1}{imm11}{imm19_12}{rd}{op_code_str}");
                if machine_code_str.len() > 32 {
                    return Err(AsmError::ConversionFailed((file!(), line!()).into(), format!("machine code {machine_code_str} is too long to convert in J inc type")));
                }
                let r = MachineCode::from_string(&machine_code_str, inc_offset)?;
                Ok([r].to_vec())
            }
            InstructionTypes::R => {
                let rd = inc.get_register_id_as_string(InstructionRegisterName::Rd, regs)?;
                let r1 = inc.get_register_id_as_string(InstructionRegisterName::Rs1, regs)?;
                let r2 = inc.get_register_id_as_string(InstructionRegisterName::Rs2, regs)?;
                let funct3 = if let Some(n) = inc.get_option_value_str() { n } 
                                     else { op_code.get_funct3_str()? };
                let funct7 = 
                    if inc.has_reg(InstructionRegisterName::Rs3) {  //if Rs3 present, the Rs3 will be placed in the funct7 place
                        let reg_id = regs.get_register_value(inc.r3_name.as_ref())? as u32;
                        if let Some(funct7) = op_code.get_funct7() {
                            let v = (funct7 as u32) | (reg_id << 2);     //move reg_id left,those 2 bits are fmt format and set previously in opcode side
                            format!("{v:0>7b}") 
                        }
                        else {
                            return Err(AsmError::NoFound((file!(), line!()).into(), format!("cannot find funct7 from {op_code:?} when there is R3 present")))
                        }
                    } 
                    else { 
                        if inc.is_atomic() {
                            let v = inc.get_atomic_option()?;
                            if let Some(funct7) = op_code.get_funct7() {
                                let y = (funct7 as u32) & ((! 0b11) | v);
                                format!("{y:0>7b}")
                            }
                            else {
                                return Err(AsmError::NoFound((file!(), line!()).into(), format!("cannot find funct7 from {op_code:?} for atomic operation")))
                            }
                        }
                        else {
                            op_code.get_funct7_str()? 
                        }
                    };
                let machine_code_str = format!("{funct7}{r2}{r1}{funct3}{rd}{op_code_str}");
                if machine_code_str.len() > 32 {
                    return Err(AsmError::ConversionFailed((file!(), line!()).into(), format!("machine code {machine_code_str} is too long to convert in R inc type")));
                }
                let r = [MachineCode::from_string(&machine_code_str, inc_offset)?].to_vec();
                Ok(r)
            }
            InstructionTypes::S => {
                let imm = inc.label_virtual_address;
                let imm_len = 12;
                let mut imm_str = format!("{imm:0>imm_len$b}");
                if imm_str.len() > imm_len {
                    imm_str = Self::substring(&imm_str, imm_str.len()-imm_len, imm_len);
                }
                let imm4_0 = imm_str[imm_str.len() - 5..].to_string();
                let funct3 = if let Some(n) = inc.get_option_value_str() { n } 
                                     else { op_code.get_funct3_str()? };
                let r2 = inc.get_register_id_as_string(InstructionRegisterName::Rd, regs)?;
                let r1 = inc.get_register_id_as_string(InstructionRegisterName::Rs1, regs)?;
                let imm11_5 = Self::substring(&imm_str, 0, imm_str.len() - 5);
                let machine_code_str = format!("{imm11_5}{r2}{r1}{funct3}{imm4_0}{op_code_str}");
                if machine_code_str.len() > 32 {
                    return Err(AsmError::ConversionFailed((file!(), line!()).into(), format!("machine code {machine_code_str} is too long to convert in S inc type")));
                }
                let r = [MachineCode::from_string(&machine_code_str, inc_offset)?].to_vec();
                Ok(r)
            }
            InstructionTypes::U => {
                let rd = inc.get_register_id_as_string(InstructionRegisterName::Rd, regs)?;
                let imm_str_len = 20;
                let mut imm_str = format!("{:0>imm_str_len$b}", inc.label_virtual_address);
                if imm_str.len() > imm_str_len {
                    imm_str = Self::substring(&imm_str, imm_str.len()-imm_str_len, imm_str_len);
                }
                
                let machine_code_str = format!("{imm_str}{rd}{op_code_str}");
                if machine_code_str.len() > 32 {
                    return Err(AsmError::ConversionFailed((file!(), line!()).into(), format!("machine code {machine_code_str} is too long to convert in U inc type")));
                }
                let machine_code = u32::from_str_radix(machine_code_str.as_str(), 2)
                                .map_err(|_| AsmError::ConversionFailed((file!(), line!()).into(), format!("cannot convert {machine_code_str} to machine code in instruction type U")))?;
                let r = [MachineCode::new1(machine_code)].to_vec();
                Ok(r)
            }
            InstructionTypes::COMPACT => {
                let r = get_machine_code_from_compact_inc(inc, inc_offset, regs)?;
                Ok([r].to_vec())
            }
            InstructionTypes::UnKnown => {
                warn_string(format!("skip generate {item:?}"));
                Ok(Vec::default())
            }
        };

        // debug_string(format!("output binary = {rr:?}"));
        rr
    }

    pub fn merge(&mut self, asm:&mut Self) {
        self.sections.append(&mut asm.sections)
    }

    pub fn get_sections(&self) -> &Vec<Section> {
        &self.sections
    }
}
