use std::ffi::CString;

use super::super::{alignment::Alignment, machinecode::MachineCode};

use super::super::elf_file::{align_up, bytes_to_hex, hex_to_bytes};
use chrono::{Datelike, Local, Timelike};

#[derive(Debug, Clone)]
pub struct NoteSection {
    name: String,
    note_type: u32,
    descriptor: Vec<u8>,
    alignment : Alignment,
}

// Note type constants
pub const NT_TSPT: u32 = 0x14; // "0x14" indicates a my compiler note

impl NoteSection {
    /// Create a new note section
    pub fn new(name: String, note_type: u32, descriptor: Vec<u8>) -> Self {
        NoteSection {
            name,
            note_type,
            descriptor,
            alignment: Alignment::new(4).unwrap(),
        }
    }

    /// create a TSPT note section
    pub fn new_tspt() -> Self {
        let data = Self::datetime_to_vec();

        NoteSection::new("TT".to_string(), NT_TSPT, data)
    }

    /// Get size in bytes
    pub fn get_size_in_bytes(&self) -> usize {
        let binding = CString::new(self.name.clone()).unwrap();
        let name_bytes = binding.to_bytes_with_nul();
        let name_size = name_bytes.len();
        let desc_size = self.descriptor.len();
        
        12 + name_size + desc_size // 12 bytes for the header
    }

    /// get alignment
    pub fn get_alignment(&self) -> &Alignment {
        &self.alignment
    }

    /// Convert to ELF note format bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let namesz = self.name.len() as u32;
        let descsz = self.descriptor.len() as u32;

        let binding = CString::new(self.name.clone()).unwrap();
        let name_bytes = binding.to_bytes_with_nul();
        
        // Calculate aligned sizes (4-byte alignment)
        let name_size = align_up(name_bytes.len(), 4);
        let desc_size = align_up(self.descriptor.len(), 4);
        
        let mut bytes = Vec::new();
        
        // Write header
        bytes.extend_from_slice(&namesz.to_le_bytes());
        bytes.extend_from_slice(&descsz.to_le_bytes());
        bytes.extend_from_slice(&self.note_type.to_le_bytes());
        
        // Write name with padding
        bytes.extend_from_slice(name_bytes);
        bytes.resize(bytes.len() + name_size - name_bytes.len(), 0);
        
        // Write descriptor with padding
        bytes.extend_from_slice(&self.descriptor);
        bytes.resize(bytes.len() + desc_size - self.descriptor.len(), 0);
        
        bytes
    }

    /// Convert to space-separated hex string
    pub fn to_hex(&self) -> String {
        bytes_to_hex(&self.to_bytes())
    }

    /// Parse from hex string
    pub fn from_hex(hex: &str) -> Result<Self, &'static str> {
        let bytes = hex_to_bytes(hex)?;
        let mut cursor = 0;
        
        // Read header
        if bytes.len() < 12 {
            return Err("Invalid note section - too short");
        }
        
        let namesz = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let descsz = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let n_type = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        
        cursor += 12;
        
        // Read name
        if bytes.len() < cursor + namesz as usize {
            return Err("Invalid note name");
        }
        
        let name_bytes = &bytes[cursor..cursor + namesz as usize];
        let name = CString::new(name_bytes)
            .map_err(|_| "Invalid note name")?
            .into_string()
            .map_err(|_| "Invalid note name")?;
        
        cursor += align_up(namesz as usize, 4);
        
        // Read descriptor
        if bytes.len() < cursor + descsz as usize {
            return Err("Invalid note descriptor");
        }
        
        let desc = bytes[cursor..cursor + descsz as usize].to_vec();
        
        Ok(NoteSection {
            name,
            note_type: n_type,
            descriptor: desc,
            alignment: Alignment::new(4).unwrap(),
        })
    }

    /// Get the note name
    pub fn get_name(&self) -> &str {
        &self.name
    }

    fn to_bcd(value: u8) -> u8 {
        ((value / 10) << 4) | (value % 10)
    }

    fn datetime_to_vec() -> Vec<u8> {
        let now = Local::now();

        let year = now.year();       // full year, e.g., 2021
        let year_high = Self::to_bcd((year / 100) as u8); // 20
        let year_low = Self::to_bcd((year % 100) as u8);  // 21

        let month = Self::to_bcd(now.month() as u8);
        let day = Self::to_bcd(now.day() as u8);
        let hour = Self::to_bcd(now.hour() as u8);
        let minute = Self::to_bcd(now.minute() as u8);
        let second = Self::to_bcd(now.second() as u8);
        let millisecond = (now.timestamp_subsec_millis() & 0xFF) as u8;

        vec![
            year_high,
            year_low,
            month,
            day,
            hour,
            minute,
            second,
            millisecond,
        ]
    }
}

impl From<Vec<u8>> for NoteSection {
    fn from(bytes: Vec<u8>) -> Self {
        NoteSection::from_hex(&bytes_to_hex(&bytes)).unwrap()
    }
}

impl From<Vec<MachineCode>> for NoteSection {
    fn from(machine_codes: Vec<MachineCode>) -> Self {
        let bin = machine_codes.into_iter().flat_map(|x| { x.to_vec() }).collect::<Vec<_>>();
        bin.into()
    }
}

impl Default for NoteSection {
    fn default() -> Self {
        NoteSection {
            name: String::new(),
            note_type: 0,
            descriptor: vec![],
            alignment: Alignment::new(4).unwrap(),
        }
    }
}
