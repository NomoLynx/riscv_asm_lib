use super::r5asm_pest::Rule;


#[derive(Debug, Clone)]
pub enum AsmError {
    GeneralError(AsmErrorSourceFileLocation, String),
    NoFound(AsmErrorSourceFileLocation, String),
    ConversionFailed(AsmErrorSourceFileLocation, String),
    MissingCase(AsmErrorSourceFileLocation, Rule),
    ConverstionError(AsmErrorSourceFileLocation, String),
    ParsingConversionError(AsmErrorSourceFileLocation, String),
    IncompatibleType(AsmErrorSourceFileLocation),
    NotSupportedOperation(AsmErrorSourceFileLocation, String),
    ParameterError(AsmErrorSourceFileLocation),
    WrongType(AsmErrorSourceFileLocation, String),
    CannotRetrieveValue(AsmErrorSourceFileLocation),
    IOError,
}

impl AsmError {
    pub fn get_error_location(&self) -> Option<&AsmErrorSourceFileLocation> {
        match self {
            AsmError::GeneralError(loc, _) |
            AsmError::NoFound(loc, _) |
            AsmError::ConversionFailed(loc, _) |
            AsmError::MissingCase(loc, _) |
            AsmError::ConverstionError(loc, _) |
            AsmError::ParsingConversionError(loc, _) |
            AsmError::IncompatibleType(loc) |
            AsmError::NotSupportedOperation(loc, _) |
            AsmError::ParameterError(loc) |
            AsmError::WrongType(loc, _) |
            AsmError::CannotRetrieveValue(loc) => Some(loc),
            AsmError::IOError => None
        }
    }

    pub fn get_error_message(&self) -> String {
        match self {
            AsmError::GeneralError(_, msg) |
            AsmError::NoFound(_, msg) |
            AsmError::ConversionFailed(_, msg) |
            AsmError::ConverstionError(_, msg) |
            AsmError::ParsingConversionError(_, msg) |
            AsmError::NotSupportedOperation(_, msg) |
            AsmError::WrongType(_, msg) => msg.clone(),
            AsmError::CannotRetrieveValue(_) => "Cannot retrieve value".to_string(),
            AsmError::ParameterError(_) => "Parameter error".to_string(),
            AsmError::IncompatibleType(_) => "Incompatible type".to_string(),
            AsmError::MissingCase(_, rule) => format!("Missing case for rule: {:?}", rule),
            AsmError::IOError => "IO Error".to_string()
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct AsmErrorSourceFileLocation(pub String, pub u32);

impl std::fmt::Display for AsmErrorSourceFileLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:0", self.0, self.1)
    }
}

impl From<(String, u32)> for AsmErrorSourceFileLocation {
    fn from(src: (String, u32)) -> Self {
        AsmErrorSourceFileLocation(src.0, src.1)
    }
}

impl From<(&str, u32)> for AsmErrorSourceFileLocation {
    fn from(src: (&str, u32)) -> Self {
        AsmErrorSourceFileLocation(src.0.to_string(), src.1)
    }
}

impl std::fmt::Debug for AsmErrorSourceFileLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:0", self.0, self.1)
    }
}