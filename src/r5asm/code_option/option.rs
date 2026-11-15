use std::{fmt::Display, ops::Add};

use core_utils::debug::debug_string;
use rust_macro::Accessors;

use crate::r5asm::{code_option::{extension::{Ext, ExtSet, closure}, option_operation::OptionOperation}, r5asm_pest::Rule};

#[derive(Clone, Copy, Debug, Default, Accessors)]
pub struct CodeOption {
    explicit: ExtSet,
    effective: ExtSet,
    ever_enabled: ExtSet,   // for .riscv.attributes at the end
    relax: bool,
    pic: bool,
    operation: Option<OptionOperation>,
}

impl CodeOption {
    /// Apply one "+zba" / "-v" token. Does not recompute `effective`.
    pub fn apply_arch_ext(&mut self, tok: &str) {
        let add = tok.starts_with('+');
        let ext = Ext::from_name(&tok[1..]).expect("grammar guarantees a valid name");
        if add {
            self.explicit.insert(ext);
        } else {
            self.explicit.remove(ext);
        }
    }

    pub fn recompute(&mut self, xlen: u8) {
        self.effective = closure(self.explicit, xlen);
        self.ever_enabled.union(&self.effective);
    }

    /// Handle the whole `option_arch` pair: apply every token, then recompute once.
    pub fn apply_option_arch(&mut self, pair: &pest::iterators::Pair<Rule>, xlen: u8) {
        for p in pair.clone().into_inner() {
            if p.as_rule() == Rule::arch_ext {
                self.apply_arch_ext(p.as_str());
            }
        }
        
        self.recompute(xlen);
    }

    /// Add the compact extension to the explicit set.
    pub fn add_compact(&mut self) {
        self.explicit.insert(Ext::C);
    }

    /// Remove the compact extension from the explicit set.
    pub fn remove_compact(&mut self) {
        self.explicit.remove(Ext::C);
    }

    /// Check if the effective set contains the given extension.
    pub fn contains_extension(&self, ext: Ext) -> bool {
        self.effective.has(ext)
    }

    /// Check if the effective set is empty.
    pub fn is_effective_empty(&self) -> bool {
        self.effective.is_empty()
    }

    /// Create an empty `CodeOption` with default values.
    pub fn empty() -> Self {
        Self {
            explicit: ExtSet::default(),
            effective: ExtSet::default(),
            ever_enabled: ExtSet::default(),
            relax: false,
            pic: false,
            operation: None,
        }
    }
}

impl Display for CodeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let operation = match &self.operation {
            Some(op) => op.to_string(),
            None => "None".to_string(),
        };
        
        write!(f, "explicit: {}, effective: {}, ever_enabled: {}, relax: {}, pic: {}, operation: {operation}", 
            self.explicit, self.effective, self.ever_enabled, self.relax, self.pic)
    }
}

impl Add for CodeOption {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut result = self.clone();
        result.explicit.union(&other.explicit);
        result.effective.union(&other.effective);
        result.ever_enabled.union(&other.ever_enabled);
        result.relax |= other.relax;
        result.pic |= other.pic;
        result.operation = other.operation;
        result
    }
}