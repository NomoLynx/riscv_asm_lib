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
    } else if lower.starts_with("vs1r") || lower.starts_with("vse") {
        Some(VStoreKind::UnitStride)
    } else {
        None
    }
}

pub fn parse_vwidth(inc_name: &str) -> Option<VWidth> {
    let lower = inc_name.to_ascii_lowercase();
    if lower.starts_with("vs1r") || lower.starts_with("vl1r") || lower.contains("8") {
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

pub fn parse_vtypei(spec: &str) -> Option<u16> {
    let normalized = spec.to_ascii_lowercase().replace(' ', "");
    let parts = normalized
        .split(',')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();

    if parts.len() < 2 {
        return None;
    }

    let vsew = match parts[0] {
        "e8" => 0b000,
        "e16" => 0b001,
        "e32" => 0b010,
        "e64" => 0b011,
        "e128" => 0b100,
        "e256" => 0b101,
        "e512" => 0b110,
        "e1024" => 0b111,
        _ => return None,
    };

    let vlmul = match parts[1] {
        "m1" => 0b000,
        "m2" => 0b001,
        "m4" => 0b010,
        "m8" => 0b011,
        "mf8" => 0b101,
        "mf4" => 0b110,
        "mf2" => 0b111,
        _ => return None,
    };

    let mut vta = 0u16;
    let mut vma = 0u16;
    for part in parts.iter().skip(2) {
        match *part {
            "ta" => vta = 1,
            "tu" => vta = 0,
            "ma" => vma = 1,
            "mu" => vma = 0,
            _ => return None,
        }
    }

    Some((vma << 7) | (vta << 6) | (vsew << 3) | vlmul)
}

pub fn base_vector_mnemonic(inc_name: &str) -> &str {
    inc_name.split('.').next().unwrap_or(inc_name)
}
