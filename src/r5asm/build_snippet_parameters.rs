use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Clone)]
pub struct BuildSnippetParameters {
    parameters: HashMap<String, String>,
}

impl BuildSnippetParameters {
    pub fn get(&self, key: &str) -> Option<&String> {
        self.parameters.get(key)
    }

    pub fn update(&mut self, key: String, value: String) {
        self.parameters.insert(key, value);
    }

    /// get current program counter (pc) if exists
    pub fn get_pc(&self) -> Option<u64> {
        self.get("pc").and_then(|v| v.parse::<u64>().ok())
    }

    /// get hash parameter list whose value can be converted to u64 and its name is not "pc"
    pub fn get_u64_parameters(&self) -> HashMap<String, u64> {
        self.parameters.iter()
            .filter_map(|(k, v)| {
                if k != "pc" {
                    v.parse::<u64>().ok().map(|num| (k.clone(), num))
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Debug for BuildSnippetParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // generate output for each key like (key, value/hex_value)
        write!(f, "BuildSnippetParameters {{ ")?;
        for (key, value) in &self.parameters {
            if let Ok(num) = value.parse::<u64>() {
                write!(f, "({key} : 0x{num:X}/{num}), ")?;
            } else {
                write!(f, "({}, {}), ", key, value)?;
            }
        }

        write!(f, " }}")
    }
}

impl Default for BuildSnippetParameters {
    fn default() -> Self {
        Self { parameters: HashMap::new() }
    }
}

impl From<Vec<(String, String)>> for BuildSnippetParameters {
    fn from(value: Vec<(String, String)>) -> Self {
        let parameters = value.into_iter().collect();
        Self { parameters }
    }
}

impl From<HashMap<String, String>> for BuildSnippetParameters {
    fn from(value: HashMap<String, String>) -> Self {
        Self { parameters: value }
    }
}

impl From<&HashMap<String, String>> for BuildSnippetParameters {
    fn from(value: &HashMap<String, String>) -> Self {
        value.clone().into()
    }
}