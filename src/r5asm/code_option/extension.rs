//! RISC-V extension set: names, bitset, implication closure, canonical ISA-string order.
//!
//! Single source of truth is the `define_exts!` list. Add an extension by adding one line.

macro_rules! define_exts {
    ($($variant:ident => $name:literal),* $(,)?) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
        #[repr(u8)]
        pub enum Ext { $($variant),* }

        impl Ext {
            pub const ALL: &'static [Ext] = &[$(Ext::$variant),*];
            pub const COUNT: usize = Ext::ALL.len();

            pub fn name(self) -> &'static str {
                match self { $(Ext::$variant => $name),* }
            }

            /// Lowercase name as written in an ISA string / `.option arch`.
            pub fn from_name(s: &str) -> Option<Ext> {
                match s { $($name => Some(Ext::$variant),)* _ => None }
            }
        }
    };
}

define_exts! {
    // ---- single-letter ----
    I => "i", E => "e", M => "m", A => "a", F => "f", D => "d", Q => "q",
    C => "c", B => "b", V => "v", H => "h", G => "g",

    // ---- vector ----
    Zve32x => "zve32x", Zve32f => "zve32f", Zve64x => "zve64x", Zve64f => "zve64f", Zve64d => "zve64d",
    Zvl32b => "zvl32b", Zvl64b => "zvl64b", Zvl128b => "zvl128b", Zvl256b => "zvl256b",
    Zvl512b => "zvl512b", Zvl1024b => "zvl1024b", Zvl2048b => "zvl2048b", Zvl4096b => "zvl4096b",
    Zvl8192b => "zvl8192b", Zvl16384b => "zvl16384b", Zvl32768b => "zvl32768b", Zvl65536b => "zvl65536b",
    Zvfh => "zvfh", Zvfhmin => "zvfhmin", Zvfbfmin => "zvfbfmin", Zvfbfwma => "zvfbfwma",
    Zvbb => "zvbb", Zvbc => "zvbc",
    Zvkb => "zvkb", Zvkg => "zvkg", Zvkned => "zvkned", Zvknha => "zvknha", Zvknhb => "zvknhb",
    Zvksed => "zvksed", Zvksh => "zvksh",
    Zvkn => "zvkn", Zvknc => "zvknc", Zvkng => "zvkng", Zvks => "zvks", Zvksc => "zvksc",
    Zvksg => "zvksg", Zvkt => "zvkt",

    // ---- bit manipulation ----
    Zba => "zba", Zbb => "zbb", Zbc => "zbc", Zbs => "zbs", Zbkb => "zbkb", Zbkc => "zbkc", Zbkx => "zbkx",

    // ---- atomics / memory ----
    Zaamo => "zaamo", Zalrsc => "zalrsc", Zabha => "zabha", Zacas => "zacas", Zalasr => "zalasr",
    Zawrs => "zawrs", Zama16b => "zama16b", Ztso => "ztso",
    Zicbom => "zicbom", Zicbop => "zicbop", Zicboz => "zicboz",

    // ---- floating point / multiply ----
    Zfh => "zfh", Zfhmin => "zfhmin", Zfbfmin => "zfbfmin", Zfa => "zfa",
    Zfinx => "zfinx", Zdinx => "zdinx", Zhinx => "zhinx", Zhinxmin => "zhinxmin",
    Zmmul => "zmmul",

    // ---- compressed ----
    Zca => "zca", Zcb => "zcb", Zcd => "zcd", Zce => "zce", Zcf => "zcf", Zclsd => "zclsd",
    Zcmop => "zcmop", Zcmp => "zcmp", Zcmt => "zcmt",

    // ---- base-adjacent, hints, CFI ----
    Zicsr => "zicsr", Zifencei => "zifencei", Zicntr => "zicntr", Zihpm => "zihpm", Zicond => "zicond",
    Zihintpause => "zihintpause", Zihintntl => "zihintntl", Zimop => "zimop", Zilsd => "zilsd",
    Zicfilp => "zicfilp", Zicfiss => "zicfiss",

    // ---- profile / behaviour declarations ----
    Zic64b => "zic64b", Ziccamoa => "ziccamoa", Ziccif => "ziccif", Zicclsm => "zicclsm",
    Ziccrse => "ziccrse", Za64rs => "za64rs", Za128rs => "za128rs",

    // ---- scalar crypto ----
    Zk => "zk", Zkn => "zkn", Zknd => "zknd", Zkne => "zkne", Zknh => "zknh", Zkr => "zkr",
    Zks => "zks", Zksed => "zksed", Zksh => "zksh", Zkt => "zkt",

    // ---- supervisor / virtual memory ----
    Sstc => "sstc", Ssaia => "ssaia", Sscofpmf => "sscofpmf", Sscounterenw => "sscounterenw",
    Sscsrind => "sscsrind", Ssccptr => "ssccptr", Ssccfg => "ssccfg", Ssdbltrp => "ssdbltrp",
    Ssnpm => "ssnpm", Sspm => "sspm", Ssqosid => "ssqosid", Ssstateen => "ssstateen",
    Ssstrict => "ssstrict", Sstvala => "sstvala", Sstvecd => "sstvecd", Ssu64xl => "ssu64xl", Supm => "supm",
    Svade => "svade", Svadu => "svadu", Svbare => "svbare", Svinval => "svinval", Svnapot => "svnapot",
    Svpbmt => "svpbmt", Svvptc => "svvptc",

    // ---- machine / hypervisor ----
    Smaia => "smaia", Smcdeleg => "smcdeleg", Smcntrpmf => "smcntrpmf", Smcsrind => "smcsrind",
    Smdbltrp => "smdbltrp", Smepmp => "smepmp", Smmpm => "smmpm", Smnpm => "smnpm",
    Smrnmi => "smrnmi", Smstateen => "smstateen",
    Shcounterenw => "shcounterenw", Shgatpa => "shgatpa", Shtvala => "shtvala",
    Shvsatpa => "shvsatpa", Shvstvala => "shvstvala", Shvstvecd => "shvstvecd",

    // ---- debug / trace ----
    Sdext => "sdext", Sdtrig => "sdtrig",
}

// ---------------------------------------------------------------------------
// Bitset
// ---------------------------------------------------------------------------

const WORDS: usize = (Ext::COUNT + 63) / 64;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ExtSet([u64; WORDS]);

impl ExtSet {
    pub fn has(&self, e: Ext) -> bool {
        let i = e as usize;
        (self.0[i / 64] >> (i % 64)) & 1 == 1
    }

    pub fn insert(&mut self, e: Ext) {
        let i = e as usize;
        self.0[i / 64] |= 1 << (i % 64);
    }

    pub fn remove(&mut self, e: Ext) {
        let i = e as usize;
        self.0[i / 64] &= !(1 << (i % 64));
    }
    
    pub fn union(&mut self, other: &ExtSet) {
        for (a, b) in self.0.iter_mut().zip(other.0.iter()) {
            *a |= *b;
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = Ext> + '_ {
        Ext::ALL.iter().copied().filter(move |e| self.has(*e))
    }

    pub fn from_slice(exts: &[Ext]) -> ExtSet {
        let mut s = ExtSet::default();
        for e in exts {
            s.insert(*e);
        }
        s
    }

    /// What GNU as assumes without -march: I M A F D C Zicsr Zifencei (= "gc").
    /// XLEN-independent; `closure` adds Zcf (rv32) or nothing extra (rv64) later.
    pub fn gc() -> ExtSet {
        ExtSet::from_slice(&[
            Ext::I, Ext::M, Ext::A, Ext::F, Ext::D, Ext::C, Ext::Zicsr, Ext::Zifencei,
        ])
    }
}

impl Default for ExtSet {
    fn default() -> Self {
        Self::gc()
    }
}

// ---------------------------------------------------------------------------
// Implications: (ext, &[things it turns on])
// Unconditional rules only. Verify against the spec / your reference assembler.
// ---------------------------------------------------------------------------

use Ext::*;

const IMPLIES: &[(Ext, &[Ext])] = &[
    // shorthand
    (G, &[I, M, A, F, D, Zicsr, Zifencei]),
    (B, &[Zba, Zbb, Zbs]),
    (M, &[Zmmul]),
    (A, &[Zaamo, Zalrsc]),
    (Zacas, &[Zaamo]),
    (Zabha, &[Zaamo]),
    // float
    (D, &[F]),
    (Q, &[D]),
    (F, &[Zicsr]),
    (Zfh, &[Zfhmin]),
    (Zfhmin, &[F]),
    (Zfbfmin, &[F]),
    (Zfa, &[F]),
    (Zdinx, &[Zfinx]),
    (Zhinx, &[Zhinxmin]),
    (Zhinxmin, &[Zfinx]),
    (Zfinx, &[Zicsr]),
    // vector
    (V, &[Zve64d, Zvl128b]),
    (Zve64d, &[Zve64f, D]),
    (Zve64f, &[Zve32f, Zve64x, F]),
    (Zve64x, &[Zve32x, Zvl64b]),
    (Zve32f, &[Zve32x, F]),
    (Zve32x, &[Zicsr, Zvl32b]),
    (Zvl65536b, &[Zvl32768b]),
    (Zvl32768b, &[Zvl16384b]),
    (Zvl16384b, &[Zvl8192b]),
    (Zvl8192b, &[Zvl4096b]),
    (Zvl4096b, &[Zvl2048b]),
    (Zvl2048b, &[Zvl1024b]),
    (Zvl1024b, &[Zvl512b]),
    (Zvl512b, &[Zvl256b]),
    (Zvl256b, &[Zvl128b]),
    (Zvl128b, &[Zvl64b]),
    (Zvl64b, &[Zvl32b]),
    (Zvfh, &[Zve32f, Zfhmin]),
    (Zvfhmin, &[Zve32f]),
    (Zvfbfmin, &[Zve32f]),
    (Zvfbfwma, &[Zvfbfmin, Zfbfmin]),
    (Zvbb, &[Zvkb]),
    (Zvbc, &[Zve64x]),
    (Zvkb, &[Zve32x]),
    (Zvkg, &[Zve32x]),
    (Zvkned, &[Zve32x]),
    (Zvknha, &[Zve32x]),
    (Zvknhb, &[Zve64x]),
    (Zvksed, &[Zve32x]),
    (Zvksh, &[Zve32x]),
    (Zvkn, &[Zvkned, Zvknhb, Zvkb, Zvkt]),
    (Zvknc, &[Zvkn, Zvbc]),
    (Zvkng, &[Zvkn, Zvkg]),
    (Zvks, &[Zvksed, Zvksh, Zvkb, Zvkt]),
    (Zvksc, &[Zvks, Zvbc]),
    (Zvksg, &[Zvks, Zvkg]),
    // compressed (conditional parts are in `conditional_implies`)
    (C, &[Zca]),
    (Zcb, &[Zca]),
    (Zcd, &[Zca]),
    (Zcf, &[Zca]),
    (Zcmp, &[Zca]),
    (Zcmt, &[Zca, Zicsr]),
    (Zce, &[Zca, Zcb, Zcmp, Zcmt]),
    // counters
    (Zicntr, &[Zicsr]),
    (Zihpm, &[Zicsr]),
    // scalar crypto
    (Zk, &[Zkn, Zkr, Zkt]),
    (Zkn, &[Zbkb, Zbkc, Zbkx, Zkne, Zknd, Zknh]),
    (Zks, &[Zbkb, Zbkc, Zbkx, Zksed, Zksh]),
];

/// Rules that depend on XLEN or on what else is enabled.
fn conditional_implies(e: Ext, set: &ExtSet, xlen: u8, out: &mut Vec<Ext>) {
    if e == C {
        if set.has(F) && xlen == 32 { out.push(Zcf); }
        if set.has(D) { out.push(Zcd); }
    }
}

/// Expand an explicit set to the effective set (fixpoint over implications).
pub fn closure(explicit: ExtSet, xlen: u8) -> ExtSet {
    let mut out = explicit;
    loop {
        let before = out;
        for (e, imps) in IMPLIES {
            if out.has(*e) {
                for i in *imps { out.insert(*i); }
            }
        }
        let mut extra = Vec::new();
        for e in before.iter() { conditional_implies(e, &out, xlen, &mut extra); }
        for e in extra { out.insert(e); }
        if out == before { return out; }
    }
}

// ---------------------------------------------------------------------------
// Canonical ISA-string order (for .riscv.attributes / `-march=` output)
// single letters in "imafdqlcbkjtpvh" order, then z* by category then alpha,
// then s*, then x*.
// ---------------------------------------------------------------------------

const SINGLE_ORDER: &str = "iemafdqlcbkjtpvhg";
const Z_CATEGORY_ORDER: &str = "iamfdqlcbkjtpvh";

fn canonical_key(e: Ext) -> (u8, usize, &'static str) {
    let n = e.name();
    let b = n.as_bytes();
    if n.len() == 1 {
        (0, SINGLE_ORDER.find(n).unwrap_or(99), n)
    } else if b[0] == b'z' {
        (1, Z_CATEGORY_ORDER.find(b[1] as char).unwrap_or(99), n)
    } else if b[0] == b's' {
        (2, 0, n)
    } else {
        (3, 0, n)
    }
}

/// `+g` style shorthands are expanded in the effective set, so skip them when printing.
pub fn isa_string(set: &ExtSet, xlen: u8) -> String {
    let mut v: Vec<Ext> = set.iter().filter(|e| *e != G).collect();
    v.sort_by_key(|e| canonical_key(*e));
    let mut s = format!("rv{}", xlen);
    let mut prev_single = true;
    for e in v {
        let n = e.name();
        if n.len() > 1 && prev_single && !s.ends_with(|c: char| c.is_ascii_digit()) {
            s.push('_');
        } else if n.len() > 1 {
            s.push('_');
        }
        s.push_str(n);
        prev_single = n.len() == 1;
    }
    s
}

// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        for e in Ext::ALL {
            assert_eq!(Ext::from_name(e.name()), Some(*e));
        }
    }

    #[test]
    fn d_brings_f() {
        let mut s = ExtSet::default();
        s.insert(D);
        let eff = closure(s, 64);
        assert!(eff.has(F) && eff.has(Zicsr));
        assert!(!s.has(F));
    }

    #[test]
    fn v_chain() {
        let mut s = ExtSet::default();
        s.insert(V);
        let eff = closure(s, 64);
        for e in [Zve64d, Zve64f, Zve64x, Zve32x, Zvl128b, Zvl32b, D, F, Zicsr] {
            assert!(eff.has(e), "{:?}", e);
        }
    }

    #[test]
    fn c_is_conditional() {
        let mut s = ExtSet::default();
        s.insert(C);
        s.insert(D);
        let eff = closure(s, 64);
        assert!(eff.has(Zca) && eff.has(Zcd) && !eff.has(Zcf));
    }

    #[test]
    fn remove_then_recompute() {
        let mut explicit = ExtSet::default();
        explicit.insert(D);
        assert!(closure(explicit, 64).has(F));
        explicit.remove(D);
        assert!(!closure(explicit, 64).has(F));
    }
}