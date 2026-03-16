use crate::r5asm::dynamic_structure::dynamic_symbol_entry::DynamicSymbolEntry;

use super::super::asm_program::*;
use super::super::{code_gen_config::CodeGenConfiguration, elf_file::{data_section::DataSection}, linker_config::LinkerConfig};
use super::super::traits::*;
use super::*;
use super::super::alignment::*;
use super::super::elf_section::*;
use super::dt_tags::*;
use super::super::elf_file::code_section::CodeSection;

use super::elf_string_table::ELFStringTable;
use core_utils::string::vna_hash;
use core::panic;
use std::fmt::{self, Debug, Formatter};

pub const DT_RELA: u64 = 7;           // Address of relocation entries with addends

// ELF special section indices
const SHN_ABS: u16 = 0xFFF1;

/// elf dynamic structure representation, it has many structure related to dynamic linking
#[derive(Clone)]
pub struct DynamicSection {
    virtual_address: u64,           // virtual address of dynamic section
    offset: u64,
    alignment: Alignment,           // alignment for dynamic entries
    entries: Vec<DynamicEntry>,     // .dynamic section data
}

impl Default for DynamicSection {
    fn default() -> Self {
        Self {
            virtual_address: 0,
            offset: 0,
            alignment: Alignment::new(16).unwrap(),
            entries: vec![],
        }
    }
}

/// elf dynamic structure representation, it has many structure related to dynamic linking
#[derive(Clone)]
pub struct ELFDynamicStructure {
    dynamic_section: DynamicSection,
    dynamic_symbol_table : ELFDynamicSymbolTable,   //.dynsym section data
    dynamic_string_table : DynamicStringTable,          //.dynstr section data
    plt_relocation_table : ELFPLTRelocationTable,   //.rela.plt section data
    plt_section : ELFPLTSection,                    //.plt section data
    got_plt_section : GOTSection,                   //.got section data
    gnu_hash_section : GnuHashSection,         //.gnu.hash section data
    interp_offset : u64,                         //interpreter path offset in the elf dynamic string table
    gnu_version_section : GnuVersionSection,   //.gnu.version section data
    gnu_version_required_section : GnuVersionRequiredSection, //.gnu.version_r section data
}

impl ELFDynamicStructure {
    pub fn get_plt_section(&self) -> &ELFPLTSection {
        &self.plt_section
    }

    pub fn get_dynamic_symbol_table(&self) -> &ELFDynamicSymbolTable {
        &self.dynamic_symbol_table
    }

    pub fn get_got_plt_section(&self) -> &GOTSection {
        &self.got_plt_section
    }

    pub fn get_plt_relocation_table(&self) -> &ELFPLTRelocationTable {
        &self.plt_relocation_table
    }

    pub fn get_dynamic_string_table(&self) -> &ELFStringTable {
        &self.dynamic_string_table.0
    }

    pub fn get_interp(&self) -> Option<String> {
        self.dynamic_string_table.0.get_string_from_file_offset(self.interp_offset)
    }

    pub fn get_interp_offset(&self) -> u64 {
        self.interp_offset
    }

    /// create gnu hash section based on dynamic symbol table and dynamic string table
    pub fn create_hash_section(&mut self) {
        let r = self.get_dynamic_symbol_table().into_with(&self.dynamic_string_table.0);
        self.gnu_hash_section = r;
    }

    /// find dynamic entry for required symbol or library
    fn find_dyn_dt_need_entry(&self, required_symbol_or_lib:&str) -> Option<&DynamicEntry> {
        let offset = self.dynamic_string_table.0.get_string_offset(required_symbol_or_lib)?;
        self.dynamic_section.entries.iter().find(|entry| entry.d_tag == DTTags::DT_NEEDED && entry.d_val == offset as u64)
    }

    /// add new dynamic entry for required symbol or library
    pub fn insert_required_lib(&mut self, required_symbol_or_lib:&str) {
        let same_entry_exists = self.find_dyn_dt_need_entry(required_symbol_or_lib).is_some();
        if same_entry_exists {
            return;
        }

        let offset = self.dynamic_string_table.0.get_string_offset_or_add(required_symbol_or_lib);
        self.dynamic_section.entries.push(DynamicEntry::new_dt_needed(offset as u64));
        self.update_string_table_size();
    }

    /// add dynamic entry size
    pub fn insert_dynamic_entry_size(&mut self, d_val:u64) {
        self.dynamic_section.entries.push(DynamicEntry::new(DTTags::DT_SYMENT, d_val));
    }

    /// add or update string table size entry based on dynamic string table
    pub fn update_string_table_size(&mut self) {
        if self.contains(DTTags::DT_STRSZ) {
            let index = self.dynamic_section.entries.iter().position(|x| x.d_tag == DTTags::DT_STRSZ).unwrap();
            self.dynamic_section.entries[index] = self.dynamic_string_table.0.to_dynamic_entry_string_table_size();
        } else {
            let entry = self.dynamic_string_table.0.to_dynamic_entry_string_table_size();
            self.dynamic_section.entries.push(entry);
        }        
    } 

    /// check if contain certain dynamic entry
    pub fn contains(&self, d_tag:DTTags) -> bool {
        self.dynamic_section.entries.iter().any(|x| x.d_tag == d_tag)
    }

    /// get size of entries in bytes
    pub fn get_dynamic_size(&self) -> usize {
        self.dynamic_section.entries.len() * std::mem::size_of::<DynamicEntry>()
    }

    /// Insert an external function symbol with GNU version info and PLT/GOT relocation entries.
    pub fn insert_external_function(&mut self, symbol_name:&str, config:&super::super::code_gen_config::CodeGenConfiguration) {
        let mapping = config.get_external_function_versions();
        if let Some((file, versions)) = mapping.find_version(symbol_name) {
            let strs = std::iter::once(file.clone()).chain(versions.clone()).collect::<Vec<_>>();
            let offsets = strs.iter().map(|s| (self.insert_string(s) as u32, vna_hash(s))).collect();            
            if let Some(version_index) = self.gnu_version_required_section.find_offset(&offsets) {
                self.gnu_version_section.add_entry(version_index); // add version entry for this symbol
            }
            else {
                self.gnu_version_required_section.add_new_version(offsets);
                let last_header = self.gnu_version_required_section.get_headers().last().unwrap();
                let version_index = last_header.get_max_version_index().unwrap();
                self.gnu_version_section.add_entry(version_index); // add version entry for this symbol
            }
        }
        else {
            panic!("External function '{}' version info not found", symbol_name);
            // self.gnu_version_section.add_entry(1); // all external symbols without version are set to 1
        }

        let offset = self.insert_string(symbol_name);
        let symbol = DynamicSymbolEntry::new_extern_global_function_entry(offset as u32);
        let symbol_index = self.dynamic_symbol_table.insert_symbol(symbol);

        self.plt_section.add_symbol();
        let r_offset = self.got_plt_section.add_symbol(self.get_plt_section().get_plt0_address());  
        self.plt_relocation_table.add_jump_slot_entry(r_offset, symbol_index as u32);
    }

    /// insert global pointer as symbol
    fn insert_global_poniter_symbol(&mut self, v:u64) {
        let symbol_name = "__global_pointer$";
        let offset = self.insert_string(symbol_name);
        let mut symbol = DynamicSymbolEntry::new_global_notype_entry(offset as u32, v);
        symbol.set_st_shndx(SHN_ABS.into()); // Absolute symbol
        self.dynamic_symbol_table.insert_symbol(symbol);
        self.gnu_version_section.add_entry(1);
    }

    /// insert text pointer as symbol
    fn insert_text_symbol(&mut self, v:u64) {
        let symbol_name = ".text";
        let offset = self.insert_string(symbol_name);
        let symbol = DynamicSymbolEntry::new_local_section_entry(offset as u32, v);
        self.dynamic_symbol_table.insert_symbol(symbol.clone());
        self.gnu_version_section.add_entry(0); // local symbol has version 0
    }

    /// insert a global symbol if not exists
    pub fn insert_global_function_symbol(&mut self, symbol_name:&str, v:u64, size:u64) {
        if self.dynamic_symbol_table.find_symbol_index_by_name(symbol_name, &self.dynamic_string_table.0).is_some() {
            return;
        }

        // get .text symbol and its section index
        let text_symbol = self.dynamic_symbol_table.get_symbol_entry_by_name(".text", &self.dynamic_string_table.0)
            .expect("Cannot find .text symbol when inserting global function symbol");
        let text_section_index = text_symbol.get_st_shndx();

        let offset = self.insert_string(symbol_name);
        let mut symbol = DynamicSymbolEntry::new_global_function_entry(offset as u32, v);
        symbol.set_symbol_size(size);
        symbol.set_st_shndx(text_section_index); // use .text section index
        self.dynamic_symbol_table.insert_symbol(symbol);
        self.gnu_version_section.add_entry(1);
    }

    /// insert all external functions from asm program
    pub fn insert_external_functions(&mut self, asm_prog:&AsmProgram, config:&mut CodeGenConfiguration) {
        let external_symbols = asm_prog.get_external_symbols();
        for symbol in external_symbols {
            self.insert_external_function(symbol.get_name(), config);
        }
    }

    /// update global function symbol value, if cannot find the symbol, will panic
    pub fn update_global_function_symbol(&mut self, symbol_name:&str, offset:u64, size:usize) {
        if let Some(symbol) = self.dynamic_symbol_table.find_symbol_by_name_mut(symbol_name, &self.dynamic_string_table.0) {
            symbol.set_value(offset);
            symbol.set_symbol_size(size as u64);
        }
        else {
            panic!("Cannot find global function symbol '{}' to update", symbol_name);
        }
    }

    /// insert all global functions from asm program, its value can be invalid and only serve as placeholder
    /// its value will be updated later
    pub fn add_global_functions(&mut self, asm_prog:&AsmProgram) {
        let global_functions = asm_prog.get_global_function_symbol_data();
        for (symbol_name, offset, size) in global_functions {
            self.insert_global_function_symbol(&symbol_name, offset as u64, size as u64);
        }
    }

    /// add string or get existing string offset, this function can also update the string table size
    pub fn insert_string(&mut self, s:&str) -> usize {
        let offset = self.dynamic_string_table.0.get_string_offset_or_add(s);
        self.update_string_table_size();
        offset
    }
    
    pub fn build_dynamic_section(&mut self, config : Option<&LinkerConfig>) {
        let is_lab = config.and_then(|x| Some(x.get_is_build_lib()));
        let mut dynamic_entries = Vec::new();
    
        match is_lab {
            Some(true) => {
                // Symbol table (.dynsym)
                dynamic_entries.push(DynamicEntry {
                    d_tag: DTTags::DT_SYMTAB,
                    d_val: self.dynamic_symbol_table.get_virtual_address(),
                });

                // String table (.dynstr)
                dynamic_entries.push(DynamicEntry {
                    d_tag: DTTags::DT_STRTAB,
                    d_val: self.dynamic_string_table.0.get_virtual_address(),
                });

                // Size of .dynstr
                dynamic_entries.push(DynamicEntry {
                    d_tag: DTTags::DT_STRSZ,
                    d_val: self.dynamic_string_table.0.get_size() as u64,
                });

                // GNU Hash table (.gnu.hash)
                dynamic_entries.push(DynamicEntry {
                    d_tag: DTTags::DT_GNU_HASH,
                    d_val: self.gnu_hash_section.get_virtual_address(),
                });

                // soname entry
                if let Some(linker_config) = config {
                    if linker_config.get_is_build_lib() {
                        if let Some(soname) = linker_config.get_soname() {
                            dynamic_entries.push(DynamicEntry {
                                d_tag: DTTags::DT_SONAME,
                                d_val: self.insert_string(soname) as u64,
                            });
                        }
                    }
                }

                // Terminator
                dynamic_entries.push(DynamicEntry {
                    d_tag: DTTags::DT_NULL,
                    d_val: 0,
                });
            }
            Some(false) | 
            None => {
                // Symbol table (.dynsym)
                dynamic_entries.push(DynamicEntry {
                    d_tag: DTTags::DT_SYMTAB,
                    d_val: self.dynamic_symbol_table.get_virtual_address(),
                });
            
                // String table (.dynstr)
                dynamic_entries.push(DynamicEntry {
                    d_tag: DTTags::DT_STRTAB,
                    d_val: self.dynamic_string_table.0.get_virtual_address(),
                });
            
                // Size of .dynstr
                dynamic_entries.push(DynamicEntry {
                    d_tag: DTTags::DT_STRSZ,
                    d_val: self.dynamic_string_table.0.get_size() as u64,
                });
            
                // PLT's GOT (.got.plt)
                dynamic_entries.push(DynamicEntry {
                    d_tag: DTTags::DT_PLTGOT,
                    d_val: self.got_plt_section.get_virtual_address(),
                });
            
                // Relocations for PLT (.rela.plt)
                dynamic_entries.push(DynamicEntry {
                    d_tag: DTTags::DT_JMPREL,
                    d_val: self.plt_relocation_table.get_virtual_address(),
                });
            
                // Size of .rela.plt
                dynamic_entries.push(DynamicEntry {
                    d_tag: DTTags::DT_PLTRELSZ,
                    d_val: self.plt_relocation_table.get_size() as u64,
                });
            
                // Type of PLT relocations (RELA)
                dynamic_entries.push(DynamicEntry {
                    d_tag: DTTags::DT_PLTREL,
                    d_val: DT_RELA,
                });

                // GNU Hash table (.gnu.hash)
                dynamic_entries.push(DynamicEntry {
                    d_tag: DTTags::DT_GNU_HASH,
                    d_val: self.gnu_hash_section.get_virtual_address(),
                });

                // Version table (.gnu.version)
                dynamic_entries.push(DynamicEntry {
                    d_tag: DTTags::DT_VERSYM,
                    d_val: self.gnu_version_section.get_virtual_address(),
                });

                // Version requirements (.gnu.version_r)
                dynamic_entries.push(DynamicEntry {
                    d_tag: DTTags::DT_VERNEED,
                    d_val: self.gnu_version_required_section.get_virtual_address(),
                });

                // version need entries size
                dynamic_entries.push(DynamicEntry {
                    d_tag: DTTags::DT_VERNEEDNUM,
                    d_val: self.gnu_version_required_section.get_headers().len() as u64,
                });

                // rela.dyn virtual address and size (if 0 means there is no such thing)
                // dynamic_entries.push(DynamicEntry { d_tag: DT_RELA, d_val: self.plt_relocation_table.get_virtual_address() });
                // dynamic_entries.push(DynamicEntry { d_tag: DT_RELASZ, d_val: self.plt_relocation_table.get_size() as u64 });
                // dynamic_entries.push(DynamicEntry { d_tag: DT_RELAENT, d_val: PLTRelocationEntry::default().get_size() as u64 });

                // soname entry
                if let Some(linker_config) = config {
                    if linker_config.get_is_build_lib() {
                        if let Some(soname) = linker_config.get_soname() {
                            dynamic_entries.push(DynamicEntry {
                                d_tag: DTTags::DT_SONAME,
                                d_val: self.insert_string(soname) as u64,
                            });
                        }
                    }
                }
            
                // Terminator
                dynamic_entries.push(DynamicEntry {
                    d_tag: DTTags::DT_NULL,
                    d_val: 0,
                });
            }
        }
    
        dynamic_entries.into_iter().for_each(|n| self.update_entry(n));
    }

    fn update_entry(&mut self, entry: DynamicEntry) {
        if let Some(index) = self.dynamic_section.entries.iter().position(|x| x.d_tag == entry.d_tag) {
            self.dynamic_section.entries[index] = entry;
        } else {
            self.dynamic_section.entries.push(entry);
        }
    }

    /// find symbol's plt virtual address from symbol name
    /// - Returns Some(virtual_address) if found, None otherwise
    pub fn find_symbol_plt_virtual_address(&self, symbol_name:&str) -> Option<u64> {
        let string_table = &self.get_dynamic_string_table();
        let symbol_table = self.get_dynamic_symbol_table();
        let rela_plt_table = self.get_plt_relocation_table();

        for (i, entry) in rela_plt_table.iter().enumerate() {
            let sym_index = entry.get_elf64_r_sym_index();
            if let Some(index) = symbol_table.find_symbol_index_by_name(symbol_name, string_table) {
                if sym_index == index as u32 {
                    let plt = self.get_plt_section();
                    let v = plt.get_virtual_address_of_entry(i);
                    if v.is_none() {
                        panic!("Failed to get virtual address of PLT entry for symbol {symbol_name}");
                    }

                    return v;
                }
            }
        }
        
        None
    }

    /// get the dynamic entry list as vector of bytes
    pub fn get_dyanmic_bytes(&self) -> Vec<u8> {
        let mut bytes = self.dynamic_section.alignment.get_padding_vec();
        for entry in &self.dynamic_section.entries {
            bytes.extend_from_slice(&entry.to_bytes());
        }
        bytes
    }

    /// generate the plt code and set to plt section's code variable
    pub fn generate_plt_code(&mut self) {
        let code = self.plt_section.generate_code();
        self.plt_section.set_code(code);
    }

    /// get text segment data, the plt section are packing to text segment
    pub fn to_text_segment_data(&self) -> CodeSection {
        self.validate();

        let code = self.get_plt_section().to_code_section();
        code.into()
    }

    /// set the virtual address of the text segment
    /// plt doesn't need to get virtual address, but other sections will need this virtual address
    /// so we provide this function to keep the interface consistent
    pub fn set_text_segment_address(&mut self, vadd:u64, linker_config:&LinkerConfig) {
        let va_offset = linker_config.get_virutual_address_start();
        self.plt_section.set_offset(vadd);
        self.plt_section.set_virtual_address(va_offset + self.plt_section.get_offset());
        self.got_plt_section.set_initial_resolver(self.plt_section.get_plt0_address());
        self.build_dynamic_section(Some(linker_config));
    }

    pub fn get_text_segment_size(&self) -> usize {
        self.plt_section.get_section_size()
    }

    /// get data segment data, the got section & dynamic entry list are packing to data segment
    pub fn to_data_segment_data(&self) -> DataSection {
        self.validate();

        let mut data = self.get_got_plt_section().to_data_section().into();
        data = data + self.to_data_section();  
        data
    }

    pub fn set_data_segment_address(&mut self, starting_vadd:u64, linker_config:&LinkerConfig) {
        let va_offset = linker_config.get_virutual_address_start();
        let mut vadd = starting_vadd;
        self.got_plt_section.set_offset(vadd);
        self.got_plt_section.set_virtual_address(self.got_plt_section.get_offset() + va_offset);

        //add one more unit size for .dynamic last entry
        vadd += (self.got_plt_section.get_section_size() + self.got_plt_section.get_got_unit_size()) as u64;

        self.set_offset(vadd);
        self.set_virtual_address(va_offset + self.get_offset());

        //append the last element which has .dynamic virtual address
        self.got_plt_section.append_last_element(self.get_virtual_address());

        //set plt session's got virtual address
        self.plt_section.set_got_plt_virtual_address(self.got_plt_section.get_virtual_address());

        self.build_dynamic_section(Some(linker_config));
    }

    pub fn get_data_segment_size(&self) -> usize {
        self.got_plt_section.get_section_size() + self.get_section_size()
    }

    /// get read only segment data, the dynamic section are packing to data section with special order
    /// order: .dynsym, .dynstr, .rela.plt
    /// See also: set_rosections_virtual_address 
    /// the [`set_rosections_virtual_address`] function
    pub fn to_readonly_segment_data(&self) -> DataSection {
        self.validate();

        let mut data = if let Some(interp) = self.get_interp() {
            assert!(self.get_dynamic_string_table().contains(&interp));
            DataSection::default()
        }
        else {
            DataSection::default()
        };

        data = data + self.dynamic_symbol_table.to_data_section();
        data = data + self.dynamic_string_table.0.to_data_section();
        data = data + self.plt_relocation_table.to_data_section();
        data = data + self.gnu_hash_section.to_data_section();
        data = data + self.gnu_version_section.to_data_section();
        data = data + self.gnu_version_required_section.to_data_section();
        data
    }

    /// set the virtual address and offset of the sections in read only segment
    /// the order must be keep the same as the [`to_readonly_segment_data`] function
    pub fn set_rosections_address(&mut self, starting_offset:u64, linker_config:&LinkerConfig) {
        let va_offset = linker_config.get_virutual_address_start();
        let mut vadd = starting_offset;
        self.dynamic_symbol_table.set_offset(vadd);
        self.dynamic_symbol_table.set_virtual_address(va_offset + self.dynamic_symbol_table.get_offset());

        vadd += self.dynamic_symbol_table.get_section_size() as u64;
        self.dynamic_string_table.0.set_offset(vadd);
        self.dynamic_string_table.0.set_virtual_address(va_offset + self.dynamic_string_table.0.get_offset());

        // update the interp offset along with the string table virtual address, 
        // the interp initial value is set as offset of the string inside the string table in default impl
        self.interp_offset += self.dynamic_string_table.0.get_offset();  

        vadd += self.dynamic_string_table.0.get_section_size() as u64;
        self.plt_relocation_table.set_offset(vadd);
        self.plt_relocation_table.set_virtual_address(va_offset + self.plt_relocation_table.get_offset());

        vadd += self.plt_relocation_table.get_section_size() as u64;
        self.gnu_hash_section.set_offset(vadd);
        self.gnu_hash_section.set_virtual_address(va_offset + self.gnu_hash_section.get_offset());

        vadd += self.gnu_hash_section.get_section_size() as u64;
        self.gnu_version_section.set_offset(vadd);
        self.gnu_version_section.set_virtual_address(va_offset + self.gnu_version_section.get_offset());

        vadd += self.gnu_version_section.get_section_size() as u64;
        self.gnu_version_required_section.set_offset(vadd);
        self.gnu_version_required_section.set_virtual_address(va_offset + self.gnu_version_required_section.get_offset());

        self.build_dynamic_section(Some(linker_config));
    }

    /// generate readonly segment tag for debug purpose
    pub fn generate_readonly_segment_tag(&self) -> String {
        let mut tag = String::new();
        if let Some(interp) = self.get_interp() {
            assert!(self.get_dynamic_string_table().contains(&interp));
            tag += &format!("interp @ 0x{:X}, ", self.get_interp_offset());
        }
        tag += &format!("{} @ 0x{:X}/{}, ", self.dynamic_symbol_table.get_section_name(), self.dynamic_symbol_table.get_virtual_address(), self.dynamic_symbol_table.get_section_size());
        tag += &format!("{} @ 0x{:X}/{}, ", self.dynamic_string_table.0.get_section_name(), self.dynamic_string_table.0.get_virtual_address(), self.dynamic_string_table.0.get_section_size());
        tag += &format!("{} @ 0x{:X}/{}, ", self.plt_relocation_table.get_section_name(), self.plt_relocation_table.get_virtual_address(), self.plt_relocation_table.get_section_size());
        tag += &format!("{} @ 0x{:X}/{}, ", self.gnu_hash_section.get_section_name(), self.gnu_hash_section.get_virtual_address(), self.gnu_hash_section.get_section_size());
        tag += &format!("{} @ 0x{:X}/{}, ", self.gnu_version_section.get_section_name(), self.gnu_version_section.get_virtual_address(), self.gnu_version_section.get_section_size());
        tag += &format!("{} @ 0x{:X}/{}, ", self.gnu_version_required_section.get_section_name(), self.gnu_version_required_section.get_virtual_address(), self.gnu_version_required_section.get_section_size());
        tag
    }

    /// get readonly segment data size
    pub fn get_readonly_segment_size(&self) -> usize {
        self.dynamic_symbol_table.get_section_size() +
        self.dynamic_string_table.0.get_section_size() +
        self.plt_relocation_table.get_section_size() +
        self.gnu_hash_section.get_section_size() +
        self.gnu_version_section.get_section_size() +
        self.gnu_version_required_section.get_section_size()
    }

    /// generate text segment tag for debug purpose
    pub fn generate_text_segment_tag(&self) -> String {
        format!("{} @ 0x{:X}/{}, {} entries", 
            self.plt_section.get_section_name(), 
            self.plt_section.get_virtual_address(), 
            self.plt_section.get_section_size(),
            self.plt_section.get_entry_number()
        )
    }

    /// generate data segment tag for debug purpose
    pub fn generate_data_segment_tag(&self) -> String {
        format!("{} @ 0x{:X}/{}, {} entries, {} @ 0x{:X}/{}, {} entries", 
            self.got_plt_section.get_section_name(), 
            self.got_plt_section.get_virtual_address(), 
            self.got_plt_section.get_section_size(),
            self.got_plt_section.get_entry_number(),
            self.get_section_name(),
            self.get_virtual_address(),
            self.get_section_size(),
            self.dynamic_section.entries.len()
        )
    }

    pub fn validate(&self) {
        let r = self.get_dynamic_symbol_table().get_entries().len() == self.gnu_version_section.get_entries().len();
        if !r {
            panic!("Dynamic symbol table and GNU version section entries count mismatch");
        }
    }

    pub fn get_virtual_address(&self) -> u64 {
        self.dynamic_section.virtual_address
    }

    pub fn set_virtual_address(&mut self, vadd:u64) {
        self.dynamic_section.virtual_address = vadd;
    }

    pub fn get_offset(&self) -> u64 {
        self.dynamic_section.offset
    }

    pub fn set_offset(&mut self, offset:u64) {
        let v = self.dynamic_section.alignment.calculate_padding_and_offset(offset);
        self.dynamic_section.offset = v;
    }

    pub fn get_alignment(&self) -> &Alignment {
        &self.dynamic_section.alignment
    }

    fn find_symbol_by_name_mut(&mut self, symbol_name:&str) -> Option<&mut DynamicSymbolEntry> {
        if let Some(index) = self.dynamic_symbol_table.find_symbol_index_by_name(symbol_name, &self.dynamic_string_table.0) {
            Some(&mut self.dynamic_symbol_table[index])
        } else {
            None
        }
    }

    fn update_text_symbol(&mut self, v:u64) {
        if let Some(symbol) = self.find_symbol_by_name_mut(".text") {
            symbol.set_value(v);
        }
    }

    fn update_global_poniter_symbol(&mut self, v:u64) {
        if let Some(symbol) = self.find_symbol_by_name_mut("__global_pointer$") {
            symbol.set_value(v + 0x800);
        }
    }

    /// setting symbol and other fields based on segment headers
    pub fn enrich_symbols(&mut self, segment_headers : &SegmentHeaderList) {
        let text_segment = & segment_headers[SectionType::Text];
        self.update_text_symbol(text_segment.get_file_offset());

        let data_segemnt = & segment_headers[SectionType::Data];
        self.update_global_poniter_symbol(data_segemnt.get_file_offset());

        let got_vadd = self.get_got_plt_section().get_virtual_address();
        self.plt_relocation_table.update_jump_slot_offset(got_vadd);
    }

    pub fn to_section_headers(&self) -> Elf64SectionHeaderList {
        let mut headers = Elf64SectionHeaderList::default();
        headers <<= &self.plt_section;
        headers <<= &self.got_plt_section;
        headers <<= &self.dynamic_symbol_table;
        headers <<= &self.dynamic_string_table;
        headers <<= &self.plt_relocation_table;
        headers <<= &self.gnu_hash_section;
        headers <<= &self.gnu_version_section;
        headers <<= &self.gnu_version_required_section;
        headers <<= self;
        headers
    }
}

impl Debug for ELFDynamicStructure {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = self.get_section_name();
        let alignment = self.get_alignment();
        let data_size = self.get_section_data_size();
        let memory_size = self.get_section_size();
        let hash_section = & self.gnu_hash_section;
        let gnu_version = & self.gnu_version_section;
        let gnu_version_r = & self.gnu_version_required_section;
        let offset = self.get_offset();

        write!(f, r###"{name} {{ 
                    virtual_address: 0x{:X}, offset = 0x{offset:X}, {alignment:?}, data size = 0x{data_size:X}/{data_size}, memory size = 0x{memory_size:X}/{memory_size},
                    {} entries: {:?}, 
                    {:?}, 
                    {:?}, 
                    {:?}, 
                    {:?}, 
                    {:?}, 
                    {hash_section:?},
                    {gnu_version:?},
                    {gnu_version_r:?},
                    interp_offset @ 0x{:X} }}"###, 
                    self.get_virtual_address(),
                    self.dynamic_section.entries.len(), self.dynamic_section.entries, 
            self.dynamic_string_table, 
            self.dynamic_symbol_table, 
            self.plt_relocation_table, 
            self.plt_section, 
            self.got_plt_section, 
            self.interp_offset
        )
    }
}

impl Default for ELFDynamicStructure {
    fn default() -> Self {
        let mut string_table = DynamicStringTable::default();
        let dynamic_symbol_table = ELFDynamicSymbolTable::default();
        let plt_relocation_table = ELFPLTRelocationTable::default();  //plt.rela section
        let plt_section = ELFPLTSection::default();
        let got_plt_section = GOTSection::default();
        let interp_offset = string_table.0.get_string_offset_or_add("/lib/ld-linux-riscv64-lp64d.so.1".as_ref()) as u64;   //set the default interpreter path wich is the offset inside the string table
        let gnu_hash_section = GnuHashSection::default();
        let gnu_version_section = GnuVersionSection::default();
        let gnu_version_required_section = GnuVersionRequiredSection::default();

        let mut r = Self {
            dynamic_section: DynamicSection::default(),
                dynamic_string_table: string_table,
                dynamic_symbol_table,
                plt_relocation_table,
                plt_section,
                got_plt_section,
                interp_offset,
                gnu_hash_section,
                gnu_version_section,
                gnu_version_required_section,
            };

        // insert text symbol which is local first
        r.insert_text_symbol(0);           //initial value is 0, will be updated later

        // insert global pointer symbol which is global next
        r.insert_global_poniter_symbol(0); //initial value is 0, will be updated later

        r.insert_required_lib("libc.so.6");
        r.insert_dynamic_entry_size(DynamicSymbolEntry::default().get_size() as u64);
        r.build_dynamic_section(None);
        r
    }
}

#[derive(Clone)]
pub struct DynamicEntry {
    d_tag: DTTags,
    d_val: u64,
}

impl DynamicEntry {
    pub fn new(d_tag:DTTags, d_val:u64) -> Self {
        Self { d_tag, d_val }
    }

    /// new dt_needed entry where d_value is the offset in symbol table
    pub fn new_dt_needed(d_val:u64) -> Self {
        Self::new(DTTags::DT_NEEDED, d_val)
    }

    pub fn new_strtab_size(d_val:u64) -> Self {
        Self::new(DTTags::DT_STRSZ, d_val)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let tag = self.d_tag as u64;
        bytes.extend_from_slice(&tag.to_le_bytes());
        bytes.extend_from_slice(&self.d_val.to_le_bytes());
        bytes
    }

    pub fn tag_to_string(&self) -> String {
        self.d_tag.item_to_string()
            .to_string()
    }
}

impl Debug for DynamicEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let tag = format!("{}/0x{:X}", self.tag_to_string(), self.d_tag as u64);
        let info = match self.d_tag {
            DTTags::DT_NEEDED => format!("{tag}: required library or symbol @ 0x{:X}", self.d_val),
            DTTags::DT_STRSZ => format!("{tag} dynamic string table {} bytes", self.d_val),
            DTTags::DT_SYMENT => format!("{tag} dynamic symbol table entry {} bytes", self.d_val),
            DTTags::DT_SYMTAB => format!("{tag} dynamic symbol table @ 0x{:X}", self.d_val),
            DTTags::DT_STRTAB => format!("{tag} dynamic string table @ 0x{:X}", self.d_val),
            DTTags::DT_PLTGOT => format!("{tag} GOT @ 0x{:X}", self.d_val),
            DTTags::DT_PLTRELSZ => format!("{tag} rela.plt size = {} bytes", self.d_val),
            DTTags::DT_JMPREL => format!("{tag} Rela.PLT @ 0x{:X}", self.d_val),
            DTTags::DT_PLTREL => format!("{tag} PLT relocation type is {}", if self.d_val == DT_RELA { "RELA" } else { "REL" }),
            DTTags::DT_GNU_HASH => format!("{tag} GNU hash table @ 0x{:X}", self.d_val),
            DTTags::DT_VERSYM => format!("{tag} Version symbol table @ 0x{:X}", self.d_val),
            DTTags::DT_VERNEED => format!("{tag} Version need table @ 0x{:X}", self.d_val),
            DTTags::DT_VERNEEDNUM => format!("{tag} Version need entries number = {}", self.d_val),
            DTTags::DT_NULL => "DT_NULL is the end".to_string(),
            _ => format!("{tag} with value 0x{:X}", self.d_val),
        };

        write!(f, "DynamicEntry({info})")
    }
}

impl Default for DynamicEntry {
    fn default() -> Self {
        Self { d_tag: DTTags::DT_NULL , d_val: 0 }
    }
}
