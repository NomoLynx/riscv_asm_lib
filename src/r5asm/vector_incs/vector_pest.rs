use super::vector_inc::{VLoadKind, VStoreKind, VWidth, ValueForm};

pub fn parse_value_form(inc_name: &str) -> Option<ValueForm> {
    let lower = inc_name.to_ascii_lowercase();
    if lower.ends_with(".vv") || lower.ends_with(".mm") || lower.ends_with(".vs") {
        Some(ValueForm::VV)
    } else if lower.ends_with(".vx") {
        Some(ValueForm::VX)
    } else if lower.ends_with(".vi") {
        Some(ValueForm::VI)
    } else {
        None
    }
}

pub fn parse_vm_from_option(option: Option<&str>) -> bool {
    !matches!(option.map(|x| x.to_ascii_lowercase()), Some(s) if s == "v0.t")
}

pub fn parse_vload_kind(inc_name: &str) -> Option<VLoadKind> {
    let lower = inc_name.to_ascii_lowercase();
    if lower.starts_with("vluxei") || lower.starts_with("vloxei") {
        Some(VLoadKind::Indexed)
    } else if lower.starts_with("vlse") {
        Some(VLoadKind::Strided)
    } else if lower.starts_with("vl") {
        Some(VLoadKind::UnitStride)
    } else {
        None
    }
}

pub fn parse_vstore_kind(inc_name: &str) -> Option<VStoreKind> {
    let lower = inc_name.to_ascii_lowercase();
    if lower.starts_with("vsuxei") || lower.starts_with("vsoxei") {
        Some(VStoreKind::Indexed)
    } else if lower.starts_with("vsse") {
        Some(VStoreKind::Strided)
    } else if lower.starts_with("vs") {
        Some(VStoreKind::UnitStride)
    } else {
        None
    }
}

pub fn parse_vwidth(inc_name: &str) -> Option<VWidth> {
    let lower = inc_name.to_ascii_lowercase();
    if lower.contains("8") {
        Some(VWidth::E8)
    } else if lower.contains("16") {
        Some(VWidth::E16)
    } else if lower.contains("32") {
        Some(VWidth::E32)
    } else if lower.contains("64") {
        Some(VWidth::E64)
    } else {
        None
    }
}

pub fn base_vector_mnemonic(inc_name: &str) -> &str {
    inc_name.split('.').next().unwrap_or(inc_name)
}
