pub use std::fmt::{self, Debug, Formatter};

use parser_lib::json::{Object, json_to_int_value};

#[derive(Clone)]
pub struct Alignment {
    value: u32,     // alignment value, must be power of 2
    padding: u32,   // padding bytes number needed to achieve the alignment
}

impl Alignment {
    pub fn new(value: u32) -> Result<Self, &'static str> {
        if !value.is_power_of_two() {
            return Err("Alignment value must be a power of 2");
        }
        Ok(Alignment { value, padding: 0 })
    }

    /// Calculate padding and return new offset after alignment
    pub fn calculate_padding_and_offset(&mut self, offset: u64) -> u64 {
        let align_mask = self.value as u64 - 1;
        if (offset & align_mask) == 0 {
            self.padding = 0; // already aligned
            offset
        } else {
            self.padding = (self.value as u64 - (offset & align_mask)) as u32;
            offset + self.padding as u64
        }
    }

    /// Get the alignment value
    pub fn get_value(&self) -> u32 {
        self.value
    }

    /// Get the calculated padding
    pub fn get_padding(&self) -> u32 {
        self.padding
    }

    /// get padding vec, its content is 0 with length of padding
    pub fn get_padding_vec(&self) -> Vec<u8> {
        vec![0; self.padding as usize]
    }

    pub fn from_json(obj:&Object) -> Self {
        Self { 
            value: json_to_int_value(&obj["value"]) as u32, 
            padding: json_to_int_value(&obj["padding"]) as u32,
        }
    }
}

impl Default for Alignment {
    fn default() -> Self {
        Alignment { value: 1, padding: 0 } // default alignment is 1 (no alignment)
    }
}

impl Debug for Alignment {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Alignment(0x{:X}, {} bytes padding)", self.value, self.padding)
    }
}

impl From<&Alignment> for u32 {
    fn from(align: &Alignment) -> Self {
        align.value
    }
}

impl From<&Alignment> for u64 {
    fn from(align: &Alignment) -> Self {
        align.value as u64
    }
}

// generate test cases for Alignment
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_alignment() {
        let mut align = Alignment::new(16).unwrap();
        let new_offset = align.calculate_padding_and_offset(18);
        assert_eq!(new_offset, 32);
        assert_eq!(align.get_padding(), 14);

        let mut align = Alignment::new(8).unwrap();
        let new_offset = align.calculate_padding_and_offset(24);
        assert_eq!(new_offset, 24);
        assert_eq!(align.get_padding(), 0);

        let mut align = Alignment::new(32).unwrap();
        let new_offset = align.calculate_padding_and_offset(45);
        assert_eq!(new_offset, 64);
        assert_eq!(align.get_padding(), 19);
    }
}