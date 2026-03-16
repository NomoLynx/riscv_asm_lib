/// define a label struct which is a string.
/// if string ends with _b, it is backward label
/// if string ends with _fw, it is forward label
/// otherwise it is a normal label, which is the label without any postfix
#[derive(Clone, Debug, Eq, Hash, Ord, PartialOrd)]
pub struct LabelIndicator {
    name: String, // The label name
}

impl LabelIndicator {
    const BACKWARD_POSTFIX : &str = "_b";
    const FORWARD_POSTFIX : &str = "_fw";
    const PREFIX : &str = "%";

    pub fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }

    pub fn is_backward(&self) -> bool {
        self.name.ends_with(Self::BACKWARD_POSTFIX)
    }

    pub fn is_forward(&self) -> bool {
        self.name.ends_with(Self::FORWARD_POSTFIX)
    }

    /// Check if the label is not backward or forward, meaning it is a normal label.
    pub fn is_normal(&self) -> bool {
        !self.is_backward() && !self.is_forward()
    }

    pub fn is_prefixed(&self) -> bool {
        self.name.starts_with(Self::PREFIX)
    }

    pub fn is_pcrel_lo(&self) -> bool {
        self.name.to_lowercase() == format!("{}pcrel_lo", Self::PREFIX)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get_real_name(&self) -> &str {
        if self.is_backward() {
            &self.name[..self.name.len() - Self::BACKWARD_POSTFIX.len()]
        } else if self.is_forward() {
            &self.name[..self.name.len() - Self::FORWARD_POSTFIX.len()]
        } else {
            &self.name
        }
    }
}

impl PartialEq for LabelIndicator {
    fn eq(&self, other: &Self) -> bool {
        self.get_real_name() == other.get_real_name()
    }
}

impl From<String> for LabelIndicator {
    fn from(name: String) -> Self {
        Self::new(&name)
    }
}

impl From<&String> for LabelIndicator {
    fn from(name: &String) -> Self {
        Self::new(name)
    }
}

impl From<&str> for LabelIndicator {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

impl From<&LabelIndicator> for String {
    fn from(label: &LabelIndicator) -> Self {
        label.name.clone()
    }
}

impl From<LabelIndicator> for String {
    fn from(label: LabelIndicator) -> Self {
        label.name
    }
}