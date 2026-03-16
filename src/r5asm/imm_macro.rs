use pest::iterators::Pair;

use core_utils::debug::*;
use super::{asm_error::AsmError, r5asm_pest::Rule};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImmMacro {
    PtrSize
}

impl ImmMacro {
    pub fn new() -> Self {
        Self::PtrSize
    }

    pub fn to_string(&self) -> String {
        match self {
            ImmMacro::PtrSize => "PTRSIZE".to_string(),
        }
    }

    pub fn from_pair(pair:&Pair<Rule>) -> Result<Self, AsmError> {
        assert!(pair.as_rule() == Rule::imm_macro, "Expected Rule::imm_macro, found: {:?}", pair.as_rule());
        let inner = pair.to_owned().into_inner().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
        match inner.as_slice() {
            [(Rule::ptr_size, _p)] => {
                Ok(ImmMacro::PtrSize)
            }
            _ => {
                let err_str = format!("cannot find {:?} in imm_macro processing logic", inner);
                error_string(err_str.clone());
                return Err(AsmError::GeneralError((file!(), line!()).into(), err_str));
            }
        }
    }
}