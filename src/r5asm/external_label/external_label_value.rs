use std::fmt::Debug;

#[derive(Clone)]
pub (crate) enum ExternalLabelValue {
    Address(u64),
    Offset(i64),
    Immediate(i64),
    Undefined,
}

impl ExternalLabelValue {
    pub fn is_defined(&self) -> bool {
        !matches!(self, ExternalLabelValue::Undefined)
    }

    pub fn get_address(&self) -> Option<u64> {
        if let ExternalLabelValue::Address(addr) = self {
            Some(*addr)
        } else {
            None
        }
    }

    pub fn get_offset(&self) -> Option<i64> {
        if let ExternalLabelValue::Offset(offset) = self {
            Some(*offset)
        } else {
            None
        }
    }

    pub fn get_immediate(&self) -> Option<i64> {
        if let ExternalLabelValue::Immediate(imm) = self {
            Some(*imm)
        } else {
            None
        }
    }
}

impl Default for ExternalLabelValue {
    fn default() -> Self {
        ExternalLabelValue::Undefined
    }
}

impl Debug for ExternalLabelValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExternalLabelValue::Address(addr) => write!(f, "Address(0x{:X})", addr),
            ExternalLabelValue::Offset(offset) => write!(f, "Offset({})", offset),
            ExternalLabelValue::Immediate(imm) => write!(f, "Immediate({})", imm),
            ExternalLabelValue::Undefined => write!(f, "Undefined"),
        }
    }
}