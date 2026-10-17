use rust_macro::{Accessors, GetMut};

use crate::r5asm::alignment::Alignment;
use crate::r5asm::dynamic_structure::dt_tags::DTTags;
use crate::r5asm::dynamic_structure::*;

/// elf dynamic structure representation, it has many structure related to dynamic linking
#[derive(Clone, GetMut, Accessors)]
pub struct DynamicSection {
    virtual_address: u64,           // virtual address of dynamic section
    offset: u64,
    alignment: Alignment,           // alignment for dynamic entries
    entries: Vec<DynamicEntry>,     // .dynamic section data
}

impl DynamicSection {
    /// insert a dynamic entry, if the entry is null entry, 
    /// it will only insert if the last entry is not null
    pub fn insert_entry(&mut self, entry: DynamicEntry) {
        if entry.is_null_entry() {
            if !self.is_last_entry_null() {
                self.entries.push(entry);
            }
        }
        else {
            if self.is_entry_empty() {
                self.entries.push(entry);
            }
            else {
                if self.is_last_entry_null() {
                    // insert before the last null entry
                    let last_index = self.entries.len() - 1;
                    self.entries.insert(last_index, entry);
                }
                else {
                    // no null entry at the end, just push to the end
                    self.entries.push(entry);
                }
            }
        }
        
    }

    /// check if the dynamic section has no entries
    pub fn is_entry_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// check if the last entry is null entry
    pub fn is_last_entry_null(&self) -> bool {
        self.entries.last()
            .map_or(false, |e| e.is_null_entry())
    }

    /// check if the dynamic section contains a specific entry
    pub fn contains_entry(&self, d_tag:DTTags) -> bool {
        self.entries.iter().any(|e| *e.get_d_tag() == d_tag)
    }
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