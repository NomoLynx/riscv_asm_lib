use rust_macro::*;

use super::super::alignment::Alignment;
use std::fmt::Debug;
use super::super::traits::SectionNameTrait;

#[derive(Clone)]
pub struct GnuVersionRequiredHeader {
    version: u16,   // always 1
    cnt: u16,          // number of version entries
    file_offset: u32,  // offset to the file name string in the string table
    aux_offset: u32,    // offset to the first auxiliary entry, usually size of this header
    vn_next: u32,    // offset to the next version header, or 0 if last

    required_entries : Vec<GnuVersionRequiredAux>, // auxiliary entries, it contains the version entries for this dependency, the number of entries is determined by `cnt`
}

impl GnuVersionRequiredHeader {

    pub fn get_size_in_bytes(&self) -> usize {
        Self::get_header_size()    //header size
            + std::mem::size_of::<GnuVersionRequiredAux>() * self.required_entries.len()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(&self.version.to_le_bytes());
        bytes.extend(&self.cnt.to_le_bytes());
        bytes.extend(&self.file_offset.to_le_bytes());
        bytes.extend(&self.aux_offset.to_le_bytes());
        bytes.extend(&self.vn_next.to_le_bytes());

        for aux in &self.required_entries {
            bytes.extend(&aux.to_bytes());
        }

        bytes
    }

    pub fn get_max_version_index(&self) -> Option<u16> {
        self.required_entries.iter().map(|aux| aux.other).max()
    }

    pub fn get_header_size() -> usize {
        std::mem::size_of::<u16>() * 2 + std::mem::size_of::<u32>() * 3
    }

    pub fn set_next_offset(&mut self, offset: u32) {
        self.vn_next = offset;
    }

    /// Check if the section already contains the given file offset and hash (indicating the same dependency and version)
    pub fn contains_offset(&self, file_name_offset:u32, hash:u32) -> bool {
        self.file_offset == file_name_offset 
            && self.required_entries.iter().any(|aux| aux.get_hash() == hash)
    }
}

impl Default for GnuVersionRequiredHeader {
    fn default() -> Self {
        GnuVersionRequiredHeader {
            version: 1,
            cnt: 0,
            file_offset: 0,
            aux_offset: std::mem::size_of::<GnuVersionRequiredHeader>() as u32,
            vn_next: 0,
            required_entries: Vec::new(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Accessors)]
pub (self) struct GnuVersionRequiredAux {
    hash: u32,  // hash of the dependency name
    flags: u16, // usually 0
    other: u16,  // index to the version definition section, usually 1-based
    name_offset: u32,   // offset to the version name string in the string table
    next_offset: u32,   // offset to the next auxiliary entry, or 0 if last
}

impl GnuVersionRequiredAux {
    pub fn new(hash: u32, flags: u16, other: u16, name_offset: u32, next_offset: u32) -> Self {
        GnuVersionRequiredAux {
            hash,
            flags,
            other,
            name_offset,
            next_offset,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(&self.hash.to_le_bytes());
        bytes.extend(&self.flags.to_le_bytes());
        bytes.extend(&self.other.to_le_bytes());
        bytes.extend(&self.name_offset.to_le_bytes());
        bytes.extend(&self.next_offset.to_le_bytes());
        bytes
    }

    pub fn get_size_in_bytes() -> usize {
        let r = std::mem::size_of::<GnuVersionRequiredAux>();
        assert!(r == 16, "gnu version r aux size should be 16, but it is {r}");
        r
    }

    pub fn set_as_last(&mut self) {
        self.next_offset = 0;
    }
}


impl Default for GnuVersionRequiredAux {
    fn default() -> Self {
        GnuVersionRequiredAux {
            hash: 0,
            flags: 0,
            other: 0,
            name_offset: 0,
            next_offset: 0,
        }
    }
}

#[derive(Clone)]
pub struct GnuVersionRequiredSection {
    virtual_address : u64,
    offset : u64,
    alignment: Alignment,
    header: Vec<GnuVersionRequiredHeader>,
}

impl GnuVersionRequiredSection {
    pub fn get_offset(&self) -> u64 {
        self.offset
    }
    
    /// get virtual address
    pub fn get_virtual_address(&self) -> u64 {
        self.virtual_address
    }

    /// set virtual address
    pub fn set_virtual_address(&mut self, offset:u64) {
        self.virtual_address = offset;
    }

    pub fn set_offset(&mut self, offset:u64) {
        let v = self.alignment.calculate_padding_and_offset(offset);
        self.offset = v;
    }

    /// get alignment
    pub fn get_alignment(&self) -> &Alignment {
        &self.alignment
    }

    /// Add a new version
    /// offsets: Vec of (name_offset, hash_value), the first entry is the (file name offset, file name hash), 
    /// the rest are version names offsets and hashes
    pub fn add_new_version(&mut self, offsets : Vec<(u32, u32)>) {
        let mut aux_entries = Vec::new();
        let count = offsets.len() - 1; // excluding the file name entry which is the first one
        let version_index_starting = self.header.iter()
                                            .map(|x| x.get_max_version_index().unwrap_or(1))
                                            .max()
                                            .unwrap_or(1);

        for (i, (name_offset, hash_value)) in offsets.iter().enumerate() {
            if i==0 {
                continue; // skip the first entry, which is the file name
            }

            // create auxiliary entry
            let next_offset = if i == count - 1 { 0 } else { GnuVersionRequiredAux::get_size_in_bytes() as u32 };
            let aux = GnuVersionRequiredAux::new(*hash_value, 0, version_index_starting + i as u16, *name_offset, next_offset);
            aux_entries.push(aux);
        }

        if let Some(last) = aux_entries.last_mut() {
            last.set_as_last();
        }

        let header = GnuVersionRequiredHeader {
            version: 1,
            cnt: count as u16,
            file_offset: offsets[0].0, // assuming the first offset is the file name
            aux_offset: GnuVersionRequiredHeader::get_header_size() as u32,
            vn_next: 0,
            required_entries: aux_entries,
        };

        // compute vn_next of the previous header if exists
        let current_size = self.get_size_in_bytes();
        if let Some(prev) = self.header.last_mut() {
            prev.set_next_offset(current_size as u32);
        }

        self.header.push(header);
    }

    pub fn get_headers(&self) -> &Vec<GnuVersionRequiredHeader> {
        &self.header
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        self.validate();

        let mut bytes = self.alignment.get_padding_vec();
        for header in &self.header {
            bytes.extend(header.to_bytes());
        }
        bytes
    }

    /// Get size in bytes
    pub fn get_size_in_bytes(&self) -> usize {
        let header_size: usize = self.header.iter().map(|h| h.get_size_in_bytes()).sum();
        header_size
    }

    /// Find the version index for the given file offset and hash, return None if not found
    /// offset vec's first element is (file name offset in string table, file name hash value)
    /// offset vec's rest elements are (version name offset in string table, version name hash value)
    pub fn find_offset(&self, offsets:&Vec<(u32, u32)>) -> Option<u16> {
        let file_offset = offsets[0].0;
        if let Some(file_entry) = self.header.iter().find(|h| h.file_offset == file_offset) {
            for offset in offsets.iter().skip(1) {
                let hash = offset.1;
                if let Some(aux) = file_entry.required_entries.iter().find(|aux| aux.get_hash() == hash) {
                    return Some(aux.get_other());
                }
            }
        }

        None // not found
    }

    fn validate(&self) {
        for header in self.get_headers() {
            assert!(header.cnt as usize == header.required_entries.len())
        }
    }
}

impl Default for GnuVersionRequiredSection {
    fn default() -> Self {
        GnuVersionRequiredSection {
            virtual_address: 0,
            offset: 0,
            alignment: Alignment::new(8).unwrap(),
            header: Vec::new(),
        }
    }
}

impl Debug for GnuVersionRequiredAux {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = "required_aux";
        let name_offset = self.name_offset;
        let next_offset = self.next_offset;
        let next_offset_tag = if next_offset == 0 { "LAST".to_string() } 
                    else { format!("0x{next_offset:X}/{next_offset}") };
        let hash = self.hash;
        write!(f, "{name} {{ hash: 0x{hash:08X}/{hash}, flags: {}, other: {}, name_offset: 0x{name_offset:X}/{name_offset}, next_offset: {next_offset_tag} }}",
            self.flags, self.other)
    }
}

impl Debug for GnuVersionRequiredHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.get_section_name();
        let file_offset = self.file_offset;
        let aux_offset = self.aux_offset;
        let next = self.vn_next;
        let next_tag = if next == 0 { "LAST".to_string() } 
                    else { format!("0x{next:X}/{next}") };
        write!(f, "{name} {{ ver: {}, cnt: {}, file_offset: 0x{file_offset:X}/{file_offset}, aux_offset: 0x{aux_offset:X}/{aux_offset}, vn_next : {next_tag}, {} required_entries: {:?} }}",
            self.version, self.cnt, self.required_entries.len(), self.required_entries)
    }
}

impl Debug for GnuVersionRequiredSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.get_section_name();
        let offset = self.get_virtual_address();
        write!(f, "{name} @ 0x{offset:X}/{offset} {{ alignment: {:?}, headers: {:?} }}",
            self.alignment, self.header)
    }
}