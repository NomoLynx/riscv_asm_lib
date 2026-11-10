use rust_macro::Accessors;

use crate::r5asm::{code_option::{extension::{Ext, ExtSet, closure}, option_operation::OptionOperation}, r5asm_pest::Rule};

#[derive(Clone, Copy, Debug, Default, Accessors)]
pub struct CodeOption {
    explicit: ExtSet,
    effective: ExtSet,
    ever_enabled: ExtSet,   // for .riscv.attributes at the end
    relax: bool,
    pic: bool,
    operation: OptionOperation,
}

impl CodeOption {
    /// Apply one "+zba" / "-v" token. Does not recompute `effective`.
    fn apply_arch_ext(&mut self, tok: &str) {
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
}