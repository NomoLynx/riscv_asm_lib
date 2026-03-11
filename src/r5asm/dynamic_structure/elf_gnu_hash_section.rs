use super::super::{alignment::Alignment, traits::{section_name_trait::SectionNameTrait, section_size_trait::SectionSizeTrait}};
use super::{ELFDynamicSymbolTable, ELFStringTable};
use super::super::traits::*;
use std::fmt::Debug;

/// DT_GNU_HASH constant
pub const DT_GNU_HASH: u64 = 0x6ffffef5;

/// Representation of a GNU hash section + DT_GNU_HASH entry
#[derive(Clone)]
pub struct GnuHashSection {
    virtual_address: u64,
    offset : u64,
    
    nbuckets: u32,
    symoffset: u32,
    bloom_size: u32,
    bloom_shift: u32,
    bloom_filter: Vec<u64>,
    buckets: Vec<u32>,
    chains: Vec<u32>,

    alignment: Alignment,
}

impl GnuHashSection {
    /// Initialize from a list of symbol names
    pub fn new(symbols: &[&str], symoffset:u32) -> Self {
        let count = symbols.len() as u32;
        let nbuckets = count.max(1);
        let bloom_shift = 26;
        // power-of-two near count/8, at least 1
        let bloom_size = ((count.saturating_add(7)) / 8).max(1).next_power_of_two();

        let mut bloom_filter = vec![0u64; bloom_size as usize];
        let mut buckets = vec![0u32; nbuckets as usize];
        let mut chains = vec![0u32; count as usize];
        // Track last absolute dynsym index for each bucket to clear previous terminal bit
        let mut last_in_bucket: Vec<Option<u32>> = vec![None; nbuckets as usize];

        for (i, sym) in symbols.iter().enumerate() {
            let h = Self::gnu_hash(sym);
            // By default mark as terminal; if another symbol lands in same bucket,
            // we will clear the previous one's terminal bit.
            chains[i] = h | 1;

            // Bucket stores the first symbol index in this chain (absolute dynsym index)
            let b = (h % nbuckets) as usize;
            let abs_idx = symoffset + i as u32;
            if buckets[b] == 0 {
                buckets[b] = abs_idx;
            }
            if let Some(prev_abs) = last_in_bucket[b] {
                let prev_rel = (prev_abs - symoffset) as usize;
                chains[prev_rel] &= !1; // clear terminal bit of previous chain element
            }
            last_in_bucket[b] = Some(abs_idx);

            // Bloom filter for ELF64: set two bits
            let word_ix = ((h / 64) % bloom_size) as usize;
            let bit1 = (h % 64) as u32;
            let bit2 = ((h >> bloom_shift) % 64) as u32;
            bloom_filter[word_ix] |= (1u64 << bit1) | (1u64 << bit2);
        }

        GnuHashSection {
            virtual_address: 0,
            offset : 0,
            nbuckets,
            symoffset,
            bloom_size,
            bloom_shift,
            bloom_filter,
            buckets,
            chains,
            alignment: Alignment::new(8).unwrap(),
        }
    }

    /// Real GNU hash function (same as glibc/ld)
    pub fn gnu_hash(name: &str) -> u32 {
        let mut h: u32 = 5381;
        for b in name.bytes() {
            h = h.wrapping_mul(33).wrapping_add(b as u32);
        }
        h
    }

    pub fn get_virtual_address(&self) -> u64 {
        self.virtual_address
    }

    pub fn set_virtual_address(&mut self, vadd:u64) {
        self.virtual_address = vadd;
    }

    pub fn get_offset(&self) -> u64 {
        self.offset
    }

    pub fn set_offset(&mut self, offset:u64) {
        let v = self.alignment.calculate_padding_and_offset(offset);
        self.offset = v;
    }

    /// Serialize the `.gnu.hash` section into raw bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = self.alignment.get_padding_vec();

        // Write header fields
        buf.extend_from_slice(&self.nbuckets.to_le_bytes());
        buf.extend_from_slice(&self.symoffset.to_le_bytes());
        buf.extend_from_slice(&self.bloom_size.to_le_bytes());
        buf.extend_from_slice(&self.bloom_shift.to_le_bytes());

        // Write bloom filter
        for b in &self.bloom_filter {
            buf.extend_from_slice(&b.to_le_bytes());
        }

        // Write buckets
        for b in &self.buckets {
            buf.extend_from_slice(&b.to_le_bytes());
        }

        // Write chains
        for c in &self.chains {
            buf.extend_from_slice(&c.to_le_bytes());
        }

        buf
    }

    pub fn get_size(&self) -> usize {
        16 + (self.bloom_size as usize) * 8 + (self.nbuckets as usize) * 4 + (self.chains.len()) * 4
    }

    pub fn get_alignment(&self) -> &Alignment {
        &self.alignment
    }
}

impl Default for GnuHashSection {
    fn default() -> Self {
        GnuHashSection {
            virtual_address: 0,
            offset : 0,
            nbuckets: 0,
            symoffset: 0,
            bloom_size: 0,
            bloom_shift: 0,
            bloom_filter: Vec::new(),
            buckets: Vec::new(),
            chains: Vec::new(),
            alignment: Alignment::new(8).unwrap(),
        }
    }
}

impl Debug for GnuHashSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.get_section_name();
        let va = self.virtual_address;
        let data_size = self.get_section_data_size();
        let mem_size = self.get_section_size();
        let offset = self.get_offset();

        write!(f, "{name} @ 0x{va:X}/{va}, offset = 0x{offset:X} and data size = 0x{data_size:X}/{data_size}, memory size = 0x{mem_size:X}/{mem_size}, nbuckets={}, symoffset={}, bloom_size={}, bloom_shift={}, bloom_filter={:?}, buckets={:?}, chains={:?}, alignment={:?}",
            self.nbuckets,
            self.symoffset,
            self.bloom_size,
            self.bloom_shift,
            self.bloom_filter,
            self.buckets,
            self.chains,
            self.alignment,
        )
    }
}

impl IntoWith<&ELFStringTable, GnuHashSection> for &ELFDynamicSymbolTable  {
    fn into_with(self, string_table: &ELFStringTable) -> GnuHashSection {
        let mut strs = Vec::new();
        for symbol in self.get_entries() {
            if symbol.is_global() {
                let offset = symbol.get_name_offset() as u64;
                if let Some(name) = string_table.get_string_from_virtual_address(offset) {
                    strs.push(name);
                } else {
                    panic!("Failed to get symbol from offset 0x{offset:X}/{offset} from string table for global symbol");
                }
            }
        }

        let strs_ref: Vec<&str> = strs.iter().map(|s| s.as_str()).collect();

        if let Some(n) = self.get_global_symbol_index() {
            GnuHashSection::new(&strs_ref, n as u32)        
        } else {
            panic!("No global symbols found for GNU hash section");
        }
    }
}