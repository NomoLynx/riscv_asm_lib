use rust_macro::GenIsEnumVariant;
use rust_macro::Accessors;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Accessors)]
pub struct SectionMetaData {
    metadata_type : SectionMetaDataType,
    scope: SectionMetaDataScope,
    size : Option<usize>,
}

impl SectionMetaData {
    pub fn set_global(&mut self) {
        self.scope = SectionMetaDataScope::Global;
    }

    pub fn is_global(&self) -> bool {
        self.get_scope().is_global()
    }
}

impl Default for SectionMetaData {
    fn default() -> Self {
        SectionMetaData {
            metadata_type : SectionMetaDataType::Label,
            scope : SectionMetaDataScope::Local,
            size : None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SectionMetaDataType {
    Label,
    Function,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, GenIsEnumVariant)]
pub enum SectionMetaDataScope {
    Local,
    Global,
}