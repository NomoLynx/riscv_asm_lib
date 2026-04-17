use std::fmt::Debug;

use super::*;

/// label offset table is a table which contains label and it's offset
/// it is used to resolve labels in the code, so that we can get the offset of the label
/// it also contains the sequence number of the label, which is used to resolve forward and backward labels
/// forward labels are labels which are defined after the current instruction
/// backward labels are labels which are defined before the current instruction
/// normal labels are labels which are defined in the same section as the current instruction
#[derive(Clone)]
pub struct LabelOffsetTable {
    entries: Vec<LabelOffsetTableEntry>,
}

impl LabelOffsetTable {
    pub fn new() -> Self {
        LabelOffsetTable { entries: Vec::new() }
    }

    pub fn add_entry(&mut self, label: String, offset: usize, sequence_number: usize, related_label: &str) {
        self.entries.push(LabelOffsetTableEntry::new(label.into(), offset, sequence_number, related_label))
    }

    pub fn get_offset(&self, label: &str, offset : usize, related_label: Option<&String>) -> Option<usize> {
        let target_label = LabelIndicator::from(label);
        if target_label.is_pcrel_lo() { //it is %pcrel_lo 
            let (back, _) = self.get_back_forward_label_offset("%pcrel_hi", offset, related_label);
            back
        } 
        else if target_label.is_normal() {
            self.entries.iter().find(|entry| entry.label == target_label).map(|entry| entry.offset)
        }
        else {
            let (back, foward) = self.get_back_forward_label_offset(label, offset, related_label);
            let r = if target_label.is_backward() {
                back
            } else if target_label.is_forward() {
                foward
            } else {
                None // Should not happen, as we check for normal labels above
            };

            // if the label is not found, 
            // try to find the label with the same name
            if r.is_none() {
                self.entries().iter()
                    .find(|entry| entry.label == target_label)
                    .map(|entry| entry.offset)
            }
            else {
                r
            }
        }
    }

    pub fn entries(&self) -> &Vec<LabelOffsetTableEntry> {
        &self.entries
    }

    pub fn contains_key(&self, label: &str) -> bool {
        self.entries.iter().any(|entry| entry.label == label.into())
    }

    pub fn get_back_forward_label_offset(&self, label: &str, offset : usize, related_label: Option<&String>) -> (Option<usize>, Option<usize>) {
        let mut back_offset = None;
        let mut forward_offset = None;

        // get a list of entries with the label and sorted by sequence number
        let mut entries: Vec<&LabelOffsetTableEntry> = self.entries.iter()
            .filter(|entry| entry.label == label.into() && 
                                                    (related_label.is_none() || entry.label == related_label.unwrap().into()) )
            .collect();

        entries.sort_by_key(|entry| entry.sequence_number);

        // find the entry with it's sequence number just smaller than the given offset and the one with just larger than the given offset
        for entry in entries {
            if entry.offset < offset {
                back_offset = Some(entry.offset);
            } else if entry.offset > offset && forward_offset.is_none() {
                forward_offset = Some(entry.offset);
            }
        }

        (back_offset, forward_offset)
    }

    /// return the offset of a label, if the label is not found, return None
    /// the label must be the only label in the table, otherwise it will panic
    pub fn get_single_label_offset(&self, label: &str, offset_option : Option<usize>) -> Option<usize> {
        let target_label = LabelIndicator::from(label);
        if target_label.is_normal() {
            let found_items = self.entries.iter().filter(|entry| entry.label == target_label).collect::<Vec<_>>();
            if found_items.len() == 1 {
                Some(found_items[0].offset)
            } else if found_items.is_empty() {
                None
            } else {
                panic!("Multiple entries found for label '{}'", label);
            }
        } else {
            if let Some(offset) = offset_option {
                let (back, forward) = self.get_back_forward_label_offset(label, offset, None);
                if target_label.is_backward() {
                    back
                } else if target_label.is_forward() {
                    forward
                } else {
                    None // Should not happen, as we check for normal labels above
                }
            }
            else {
                // If no offset is provided, we cannot determine the label offset
                None
            }
        }
    }

    /// set a label to global
    pub fn set_label_global(&mut self, label: &str) {
        for entry in self.entries.iter_mut() {
            if entry.label == label.into() {
                entry.set_global();
            }
        }
    }

    /// set the size of a label (e.g. function size)
    pub fn set_label_size(&mut self, label: &str, size:usize) {
        for entry in self.entries.iter_mut() {
            if entry.label == label.into() {
                entry.meta_data.set_size(size.into());
            }
        }
    }

    /// get all global labels
    pub fn get_global_labels(&self) -> Vec<&LabelOffsetTableEntry> {
        self.entries.iter()
            .filter(|entry| entry.meta_data.is_global())
            .collect()
    }
}

impl FromIterator<(String, usize)> for LabelOffsetTable {
    fn from_iter<T: IntoIterator<Item = (String, usize)>>(iter: T) -> Self {
        let mut table = LabelOffsetTable::new();

        let mut seq = 0; // Default sequence number, can be modified if needed
        for (label, offset) in iter {
            table.add_entry(label, offset, seq, ""); // sequence_number is set to 0 by default
            seq += 1; // Increment sequence number for each entry
        }

        table
    }
}

impl Debug for LabelOffsetTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LabelOffsetTable with {} entries:", self.entries.len())?;
        for entry in &self.entries {
            write!(f, "\n  {:?}", entry)?;
        }
        write!(f, "\nEnd of LabelOffsetTable\n")?;

        Ok(())
    }
}

impl From<LabelTable<'_>> for LabelOffsetTable {
    fn from(label_table: LabelTable) -> Self {
        let mut table = LabelOffsetTable::new();
        for (entry, _) in label_table {
            table.add_entry(entry.label.into(), entry.offset, entry.sequence_number, entry.related_lable.name());
        }
        table
    }
}

impl Default for LabelOffsetTable {
    fn default() -> Self {
        LabelOffsetTable::new()
    }
}

pub type Label = LabelIndicator;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct LabelOffsetTableEntry { 
    label : Label,
    offset : usize,
    sequence_number : usize,
    related_lable: LabelIndicator,
    meta_data : SectionMetaData,
}

impl LabelOffsetTableEntry {
    pub fn new(label: Label, offset: usize, sequence_number: usize, related_label:&str) -> Self {
        Self { label, offset, sequence_number, related_lable: related_label.into(), meta_data: SectionMetaData::default() }
    }

    pub fn with_related_label(label: Label, offset: usize, sequence_number: usize, related_label: LabelIndicator) -> Self {
        Self { label, offset, sequence_number, related_lable: related_label, meta_data: SectionMetaData::default() }
    }

    pub fn get_label(&self) -> &Label {
        &self.label
    }

    pub fn get_offset(&self) -> usize {
        self.offset
    }

    pub fn get_symbol_size(&self) -> usize {
        self.meta_data.get_size().copied()
            .unwrap_or(0)
    }

    pub fn get_sequence_number(&self) -> usize {
        self.sequence_number
    }

    pub fn get_related_label(&self) -> &LabelIndicator {
        &self.related_lable
    }

    pub fn set_global(&mut self) {
        self.meta_data.set_global();
    }
}

impl Debug for LabelOffsetTableEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LabelOffsetTableEntry {{ label: {} @ 0x{offset:X} or {offset} , sequence: {seq} }}", 
            self.label.name(), offset=self.offset, seq=self.sequence_number)
    }
}

impl Ord for LabelOffsetTableEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.sequence_number.cmp(&other.sequence_number)
    }
}

impl PartialOrd for LabelOffsetTableEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}