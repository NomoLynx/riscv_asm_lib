use super::*;
use super::super::section_type::*;

use std::{fmt::Debug, ops::Index};

#[derive(Clone)]
pub struct SegmentHeaderList {
    data : Vec<SegmentHeaderListItem>,
}

impl SegmentHeaderList {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn add(&mut self, item:SegmentHeaderListItem) {
        self.data.push(item);
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn insert(&mut self, index:usize, item:SegmentHeaderListItem) {
        self.data.insert(index, item);
    }

    pub fn get_segment_header(&self, section_type:&SectionType) -> Option<&ProgramHeader> {
        for item in &self.data {
            if &item.section_type == section_type {
                return Some(&item.segment);
            }
        }
        None
    }

    pub fn get_farest_offset(&self) -> u64 {
        let mut max_offset = 0;
        for item in &self.data {
            let offset = item.get_farest_offset();
            if offset > max_offset {
                max_offset = offset;
            }
        }
        max_offset
    }

    /// Get total size of all segments
    pub fn get_total_byte_size(&self) -> u64 {
        let r = self.len() * 56; // each segment header is 56 bytes
        r as u64
    }

    /// Update the file size of the phdr segment header and phdrsegment to include the new segment header
    pub fn update_phdr_segment_size(&mut self, new_segment_size: u64) {
        for item in &mut self.data {
            if item.section_type == SectionType::Phdr {
                let phdr_segment = item.get_segment_mut();
                phdr_segment.set_file_size(phdr_segment.get_file_size() + new_segment_size);
                phdr_segment.set_memory_size(phdr_segment.get_memory_size() + new_segment_size);
                break;
            }
        }

        for item in &mut self.data {
            if item.section_type == SectionType::Phdrsegment {
                let phdr_segment = item.get_segment_mut();
                phdr_segment.set_file_size(phdr_segment.get_file_size() + new_segment_size);
                phdr_segment.set_memory_size(phdr_segment.get_memory_size() + new_segment_size);
                break;
            }
        }
    }
}

impl Index<usize> for SegmentHeaderList {
    type Output = SegmentHeaderListItem;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl Index<SectionType> for SegmentHeaderList {
    type Output = ProgramHeader;

    fn index(&self, section_type: SectionType) -> &Self::Output {
        if let Some(n) = self.get_segment_header(&section_type) {
            return n;
        }
        else {
            panic!("SegmentHeaderList: no segment found for section type {:?}", section_type);
        }
    }
}

impl Debug for SegmentHeaderList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "SegmentHeaderList ({} items):", self.data.len())?;
        for (i, item) in self.data.iter().enumerate() {
            writeln!(f, "\t[{i}] {item:?}")?;
        }
        Ok(())
    }
}

impl Default for SegmentHeaderList {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct SegmentHeaderListItem {
    segment : ProgramHeader,
    section_type : SectionType,
}

impl SegmentHeaderListItem {
    pub fn new(segment:ProgramHeader, section_type:SectionType) -> Self {
        Self { segment, section_type }
    }

    pub fn get_segment(&self) -> &ProgramHeader {
        &self.segment
    }

    pub fn get_segment_mut(&mut self) -> &mut ProgramHeader {
        &mut self.segment
    }

    pub fn get_farest_offset(&self) -> u64 {
        self.get_segment().farest_offset()
    }
}

impl From<ProgramHeader> for SegmentHeaderListItem {
    fn from(segment: ProgramHeader) -> Self {
        Self::new(segment, SectionType::Text)
    }
}

impl From<(ProgramHeader, SectionType)> for SegmentHeaderListItem {
    fn from(value: (ProgramHeader, SectionType)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl Debug for SegmentHeaderListItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SegmentHeaderListItem {{ SectionType: {:?}, Segment: {:?} }}", self.section_type, self.segment)
    }
}