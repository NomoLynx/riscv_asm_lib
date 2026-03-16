use std::collections::BTreeMap;

use super::super::r5asm_pest::SectionItem2;
use super::LabelOffsetTableEntry;
use super::Label;

#[derive(Debug, Clone)]
pub struct LabelTable<'a> {
    entries: BTreeMap<LabelOffsetTableEntry, &'a SectionItem2>,
}

impl<'a> LabelTable<'a> {
    /// add item
    pub (crate) fn add(&mut self, entry: LabelOffsetTableEntry, item: &'a SectionItem2) {
        self.entries.insert(entry, item);
    }

    /// check if _start or main label exists, which is the entry point of the program
    pub (crate) fn get_entry_address(&self) -> Option<usize> {
        let start = "_start";
        if let Some(n) = self.get(start) {
            Some(n.get_offset())            
        }
        else {
            let main_tag = "main";
            if let Some(m) = self.get(main_tag) {
                Some(m.get_offset())
            }
            else {
                None
            }
        }
    }

    /// find the SectionItem2 by the label name, if there are multiple match, then panic
    pub (crate) fn get(&self, str_key:&str) -> Option<&'a SectionItem2> {
        let label = Label::from(str_key);
        let found_items = self.entries
            .iter()
            .filter(|(entry, _)| entry.get_label() == &label)
            .map(|(_, item)| *item)
            .collect::<Vec<_>>();

        if found_items.len() == 1 {
            Some(found_items[0])
        } else if found_items.is_empty() {
            None
        } else {
            panic!("Multiple entries found for label '{}'", str_key);
        }
    }

    /// get keys of the label table
    pub (crate) fn keys(&self) -> Vec<&Label> {
        self.entries.keys()
            .map(|entry| entry.get_label())
            .collect()
    }

    /// get the entries of the label table
    pub (crate) fn entries(&self) -> Vec<&LabelOffsetTableEntry> {
        self.entries.keys().collect()
    }
}

impl<'a> FromIterator<(&'a LabelOffsetTableEntry, &'a SectionItem2)> for LabelTable<'a> {
    fn from_iter<I: IntoIterator<Item = (&'a LabelOffsetTableEntry, &'a SectionItem2)>>(iter: I) -> Self {
        let mut entries = BTreeMap::new();
        for (key, value) in iter {
            entries.insert(key.clone(), value);
        }
        LabelTable { entries }
    }
}

impl<'a> IntoIterator for LabelTable<'a> {
    type Item = (LabelOffsetTableEntry, &'a SectionItem2);
    type IntoIter = std::collections::btree_map::IntoIter<LabelOffsetTableEntry, &'a SectionItem2>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl Default for LabelTable<'_> {
    fn default() -> Self {
        LabelTable {
            entries: BTreeMap::new(),
        }
    }
}