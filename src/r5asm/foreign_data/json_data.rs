use parser_lib::csv::InferredType;
use parser_lib::json::{JsonParseError, Object, Value, infer_scalar_from_str};
use parser_lib::json::parse_json_from_file;
use core_utils::number::*;

pub type JsonObject = Object;

pub fn json_file_to_asm_data_code(file_path:&str) -> Result<String, JsonParseError> {
    // Placeholder implementation, replace with actual JSON parsing and ASM code generation
    let obj = parse_json_from_file(file_path)?;
    
    let mut out = Vec::new();
    parse_json_data(obj, "", &mut out)?;

    let mut output_string = format!(".data\r\n");
    for n in out {
        output_string.push_str(&n.to_asm_code());
        output_string.push('\n');
    }

    Ok(output_string)
}

fn parse_json_data(obj: JsonObject, prefix: &str, out : &mut Vec<JsonData>) -> Result<(), JsonParseError> {
    for struct_obj in obj.get_pairs() {
        let (key, value) = struct_obj.into();
        let full_key = if prefix.is_empty() { key.to_string() } 
                                else { format!("{prefix}_{key}") };
        value_to_json_value(value, &full_key, out)?;
    }

    Ok(())
}

fn value_to_json_value(value: &Value, prefix: &str, out : &mut Vec<JsonData>) -> Result<(), JsonParseError> {
    match value {
        Value::String(s) => out.push(JsonData {
            name: prefix.to_string(),
            value: JsonValue::String(s.to_string()),
        }),
        Value::Null => out.push(JsonData {
            name: prefix.to_string(),
            value: JsonValue::Null,
        }),
        Value::Number(n) => { 
            let ty = infer_scalar_from_str(n);
            match ty {
                InferredType::Int => { 
                    let v = get_i64_from_str(n).unwrap();
                    out.push(JsonData {
                        name: prefix.to_string(),
                        value: JsonValue::Int(v),
                    })
                }
                InferredType::UInt => { 
                    let v = get_u64_from_str(n).unwrap();
                    out.push(JsonData {
                        name: prefix.to_string(),
                        value: JsonValue::UInt(v),
                    })
                }
                InferredType::Float => {
                    let v = get_f32_from_str(n).unwrap();
                    out.push(JsonData {
                        name: prefix.to_string(),
                        value: JsonValue::Float(v as i32),
                    })
                }
                _ => return Err(JsonParseError::GeneralError(format!("{ty:?} cannot be converted to Number"))),
            }
        }
        Value::Boolean(b) => out.push(JsonData {
            name: prefix.to_string(),
            value: JsonValue::Int(if *b { 1 } else { 0 }),
        }),
        Value::Array(arr) => {
            let full_key = format!("Array_{prefix}");
            for n in arr {
                value_to_json_value(n, &full_key, out)?;
            }
        }
        Value::Object(n) => {
            for json_struct in n.get_pairs() {
                let (key, value) = json_struct.into();
                let full_key = format!("{prefix}_{key}");
                value_to_json_value(value, &full_key, out)?;
            }
        }
    };

    Ok(())
}

enum JsonValue {
    String(String),
    Int(i64),
    UInt(u64),
    Float(i32),
    Null,
}

struct JsonData {
    name : String,
    value : JsonValue,
}

impl JsonData {
    pub fn to_asm_code(&self) -> String {
        match &self.value {
            JsonValue::String(s) => format!("{}: .string \"{}\"", self.name, s),
            JsonValue::Int(i) => format!("{}: .dword {}", self.name, i),
            JsonValue::UInt(u) => format!("{}: .dword {}", self.name, u),
            JsonValue::Float(f) => format!("{}: .double {}", self.name, f),
            JsonValue::Null => format!("{}: .dword 0", self.name),
        }
    }
}