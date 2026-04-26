use std::collections::HashMap;

use core_utils::debug::*;
use super::asm_error::AsmError;

#[derive(Debug, Clone)]
pub struct Register {
    from_abi: HashMap<&'static str, i32>,
}

impl Register {
    pub fn new() -> Self {
        let mut from_abi = HashMap::new();
        from_abi.insert("ZERO", 0);
        from_abi.insert("RA", 1);
        from_abi.insert("SP", 2);
        from_abi.insert("GP", 3);
        from_abi.insert("TP", 4);
        from_abi.insert("T0", 5);
        from_abi.insert("T1", 6);
        from_abi.insert("T2", 7);
        from_abi.insert("S0", 8);
        from_abi.insert("FP", 8);
        from_abi.insert("S1", 9);
        from_abi.insert("A0", 10);
        from_abi.insert("A1", 11);
        from_abi.insert("A2", 12);
        from_abi.insert("A3", 13);
        from_abi.insert("A4", 14);
        from_abi.insert("A5", 15);
        from_abi.insert("A6", 16);
        from_abi.insert("A7", 17);
        from_abi.insert("S2", 18);
        from_abi.insert("S3", 19);
        from_abi.insert("S4", 20);
        from_abi.insert("S5", 21);
        from_abi.insert("S6", 22);
        from_abi.insert("S7", 23);
        from_abi.insert("S8", 24);
        from_abi.insert("S9", 25);
        from_abi.insert("S10", 26);
        from_abi.insert("S11", 27);
        from_abi.insert("T3", 28);
        from_abi.insert("T4", 29);
        from_abi.insert("T5", 30);
        from_abi.insert("T6", 31);
        from_abi.insert("X0", 0);
        from_abi.insert("X1", 1);
        from_abi.insert("X2", 2);
        from_abi.insert("X3", 3);
        from_abi.insert("X4", 4);
        from_abi.insert("X5", 5);
        from_abi.insert("X6", 6);
        from_abi.insert("X7", 7);
        from_abi.insert("X8", 8);
        from_abi.insert("X9", 9);
        from_abi.insert("X10", 10);
        from_abi.insert("X11", 11);
        from_abi.insert("X12", 12);
        from_abi.insert("X13", 13);
        from_abi.insert("X14", 14);
        from_abi.insert("X15", 15);
        from_abi.insert("X16", 16);
        from_abi.insert("X17", 17);
        from_abi.insert("X18", 18);
        from_abi.insert("X19", 19);
        from_abi.insert("X20", 20);
        from_abi.insert("X21", 21);
        from_abi.insert("X22", 22);
        from_abi.insert("X23", 23);
        from_abi.insert("X24", 24);
        from_abi.insert("X25", 25);
        from_abi.insert("X26", 26);
        from_abi.insert("X27", 27);
        from_abi.insert("X28", 28);
        from_abi.insert("X29", 29);
        from_abi.insert("X30", 30);
        from_abi.insert("X31", 31);

        from_abi.insert("V0", 0);
        from_abi.insert("V1", 1);
        from_abi.insert("V2", 2);
        from_abi.insert("V3", 3);
        from_abi.insert("V4", 4);
        from_abi.insert("V5", 5);
        from_abi.insert("V6", 6);
        from_abi.insert("V7", 7);
        from_abi.insert("V8", 8);
        from_abi.insert("V9", 9);
        from_abi.insert("V10", 10);
        from_abi.insert("V11", 11);
        from_abi.insert("V12", 12);
        from_abi.insert("V13", 13);
        from_abi.insert("V14", 14);
        from_abi.insert("V15", 15);
        from_abi.insert("V16", 16);
        from_abi.insert("V17", 17);
        from_abi.insert("V18", 18);
        from_abi.insert("V19", 19);
        from_abi.insert("V20", 20);
        from_abi.insert("V21", 21);
        from_abi.insert("V22", 22);
        from_abi.insert("V23", 23);
        from_abi.insert("V24", 24);
        from_abi.insert("V25", 25);
        from_abi.insert("V26", 26);
        from_abi.insert("V27", 27);
        from_abi.insert("V28", 28);
        from_abi.insert("V29", 29);
        from_abi.insert("V30", 30);
        from_abi.insert("V31", 31);
        from_abi.insert("", 0);

        //float reg
        from_abi.insert("F0", 0);
        from_abi.insert("F1", 1);
        from_abi.insert("F2", 2);
        from_abi.insert("F3", 3);
        from_abi.insert("F4", 4);
        from_abi.insert("F5", 5);
        from_abi.insert("F6", 6);
        from_abi.insert("F7", 7);
        from_abi.insert("F8", 8);
        from_abi.insert("F9", 9);
        from_abi.insert("F10", 10);
        from_abi.insert("F11", 11);
        from_abi.insert("F12", 12);
        from_abi.insert("F13", 13);
        from_abi.insert("F14", 14);
        from_abi.insert("F15", 15);
        from_abi.insert("F16", 16);
        from_abi.insert("F17", 17);
        from_abi.insert("F18", 18);
        from_abi.insert("F19", 19);
        from_abi.insert("F20", 20);
        from_abi.insert("F21", 21);
        from_abi.insert("F22", 22);
        from_abi.insert("F23", 23);
        from_abi.insert("F24", 24);
        from_abi.insert("F25", 25);
        from_abi.insert("F26", 26);
        from_abi.insert("F27", 27);
        from_abi.insert("F28", 28);
        from_abi.insert("F29", 29);
        from_abi.insert("F30", 30);
        from_abi.insert("F31", 31);

        from_abi.insert("FT0", 0);
        from_abi.insert("FT1", 1);
        from_abi.insert("FT2", 2);
        from_abi.insert("FT3", 3);
        from_abi.insert("FT4", 4);
        from_abi.insert("FT5", 5);
        from_abi.insert("FT6", 6);
        from_abi.insert("FT7", 7);
        from_abi.insert("FS0", 8);
        from_abi.insert("FS1", 9);
        from_abi.insert("FA0", 10);
        from_abi.insert("FA1", 11);
        from_abi.insert("FA2", 12);
        from_abi.insert("FA3", 13);
        from_abi.insert("FA4", 14);
        from_abi.insert("FA5", 15);
        from_abi.insert("FA6", 16);
        from_abi.insert("FA7", 17);
        from_abi.insert("FS2", 18);
        from_abi.insert("FS3", 19);
        from_abi.insert("FS4", 20);
        from_abi.insert("FS5", 21);
        from_abi.insert("FS6", 22);
        from_abi.insert("FS7", 23);
        from_abi.insert("FS8", 24);
        from_abi.insert("FS9", 25);
        from_abi.insert("FS10", 26);
        from_abi.insert("FS11", 27);
        from_abi.insert("FT8", 28);
        from_abi.insert("FT9", 29);
        from_abi.insert("FT10", 30);
        from_abi.insert("FT11", 31);

        from_abi.insert("FFLAGS", 0x001);
        from_abi.insert("FRM", 0x002);
        from_abi.insert("FCSR", 0x003);

        from_abi.insert("SSTATUS", 0x100);
        from_abi.insert("SEDELEG", 0x102);
        from_abi.insert("SIDELEG", 0x103);
        from_abi.insert("SIE", 0x104);
        from_abi.insert("STVEC", 0x105);
        from_abi.insert("SCOUNTEREN", 0x106);

        from_abi.insert("SSCRATCH", 0x140);
        from_abi.insert("SEPC", 0x141);
        from_abi.insert("SCAUSE", 0x142);
        from_abi.insert("STVAL", 0x143);
        from_abi.insert("SIP", 0x144);

        from_abi.insert("SATP", 0x180);

        from_abi.insert("MSSTATUS", 0x300);
        from_abi.insert("MISA", 0x301);
        from_abi.insert("MEDELEG", 0x302);
        from_abi.insert("MIDELEG", 0x303);
        from_abi.insert("MIE", 0x304);
        from_abi.insert("MTVEC", 0x305);
        from_abi.insert("MCOUNTEREN", 0x306);
        from_abi.insert("MSSTATUSH", 0x310);

        from_abi.insert("MCOUNTINHIBIT", 0x320);

        from_abi.insert("mhpmevent3", 0x323);
        from_abi.insert("mhpmevent4", 0x324);
        from_abi.insert("mhpmevent5", 0x325);
        from_abi.insert("mhpmevent6", 0x326);
        from_abi.insert("mhpmevent7", 0x327);
        from_abi.insert("mhpmevent8", 0x328);
        from_abi.insert("mhpmevent9", 0x329);
        from_abi.insert("mhpmevent10", 0x32A);
        from_abi.insert("mhpmevent11", 0x32B);
        from_abi.insert("mhpmevent12", 0x32C);
        from_abi.insert("mhpmevent13", 0x32D);
        from_abi.insert("mhpmevent14", 0x32E);
        from_abi.insert("mhpmevent15", 0x32F);
        from_abi.insert("mhpmevent16", 0x330);
        from_abi.insert("mhpmevent17", 0x331);
        from_abi.insert("mhpmevent18", 0x332);
        from_abi.insert("mhpmevent19", 0x333);
        from_abi.insert("mhpmevent20", 0x334);
        from_abi.insert("mhpmevent21", 0x335);
        from_abi.insert("mhpmevent22", 0x336);
        from_abi.insert("mhpmevent23", 0x337);
        from_abi.insert("mhpmevent24", 0x338);
        from_abi.insert("mhpmevent25", 0x339);
        from_abi.insert("mhpmevent26", 0x33A);
        from_abi.insert("mhpmevent27", 0x33B);
        from_abi.insert("mhpmevent28", 0x33C);
        from_abi.insert("mhpmevent29", 0x33D);
        from_abi.insert("mhpmevent30", 0x33E);
        from_abi.insert("mhpmevent31", 0x33F);

        from_abi.insert("MSCRATCH", 0x340);
        from_abi.insert("MEPC", 0x341);
        from_abi.insert("MCAUSE", 0x342);
        from_abi.insert("MTVAL", 0x343);
        from_abi.insert("MIP", 0x344);
        from_abi.insert("MTINST", 0x34A);
        from_abi.insert("MTVAL2", 0x34B);

        from_abi.insert("pmpcfg0", 0x3A0);
        from_abi.insert("pmpcfg1", 0x3A1);
        from_abi.insert("pmpcfg2", 0x3A2);
        from_abi.insert("pmpcfg3", 0x3A3);
        from_abi.insert("pmpcfg4", 0x3A4);
        from_abi.insert("pmpcfg5", 0x3A5);
        from_abi.insert("pmpcfg6", 0x3A6);
        from_abi.insert("pmpcfg7", 0x3A7);
        from_abi.insert("pmpcfg8", 0x3A8);
        from_abi.insert("pmpcfg9", 0x3A9);
        from_abi.insert("pmpcfg10", 0x3AA);
        from_abi.insert("pmpcfg11", 0x3AB);
        from_abi.insert("pmpcfg12", 0x3AC);
        from_abi.insert("pmpcfg13", 0x3AD);
        from_abi.insert("pmpcfg14", 0x3AE);
        from_abi.insert("pmpcfg15", 0x3AF);

        from_abi.insert("pmpaddr0", 0x3B0);
        from_abi.insert("pmpaddr1", 0x3B1);
        from_abi.insert("pmpaddr2", 0x3B2);
        from_abi.insert("pmpaddr3", 0x3B3);
        from_abi.insert("pmpaddr4", 0x3B4);
        from_abi.insert("pmpaddr5", 0x3B5);
        from_abi.insert("pmpaddr6", 0x3B6);
        from_abi.insert("pmpaddr7", 0x3B7);
        from_abi.insert("pmpaddr8", 0x3B8);
        from_abi.insert("pmpaddr9", 0x3B9);
        from_abi.insert("pmpaddr10", 0x3BA);
        from_abi.insert("pmpaddr11", 0x3BB);
        from_abi.insert("pmpaddr12", 0x3BC);
        from_abi.insert("pmpaddr13", 0x3BD);
        from_abi.insert("pmpaddr14", 0x3BE);
        from_abi.insert("pmpaddr15", 0x3BF);
        from_abi.insert("pmpaddr16", 0x3C0);
        from_abi.insert("pmpaddr17", 0x3C1);
        from_abi.insert("pmpaddr18", 0x3C2);
        from_abi.insert("pmpaddr19", 0x3C3);
        from_abi.insert("pmpaddr20", 0x3C4);
        from_abi.insert("pmpaddr21", 0x3C5);
        from_abi.insert("pmpaddr22", 0x3C6);
        from_abi.insert("pmpaddr23", 0x3C7);
        from_abi.insert("pmpaddr24", 0x3C8);
        from_abi.insert("pmpaddr25", 0x3C9);
        from_abi.insert("pmpaddr26", 0x3CA);
        from_abi.insert("pmpaddr27", 0x3CB);
        from_abi.insert("pmpaddr28", 0x3CC);
        from_abi.insert("pmpaddr29", 0x3CD);
        from_abi.insert("pmpaddr30", 0x3CE);
        from_abi.insert("pmpaddr31", 0x3CF);
        from_abi.insert("pmpaddr32", 0x3D0);
        from_abi.insert("pmpaddr33", 0x3D1);
        from_abi.insert("pmpaddr34", 0x3D2);
        from_abi.insert("pmpaddr35", 0x3D3);
        from_abi.insert("pmpaddr36", 0x3D4);
        from_abi.insert("pmpaddr37", 0x3D5);
        from_abi.insert("pmpaddr38", 0x3D6);
        from_abi.insert("pmpaddr39", 0x3D7);
        from_abi.insert("pmpaddr40", 0x3D8);
        from_abi.insert("pmpaddr41", 0x3D9);
        from_abi.insert("pmpaddr42", 0x3DA);
        from_abi.insert("pmpaddr43", 0x3DB);
        from_abi.insert("pmpaddr44", 0x3DC);
        from_abi.insert("pmpaddr45", 0x3DD);
        from_abi.insert("pmpaddr46", 0x3DE);
        from_abi.insert("pmpaddr47", 0x3DF);
        from_abi.insert("pmpaddr48", 0x3E0);
        from_abi.insert("pmpaddr49", 0x3E1);
        from_abi.insert("pmpaddr50", 0x3E2);
        from_abi.insert("pmpaddr51", 0x3E3);
        from_abi.insert("pmpaddr52", 0x3E4);
        from_abi.insert("pmpaddr53", 0x3E5);
        from_abi.insert("pmpaddr54", 0x3E6);
        from_abi.insert("pmpaddr55", 0x3E7);
        from_abi.insert("pmpaddr56", 0x3E8);
        from_abi.insert("pmpaddr57", 0x3E9);
        from_abi.insert("pmpaddr58", 0x3EA);
        from_abi.insert("pmpaddr59", 0x3EB);
        from_abi.insert("pmpaddr60", 0x3EC);
        from_abi.insert("pmpaddr61", 0x3ED);
        from_abi.insert("pmpaddr62", 0x3EE);
        from_abi.insert("pmpaddr63", 0x3EF);

        from_abi.insert("TSELECT", 0x7a0);
        from_abi.insert("TDATA1", 0x7a1);
        from_abi.insert("TDATA2", 0x7a2);
        from_abi.insert("TDATA3", 0x7a3);

        from_abi.insert("DCSR", 0x7b0);
        from_abi.insert("DPC", 0x7b1);
        from_abi.insert("DSCRATCH0", 0x7b2);
        from_abi.insert("DSCRATCH1", 0x7b3);

        from_abi.insert("MCYCLE", 0xb00);
        from_abi.insert("MINSTRET", 0xb02);
        from_abi.insert("mpcounter3", 0xB03);
        from_abi.insert("mpcounter4", 0xB04);
        from_abi.insert("mpcounter5", 0xB05);
        from_abi.insert("mpcounter6", 0xB06);
        from_abi.insert("mpcounter7", 0xB07);
        from_abi.insert("mpcounter8", 0xB08);
        from_abi.insert("mpcounter9", 0xB09);
        from_abi.insert("mpcounter10", 0xB0A);
        from_abi.insert("mpcounter11", 0xB0B);
        from_abi.insert("mpcounter12", 0xB0C);
        from_abi.insert("mpcounter13", 0xB0D);
        from_abi.insert("mpcounter14", 0xB0E);
        from_abi.insert("mpcounter15", 0xB0F);
        from_abi.insert("mpcounter16", 0xB10);
        from_abi.insert("mpcounter17", 0xB11);
        from_abi.insert("mpcounter18", 0xB12);
        from_abi.insert("mpcounter19", 0xB13);
        from_abi.insert("mpcounter20", 0xB14);
        from_abi.insert("mpcounter21", 0xB15);
        from_abi.insert("mpcounter22", 0xB16);
        from_abi.insert("mpcounter23", 0xB17);
        from_abi.insert("mpcounter24", 0xB18);
        from_abi.insert("mpcounter25", 0xB19);
        from_abi.insert("mpcounter26", 0xB1A);
        from_abi.insert("mpcounter27", 0xB1B);
        from_abi.insert("mpcounter28", 0xB1C);
        from_abi.insert("mpcounter29", 0xB1D);
        from_abi.insert("mpcounter30", 0xB1E);
        from_abi.insert("mpcounter31", 0xB1F);

        from_abi.insert("MCYCLEH", 0xb80);
        from_abi.insert("MINSTRETH", 0xb82);
        from_abi.insert("mhpcounter3h", 0xB83);
        from_abi.insert("mhpcounter4h", 0xB84);
        from_abi.insert("mhpcounter5h", 0xB85);
        from_abi.insert("mhpcounter6h", 0xB86);
        from_abi.insert("mhpcounter7h", 0xB87);
        from_abi.insert("mhpcounter8h", 0xB88);
        from_abi.insert("mhpcounter9h", 0xB89);
        from_abi.insert("mhpcounter10h", 0xB8A);
        from_abi.insert("mhpcounter11h", 0xB8B);
        from_abi.insert("mhpcounter12h", 0xB8C);
        from_abi.insert("mhpcounter13h", 0xB8D);
        from_abi.insert("mhpcounter14h", 0xB8E);
        from_abi.insert("mhpcounter15h", 0xB8F);
        from_abi.insert("mhpcounter16h", 0xB90);
        from_abi.insert("mhpcounter17h", 0xB91);
        from_abi.insert("mhpcounter18h", 0xB92);
        from_abi.insert("mhpcounter19h", 0xB93);
        from_abi.insert("mhpcounter20h", 0xB94);
        from_abi.insert("mhpcounter21h", 0xB95);
        from_abi.insert("mhpcounter22h", 0xB96);
        from_abi.insert("mhpcounter23h", 0xB97);
        from_abi.insert("mhpcounter24h", 0xB98);
        from_abi.insert("mhpcounter25h", 0xB99);
        from_abi.insert("mhpcounter26h", 0xB9A);
        from_abi.insert("mhpcounter27h", 0xB9B);
        from_abi.insert("mhpcounter28h", 0xB9C);
        from_abi.insert("mhpcounter29h", 0xB9D);
        from_abi.insert("mhpcounter30h", 0xB9E);
        from_abi.insert("mhpcounter31h", 0xB9F);
        
        from_abi.insert("CYCLE", 0xc00);
        from_abi.insert("TIME", 0xc01);
        from_abi.insert("INSTRET", 0xc02);
        from_abi.insert("CYCLEH", 0xc80);
        from_abi.insert("TIMEH", 0xc81);
        from_abi.insert("INSTRETH", 0xc82);

        from_abi.insert("MVENDORID", 0xF11);
        from_abi.insert("MARCHID", 0xF12);
        from_abi.insert("MIMPID", 0xF13);
        from_abi.insert("MHARTID", 0xF14);

        Register { from_abi }
    }

    pub fn get_register_value(&self, name:Option<&String>) -> Result<i32, AsmError> {
        if let Some(s) = name {
            let r = self.from_abi.get(s.to_uppercase().as_str());
            if let Some(v) = r {
                Ok(v.clone())
            }
            else {
                let s = format!("cannot find register with name {s}");
                error_string(s.clone());
                Err(AsmError::NoFound((file!(), line!()).into(), s))
            }
        }
        else {
            Ok(0)
        }
    }

    pub fn get_register_compressed_value(&self, name:Option<&String>) -> Result<u32, AsmError> {
        if let Some(n) = name {
            match n.to_lowercase().as_str() {
                "x8" | "x9" | "x10" | "x11" | "x12" | "x13" | "x14" | "x15" |
                "s0" | "s1" | "a0" | "a1" | "a2" | "a3" | "a4" | "a5" |
                "f8" | "f9" | "f10" | "f11" | "f12" | "f13" | "f14" | "f15" |
                "fs0" | "fs1" | "fa0" | "fa1" | "fa2" | "fa3" | "fa4" | "fa5" => {
                    let v = self.get_register_value(name)? as u32;
                    Ok(v & 0b111)
                }
                _ => {
                    Err(AsmError::NotSupportedOperation((file!(), line!()).into(), format!("cannot find compressed value from register {n}")))
                }
            }
        }
        else {
            Err(AsmError::NoFound((file!(), line!()).into(), format!("need register name to get compressed value")))
        }
    }

    pub fn is_compact_reg(&self, name:Option<&String>) -> bool {
        self.get_register_compressed_value(name).is_ok()
    }

    pub fn is_register_name<S>(&self, name:S) -> bool 
        where S : AsRef<str> {
        let s = name.as_ref().to_uppercase();
        self.from_abi.contains_key(s.as_str())
    }

    /// Returns `true` if `name` refers to a floating-point register
    /// (F0–F31, FT0–FT11, FS0–FS11, FA0–FA7).
    pub fn is_float_register_name<S>(name: S) -> bool
    where
        S: AsRef<str>,
    {
        let normalized = name.as_ref().trim().to_ascii_uppercase();
        (Self::has_numeric_suffix(&normalized, 1) && normalized.starts_with('F'))
            || matches!(
                normalized.as_str(),
                "FT0" | "FT1" | "FT2" | "FT3" | "FT4" | "FT5" | "FT6" | "FT7"
                    | "FT8" | "FT9" | "FT10" | "FT11"
                    | "FS0" | "FS1" | "FS2" | "FS3" | "FS4" | "FS5" | "FS6" | "FS7"
                    | "FS8" | "FS9" | "FS10" | "FS11"
                    | "FA0" | "FA1" | "FA2" | "FA3" | "FA4" | "FA5" | "FA6" | "FA7"
            )
    }

    /// Returns `true` if `name` refers to a vector register (V0–V31).
    pub fn is_vector_register_name<S>(name: S) -> bool
    where
        S: AsRef<str>,
    {
        let normalized = name.as_ref().trim().to_ascii_uppercase();
        Self::has_numeric_suffix(&normalized, 1) && normalized.starts_with('V')
    }

    fn has_numeric_suffix(name: &str, prefix_len: usize) -> bool {
        name.len() > prefix_len && name[prefix_len..].chars().all(|ch| ch.is_ascii_digit())
    }
}
