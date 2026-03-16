use rust_macro_internal::*;
use rust_macro::*;

use std::fmt::Debug;

#[packet_bit_vec("src/r5asm/r5inc.mermaid")]
#[derive(Clone, PartialEq, Eq, Accessors)]
pub struct R5Inc {
    
}

impl Debug for R5Inc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        for byte in self.get_data() {
            s.push_str(&format!("{:08b} ", byte));
        }
        write!(f, "R5Inc {{ data: {s} }}")
    }
}