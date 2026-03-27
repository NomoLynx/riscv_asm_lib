#[derive(Clone, PartialEq)]
pub (crate) enum BasicInstructionExtensions {
    BaseIntegerInstructions,
    RvcInstructions,
    Rv64128,
    RvzbbInstructions,
    RvzbsInstructions,
    RvzbaInstructions,
    RvPrivilegedInstructions,
    Rvm64128Instructions,
    Rvm32Instructions,
    Rv32aInstructions,
    Rv64a128aInstructions,
    RvfInstructions,
    Rvf64128Instructions,
    PseudoInstructions,
    CompactInstructions,
    Unknown,  //this only for internal use and should NOT used in asm parsing or asm code generation
}

impl BasicInstructionExtensions {
    pub (crate) fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "base_integer_instructions" => Some(Self::BaseIntegerInstructions),
            "rvc_instructions" => Some(Self::RvcInstructions),
            "rv_64_128" => Some(Self::Rv64128),
            "rv_zbb_instructions" => Some(Self::RvzbbInstructions),
            "rv_zbs_instructions" => Some(Self::RvzbsInstructions),
            "rv_zba_instructions" => Some(Self::RvzbaInstructions),
            "rv_privileged_instructions" => Some(Self::RvPrivilegedInstructions),
            "rvm64_128_instructions" => Some(Self::Rvm64128Instructions),
            "rvm32_instructions" => Some(Self::Rvm32Instructions),
            "rv32a_instructions" => Some(Self::Rv32aInstructions),
            "rv64a_128a_instructions" => Some(Self::Rv64a128aInstructions),
            "rvf_instructions" => Some(Self::RvfInstructions),
            "rvf64_128_instructions" => Some(Self::Rvf64128Instructions),
            "pseudoinstructions" => Some(Self::PseudoInstructions),
            "compactinstructions" => Some(Self::CompactInstructions),
            _ => None,
        }
    }
}

impl std::fmt::Debug for BasicInstructionExtensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::BaseIntegerInstructions => "BaseInteger",
            Self::PseudoInstructions => "Pseudo",
            Self::Rv32aInstructions => "RV32",
            Self::Rv64128 => "RV64_128",
            Self::RvzbbInstructions => "RVZbb",
            Self::RvzbsInstructions => "RVZbs",
            Self::RvzbaInstructions => "RVZba",
            Self::Rv64a128aInstructions => "RV64a_128a",
            Self::RvPrivilegedInstructions => "RVPriviledged",
            Self::RvcInstructions => "RVC",
            Self::Rvf64128Instructions => "RVF64_128",
            Self::RvfInstructions => "RVF",
            Self::Rvm32Instructions => "RVM32",
            Self::Rvm64128Instructions => "RVM64_128",
            Self::CompactInstructions => "Compact",
            Self::Unknown => "Unknown",
        };

        write!(f, "{}", s)
    }
}
