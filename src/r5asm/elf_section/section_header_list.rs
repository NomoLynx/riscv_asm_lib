use super::super::{alignment::Alignment, dynamic_structure::{DYNAMIC_STRING_TABLE_NAME, GOT_SECTION_NAME}, traits::{SectionNameTrait, ToSectionHeaderTrait}};

use super::*;
use std::{fmt::{self, Debug, Formatter}, ops::{Index, IndexMut, ShlAssign, Deref}}; // added Deref

#[derive(Clone)]
pub struct Elf64SectionHeaderList {
    names: Vec<String>,
    headers: Vec<ElfSectionHeader>,
    
    offset : u64, // offset of the section headers in the final ELF file
    alignment : Alignment,
}

impl Elf64SectionHeaderList {
    pub fn get_headers(&self) -> Vec<&ElfSectionHeader> {
        self.headers.iter().collect()
    }

    pub fn set_headers(&mut self, headers: Vec<ElfSectionHeader>) {
        self.headers = headers;
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str, header: ElfSectionHeader) {
        self.names.push(name.to_string());
        self.headers.push(header);
    }

    pub fn add_header(&mut self, header: ElfSectionHeader) {
        self.headers.push(header);
    }

    pub fn get_offset(&self) -> u64 {
        self.offset
    }

    pub fn set_offset(&mut self, offset:u64) {
        let v = self.alignment.calculate_padding_and_offset(offset);
        self.offset = v;
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.alignment.get_padding_vec();
        for header in &self.headers {
            bytes.extend(&header.to_bytes());
        }
        bytes
    }

    pub fn len(&self) -> usize {
        self.headers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        let header_size = 64; // Each section header is 64 bytes in ELF64
        if data.len() % header_size != 0 {
            return None; // Data length must be a multiple of header size
        }

        let mut headers = Vec::new();
        for chunk in data.chunks(header_size) {
            let header = ElfSectionHeader::from_bytes(chunk)?;
            headers.push(header);
        }

        Some(Self { names:Vec::new(), headers, offset:0, alignment: Alignment::new(16).unwrap() })
    }

    pub fn clear(&mut self) {
        self.headers.clear();
    }

    /// Iterate (name, &header) pairs (same as &list iteration now).
    pub fn iter_named(&self) -> impl Iterator<Item = (&str, &ElfSectionHeader)> {
        self.names.iter().zip(self.headers.iter()).map(|(n,h)| (n.as_str(), h))
    }

    /// Consume self yielding (name, header) pairs (same as into_iter()).
    pub fn into_named(self) -> impl Iterator<Item = (String, ElfSectionHeader)> {
        self.names.into_iter().zip(self.headers.into_iter())
    }

    pub fn find_dynamic_symboles_section_index(&self) -> Option<usize> {
        self.headers.iter().position(|h| h.get_section_type() == 11) // SHT_DYNSYM
    }

    pub fn find_gnu_hash_section_index(&self) -> Option<usize> {
        self.headers.iter().position(|h| h.get_section_type() == 0x6ffffff6) // SHT_GNU_HASH
    }

    pub fn find_gnu_version_section_index(&self) -> Option<usize> {
        self.headers.iter().position(|h| h.get_section_type() == 0x6fffffff) // SHT_GNU_VERSYM
    }

    pub fn find_rela_plt_section_index(&self) -> Option<usize> {
        self.headers.iter().position(|h| h.get_section_type() == 4 ) // SHT_RELA 
    }

    pub fn find_got_section_index(&self) -> Option<usize> {
        let is = self.headers.iter().enumerate()
                    .filter(|(_i, h)| h.get_section_type() == 1) // PT_PROGBITS
                    .map(|(i, _)| i);

        for index in is {
            if let Some(name) = self.names.get(index) {
                if name == GOT_SECTION_NAME {
                    return Some(index);
                }
            }
        }

        None
    }

    pub fn find_dynamic_string_table_section_index(&self) -> Option<usize> {
        let is = self.headers.iter().enumerate()
                    .filter(|(_i, h)| h.get_section_type() == 3) // SHT_STRTAB
                    .map(|(i, _)| i);
        for index in is {
            if let Some(name) = self.names.get(index) {
                if name == DYNAMIC_STRING_TABLE_NAME {
                    return Some(index);
                }
            }
        }

        None
    }

    pub fn find_dynamic_section_index(&self) -> Option<usize> {
        self.headers.iter().position(|h| h.get_section_type() == 6) // SHT_DYNAMIC
    }

    pub fn find_gnu_version_required_section_index(&self) -> Option<usize> {
        self.headers.iter().position(|h| h.get_section_type() == 0x6ffffffe) // SHT_GNU_VERNEED
    }
}

impl Default for Elf64SectionHeaderList {
    fn default() -> Self {
        Self { 
            names : ["".to_string()].to_vec(), 
            headers: [ElfSectionHeader::default()].to_vec(),
            offset : 0,
            alignment : Alignment::new(16).unwrap(),
        }
    }
}

impl Debug for Elf64SectionHeaderList {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (i, header) in self.headers.iter().enumerate() {
            let name = if i < self.names.len() {
                &self.names[i]
            } else {
                "<unnamed>"
            };
            writeln!(f, "{}. Section Header '{name}': {:?}", i, header)?;
        }
        Ok(())
    }
}

impl Index<usize> for Elf64SectionHeaderList {
    type Output = ElfSectionHeader;

    fn index(&self, index: usize) -> &Self::Output {
        &self.headers[index]
    }
}

impl IndexMut<usize> for Elf64SectionHeaderList {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.headers[index]
    }
}

impl From<Vec<ElfSectionHeader>> for Elf64SectionHeaderList {
    fn from(headers: Vec<ElfSectionHeader>) -> Self {
        let r = Self { headers, names: Vec::new(), offset:0, alignment: Alignment::new(16).unwrap() };
        r
    }
}

impl<T> ShlAssign<&T> for Elf64SectionHeaderList
where
    T: ToSectionHeaderTrait + SectionNameTrait,
{
    fn shl_assign(&mut self, rhs: &T) {
        self.add(&rhs.get_section_name(), rhs.to_section_header());
    }
}

// Allow slice-like access (kept)
impl Deref for Elf64SectionHeaderList {
    type Target = [ElfSectionHeader];
    fn deref(&self) -> &Self::Target { &self.headers }
}

// ---------------------------------------------------------------------------
// Iterators producing name + header tuples
// ---------------------------------------------------------------------------

pub struct Elf64SectionHeaderListRefIter<'a> {
    index: usize,
    list: &'a Elf64SectionHeaderList,
}

impl<'a> Iterator for Elf64SectionHeaderListRefIter<'a> {
    type Item = (&'a str, &'a ElfSectionHeader);
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.list.headers.len() { return None; }
        let i = self.index;
        self.index += 1;
        let name = self.list.names.get(i).map(|s| s.as_str()).unwrap_or("<unnamed>");
        Some((name, &self.list.headers[i]))
    }
}

pub struct Elf64SectionHeaderListIntoIter {
    names: std::vec::IntoIter<String>,
    headers: std::vec::IntoIter<ElfSectionHeader>,
}

impl Iterator for Elf64SectionHeaderListIntoIter {
    type Item = (String, ElfSectionHeader);
    fn next(&mut self) -> Option<Self::Item> {
        let name = self.names.next()?;
        let header = self.headers.next()
            .expect("names and headers length mismatch in IntoIter");
        Some((name, header))
    }
}

// for (name, header) in list.into_iter()
impl IntoIterator for Elf64SectionHeaderList {
    type Item = (String, ElfSectionHeader);
    type IntoIter = Elf64SectionHeaderListIntoIter;
    fn into_iter(self) -> Self::IntoIter {
        Elf64SectionHeaderListIntoIter { names: self.names.into_iter(), headers: self.headers.into_iter() }
    }
}

// for (name, header) in &list
impl<'a> IntoIterator for &'a Elf64SectionHeaderList {
    type Item = (&'a str, &'a ElfSectionHeader);
    type IntoIter = Elf64SectionHeaderListRefIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        Elf64SectionHeaderListRefIter { index: 0, list: self }
    }
}

