use super::imm_macro::ImmMacro;

#[derive(Clone, PartialEq, Eq)]
pub enum Imm {
    Value(String),
    ImmMacro(ImmMacro),
}

impl Imm {
    pub fn new() -> Self {
        Self::Value("0".to_string())
    }

    pub fn is_macro(&self) -> bool {
        matches!(self, Imm::ImmMacro(_))
    }

    pub fn is_value(&self) -> bool {
        matches!(self, Imm::Value(_))
    }

    /// get string option if imm is Self::Value
    pub fn to_string_option(&self) -> Option<String> {
        self.try_into().ok()
    }

    /// get value from Self::Value
    pub fn get_value(&self) -> Option<String> {
        match self {
            Imm::Value(s) => Some(s.clone()),
            Imm::ImmMacro(_) => None,
        }
    }

    /// get value from Self::ImmMacro
    pub fn get_macro_value(&self) -> Option<&ImmMacro> {
        match self {
            Imm::Value(_) => None,
            Imm::ImmMacro(macro_value) => Some(macro_value),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Imm::Value(s) => s.clone(),
            Imm::ImmMacro(macro_value) => macro_value.to_string(),
        }
    }
}

impl From<String> for Imm {
    fn from(value: String) -> Self {
        Self::Value(value)
    }
}

impl From<&String> for Imm {
    fn from(value: &String) -> Self {
        Self::Value(value.clone())
    }
}

impl From<ImmMacro> for Imm {
    fn from(macro_value: ImmMacro) -> Self {
        Self::ImmMacro(macro_value)
    }
}

impl From<&str> for Imm {
    fn from(value: &str) -> Self {
        Self::Value(value.to_string())
    }
}

impl From<u32> for Imm {
    fn from(value: u32) -> Self {
        Self::Value(value.to_string())
    }
}

impl From<u8> for Imm {
    fn from(value: u8) -> Self {
        Self::Value(value.to_string())
    }
}

impl TryFrom<&Imm> for String {
    type Error = &'static str;

    fn try_from(value: &Imm) -> Result<Self, Self::Error> {
        match value {
            Imm::Value(s) => Ok(s.clone()),
            Imm::ImmMacro(_macro_value) => Err("Cannot convert ImmMacro to String directly"),
        }
    }
}

impl std::fmt::Debug for Imm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Imm::Value(s) => write!(f, "{}", s),
            Imm::ImmMacro(macro_value) => write!(f, "Imm::ImmMacro({})", macro_value.to_string()),
        }
    }
}