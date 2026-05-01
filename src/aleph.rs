//! ℵ-OS λ_ℵ type system — 22 Hebrew letter encodings + lattice operations.
//!
//! Each letter is a 12-primitive tuple [D,T,R,P,F,K,G,Γ,Φ,H,S,Ω].

extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;

pub type Tuple = [u8; 12];

pub const PRIM_NAMES: [&str; 12] =
    ["D", "T", "R", "P", "F", "K", "G", "Gamma", "Phi", "H", "S", "Omega"];

// ── Value-name maps for formatting ──────────────────────────────────────────

pub const PHI_NAMES: [&str; 5] = ["Phi_sub", "Phi_c", "Phi_c_complex", "Phi_EP", "Phi_super"];
pub const OMEGA_NAMES: [&str; 3] = ["Omega_0", "Omega_Z2", "Omega_Z"];
pub const P_NAMES: [&str; 5] = ["P_asym", "P_psi", "P_pm", "P_sym", "P_pm_sym"];
pub const TIER_NAMES: [&str; 5] = ["O_0", "O_1", "O_2", "O_2d", "O_inf"];

// ── Tier enum ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tier {
    O0,
    O1,
    O2,
    O2d,
    OInf,
}

impl core::fmt::Display for Tier {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", tier_name(*self))
    }
}

pub fn tier_name(t: Tier) -> &'static str {
    match t {
        Tier::O0   => "O_0",
        Tier::O1   => "O_1",
        Tier::O2   => "O_2",
        Tier::O2d  => "O_2d",
        Tier::OInf => "O_inf",
    }
}

// ── Letter definition with Hebrew glyph ───────────────────────────────────────

#[derive(Debug)]
pub struct LetterDef {
    pub name: &'static str,
    pub glyph: char,
    pub t: Tuple,
}

pub const LETTERS: [LetterDef; 22] = [
    LetterDef { name: "aleph",  glyph: 'א', t: [0,3,0,3,2,2,2,0,1,3,0,2] },
    LetterDef { name: "bet",    glyph: 'ב', t: [1,3,1,2,1,1,1,0,0,1,1,1] },
    LetterDef { name: "gimel",  glyph: 'ג', t: [0,2,3,0,0,0,0,2,0,0,0,0] },
    LetterDef { name: "dalet",  glyph: 'ד', t: [0,1,3,0,0,0,0,2,0,0,0,0] },
    LetterDef { name: "hei",    glyph: 'ה', t: [3,4,2,3,2,2,2,3,1,3,2,2] },
    LetterDef { name: "vav",    glyph: 'ו', t: [0,0,3,4,0,2,1,0,1,1,0,0] },
    LetterDef { name: "zayin",  glyph: 'ז', t: [0,0,3,0,0,0,0,2,0,0,0,0] },
    LetterDef { name: "chet",   glyph: 'ח', t: [1,3,1,2,1,1,1,0,0,1,1,1] },
    LetterDef { name: "tet",    glyph: 'ט', t: [1,1,3,0,0,2,1,2,0,1,0,0] },
    LetterDef { name: "yod",    glyph: 'י', t: [0,3,0,3,2,2,2,0,0,1,0,0] },
    LetterDef { name: "kaf",    glyph: 'כ', t: [1,3,1,2,1,1,1,0,0,1,1,1] },
    LetterDef { name: "lamed",  glyph: 'ל', t: [2,0,3,0,0,1,0,2,1,2,2,0] },
    LetterDef { name: "mem",    glyph: 'מ', t: [1,1,2,4,2,2,2,3,1,2,1,2] },
    LetterDef { name: "nun",    glyph: 'נ', t: [0,0,3,0,0,0,0,2,0,0,0,0] },
    LetterDef { name: "samech", glyph: 'ס', t: [1,3,1,3,1,1,1,0,0,1,1,1] },
    LetterDef { name: "ayin",   glyph: 'ע', t: [3,4,2,2,2,2,2,3,1,2,2,2] },
    LetterDef { name: "pei",    glyph: 'פ', t: [0,0,3,0,0,0,0,3,0,1,2,0] },
    LetterDef { name: "tzadi",  glyph: 'צ', t: [0,1,3,0,0,0,0,2,0,0,0,0] },
    LetterDef { name: "kuf",    glyph: 'ק', t: [1,3,1,3,1,2,1,0,1,2,1,1] },
    LetterDef { name: "resh",   glyph: 'ר', t: [0,3,3,0,0,1,0,0,0,1,0,0] },
    LetterDef { name: "shin",   glyph: 'ש', t: [1,2,2,4,2,2,2,3,1,3,1,2] },
    LetterDef { name: "tav",    glyph: 'ת', t: [1,3,1,3,1,2,1,0,1,3,1,2] },
];

pub const HEBREW_LO: u32 = 0x0590;
pub const HEBREW_HI: u32 = 0x05EA;

pub fn compute_tier(t: &Tuple) -> Tier {
    let (d, _t, _r, p, _f, _k, _g, _ga, phi, _h, _s, omega) =
        (t[0], t[1], t[2], t[3], t[4], t[5], t[6], t[7], t[8], t[9], t[10], t[11]);
    if phi == 1 && p == 4 { return Tier::OInf; }
    if phi == 0 || phi >= 3 { return Tier::O0; }
    if omega == 0 { return Tier::O1; }
    if d == 0 || d == 1 || d == 3 { return Tier::O2; }
    if d == 2 { return Tier::O2d; }
    Tier::O0
}

// ── Lattice operations ───────────────────────────────────────────────────────

fn tensor_s(a: u8, b: u8) -> u8 {
    if a == 2 || b == 2 { 2 }           // n:m absorbs
    else if a == 0 && b == 0 { 0 }      // 1:1 ⊗ 1:1 = 1:1
    else { 1 }                           // mixed → n:n
}

/// P-596: Coupling Destruction Rule
/// Φ_c ⊗ Φ_EP → Φ_EP
fn tensor_phi(a: u8, b: u8) -> u8 {
    if (a == 1 && b == 3) || (a == 3 && b == 1) {
        3 // Phi_EP absorbs Phi_c
    } else {
        a.max(b)
    }
}

/// ⊗ Tensor product: bottleneck (min) on P,F,K; union (max) elsewhere; special S and Phi.
pub fn tensor(a: &Tuple, b: &Tuple) -> Tuple {
    let mut r = [0u8; 12];
    for i in 0..12 {
        r[i] = match i {
            3 | 4 | 5 => a[i].min(b[i]),    // P, F, K: bottleneck
            8          => tensor_phi(a[i], b[i]), // Phi: absorption
            10         => tensor_s(a[i], b[i]), // S: stoichiometry
            _          => a[i].max(b[i]),    // union
        };
    }
    r
}

pub fn join(a: &Tuple, b: &Tuple) -> Tuple {
    let mut r = [0u8; 12];
    for i in 0..12 { r[i] = a[i].max(b[i]); }
    r
}

pub fn meet(a: &Tuple, b: &Tuple) -> Tuple {
    let mut r = [0u8; 12];
    for i in 0..12 { r[i] = a[i].min(b[i]); }
    r
}

pub fn mediate(witness: &Tuple, a: &Tuple, b: &Tuple) -> Tuple {
    let comp = tensor(a, b);
    join(witness, &comp)
}

// ── Distance ─────────────────────────────────────────────────────────────────

const WEIGHTS: [u64; 12] = [10000,10000,10000,12000,9000,8000,10000,10000,11000,8000,10000,7000];

fn isqrt(n: u64) -> u64 {
    if n == 0 { return 0; }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

pub fn distance_scaled(a: &Tuple, b: &Tuple) -> u32 {
    let sum: u64 = WEIGHTS.iter().enumerate().map(|(i, &w)| {
        let d = if a[i] > b[i] { (a[i] - b[i]) as u64 } else { (b[i] - a[i]) as u64 };
        w * d * d
    }).sum();
    isqrt(sum) as u32
}

pub fn distance(a: &Tuple, b: &Tuple) -> f64 {
    distance_scaled(a, b) as f64 / 100.0
}

pub fn conflict_set(a: &Tuple, b: &Tuple) -> Vec<usize> {
    let mut set = Vec::new();
    for i in 0..12 {
        if a[i] != b[i] {
            set.push(i);
        }
    }
    set
}

pub fn nearest_letter(t: &Tuple) -> &'static LetterDef {
    let mut best_idx = 0usize;
    let mut best_dist = u32::MAX;
    for (i, l) in LETTERS.iter().enumerate() {
        let d = distance_scaled(&l.t, t);
        if d < best_dist {
            best_dist = d;
            best_idx = i;
        }
    }
    &LETTERS[best_idx]
}

pub fn veracity_class(d: f64) -> &'static str {
    if d == 0.0 { "transparent" }
    else if d <= 1.4142 { "near-grounded" }
    else if d <= 2.4495 { "partial-emergence" }
    else { "aspirational" }
}

pub fn letter_by_name(name: &str) -> Option<&'static LetterDef> {
    for l in &LETTERS { if l.name.eq_ignore_ascii_case(name) { return Some(l); } }
    None
}

pub fn letter_by_glyph(glyph: char) -> Option<&'static LetterDef> {
    for l in &LETTERS { if l.glyph == glyph { return Some(l); } }
    None
}

pub fn resolve_letter(input: &str) -> Option<&'static LetterDef> {
    if input.chars().count() == 1 {
        if let Some(c) = input.chars().next() {
            if let Some(l) = letter_by_glyph(c) { return Some(l); }
        }
    }
    letter_by_name(input)
}

pub fn system_language() -> Tuple {
    let mut r = LETTERS[0].t;
    for l in &LETTERS[1..] { r = join(&r, &l.t); }
    r
}

pub fn tier_census() -> [u8; 5] {
    let mut counts = [0u8; 5];
    for l in &LETTERS {
        let idx = match compute_tier(&l.t) {
            Tier::O0 => 0, Tier::O1 => 1, Tier::O2 => 2, Tier::O2d => 3, Tier::OInf => 4,
        };
        counts[idx] += 1;
    }
    counts
}

pub fn is_hebrew(c: char) -> bool {
    let cp = c as u32;
    cp >= HEBREW_LO && cp <= HEBREW_HI
}

static HEBREW_GLYPH_BYTES: [[u8; 1]; 22] = [
    [0xE0], [0xE1], [0xE2], [0xE3], [0xE4], [0xE5],
    [0xE6], [0xE7], [0xE8], [0xE9], [0xEA], [0xEB],
    [0xEC], [0xED], [0xEE], [0xEF], [0xF0], [0xF1],
    [0xF2], [0xF3], [0xF4], [0xF5],
];

pub fn display_glyph(l: &LetterDef) -> &'static str {
    if crate::vga::get_display_mode() == crate::vga::DisplayMode::Framebuffer {
        let idx = LETTERS.iter().position(|x| x.name == l.name).unwrap_or(0);
        unsafe { core::str::from_utf8_unchecked(&HEBREW_GLYPH_BYTES[idx]) }
    } else {
        match l.name {
            "aleph"  => "A", "bet"    => "B", "gimel"  => "G", "dalet"  => "D",
            "hei"    => "H", "vav"    => "V", "zayin"  => "Z", "chet"   => "C",
            "tet"    => "T", "yod"    => "Y", "kaf"    => "K", "lamed"  => "L",
            "mem"    => "M", "nun"    => "N", "samech" => "S", "ayin"   => "E",
            "pei"    => "P", "tzadi"  => "Q", "kuf"    => "U", "resh"   => "R",
            "shin"   => "X", "tav"    => "O", _        => "?",
        }
    }
}

pub fn format_letter(l: &LetterDef) -> String {
    let tier = compute_tier(&l.t);
    let phi_n = PHI_NAMES.get(l.t[8] as usize).copied().unwrap_or("?");
    let om_n  = OMEGA_NAMES.get(l.t[11] as usize).copied().unwrap_or("?");
    let p_n   = P_NAMES.get(l.t[3] as usize).copied().unwrap_or("?");
    format!("  ->  {}\n    tier  {}\n    Phi  {}   Omega  {}   P  {}\n",
        display_glyph(l), tier_name(tier), phi_n, om_n, p_n)
}

pub fn format_tuple(l: &LetterDef) -> String {
    let mut s = format!("  {} ({})  tier={}\n", display_glyph(l), l.name, compute_tier(&l.t));
    for i in 0..12 {
        let val = l.t[i] as usize;
        let max_val = match i {
            0 => 3, 1 => 4, 2 => 3, 3 => 4, 4 => 2, 5 => 3,
            6 => 2, 7 => 3, 8 => 4, 9 => 3, 10 => 2, 11 => 2, _ => 4,
        };
        let filled = if max_val > 0 { (val * 10) / max_val } else { 0 };
        let empty = 10 - filled;
        let bar: String = "#".repeat(filled) + &"-".repeat(empty);
        let val_name = match i {
            0  => ["wedge","triangle","infty","holo"].get(l.t[i] as usize).copied().unwrap_or("?"),
            1  => ["network","in","bowtie","box","holo"].get(l.t[i] as usize).copied().unwrap_or("?"),
            2  => ["super","cat","dagger","lr"].get(l.t[i] as usize).copied().unwrap_or("?"),
            3  => P_NAMES.get(l.t[i] as usize).copied().unwrap_or("?"),
            4  => ["ell","eth","hbar"].get(l.t[i] as usize).copied().unwrap_or("?"),
            5  => ["fast","mod","slow","trap"].get(l.t[i] as usize).copied().unwrap_or("?"),
            6  => ["beth","gimel","aleph"].get(l.t[i] as usize).copied().unwrap_or("?"),
            7  => ["and","or","seq","broad"].get(l.t[i] as usize).copied().unwrap_or("?"),
            8  => PHI_NAMES.get(l.t[i] as usize).copied().unwrap_or("?"),
            9  => ["H0","H1","H2","H_inf"].get(l.t[i] as usize).copied().unwrap_or("?"),
            10 => ["1:1","n:n","n:m"].get(l.t[i] as usize).copied().unwrap_or("?"),
            11 => OMEGA_NAMES.get(l.t[i] as usize).copied().unwrap_or("?"), _  => "?",
        };
        s += &format!("    {}={}  {}  {}\n", PRIM_NAMES[i], l.t[i], bar, val_name);
    }
    s
}

pub fn format_distance(a: &LetterDef, b: &LetterDef) -> String {
    let d = distance(&a.t, &b.t);
    let cs = conflict_set(&a.t, &b.t);
    let vc = veracity_class(d);
    let mut s = format!("  d = {:.4}  [{}]\n", d, vc);
    if !cs.is_empty() {
        s += "  conflict_set: {";
        for (i, &idx) in cs.iter().enumerate() {
            if i > 0 { s += ", "; }
            s += PRIM_NAMES[idx];
        }
        s += "}\n";
    }
    s
}

pub fn format_explain(l: &LetterDef) -> String {
    let tier = compute_tier(&l.t);
    let phi_n = PHI_NAMES.get(l.t[8] as usize).copied().unwrap_or("?");
    let om_n = OMEGA_NAMES.get(l.t[11] as usize).copied().unwrap_or("?");
    let p_n = P_NAMES.get(l.t[3] as usize).copied().unwrap_or("?");
    let mut out = format!("  {} ({})\n  tier  {}\n", display_glyph(l), l.name, tier_name(tier));
    for i in 0..12 {
        out += &format!("  {:<8} {:>2}\n", PRIM_NAMES[i], l.t[i]);
    }
    out
}
