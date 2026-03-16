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