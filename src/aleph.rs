//! ℵ-OS λ_ℵ type system — 22 Hebrew letter encodings + lattice operations.
//!
//! Each letter is a 12-primitive tuple [D,T,R,P,F,K,G,Γ,Φ,H,S,Ω].
//!
//! Primitive ordinals:
//!   D:     wedge=0, triangle=1, infty=2, holo=3
//!   T:     network=0, in=1, bowtie=2, box=3, holo=4
//!   R:     super=0, cat=1, dagger=2, lr=3
//!   P:     asym=0, psi=1, pm=2, sym=3, pm_sym=4
//!   F:     ell=0, eth=1, hbar=2
//!   K:     fast=0, mod=1, slow=2, trap=3
//!   G:     beth=0, gimel=1, aleph=2
//!   Gamma: and=0, or=1, seq=2, broad=3
//!   Phi:   sub=0, c=1, c_complex=2, EP=3, super=4
//!   H:     0=0, 1=1, 2=2, inf=3
//!   S:     one_one=0, n_n=1, n_m=2
//!   Omega: 0=0, Z2=1, Z=2

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

// ── 22 canonical Hebrew letters (§2 + §2.1 Kabbalism revision, 2026-04-04) ──
//
// Index:  D  T  R  P  F  K  G Ga Ph  H  S  Om   Glyph
pub const LETTERS: [LetterDef; 22] = [
    LetterDef { name: "aleph",  glyph: 'א', t: [0,3,0,3,2,2,2,0,1,3,0,2] },  // א O_2
    LetterDef { name: "bet",    glyph: 'ב', t: [1,3,1,2,1,1,1,0,0,1,1,1] },  // ב O_0
    LetterDef { name: "gimel",  glyph: 'ג', t: [0,2,3,0,0,0,0,2,0,0,0,0] },  // ג O_0
    LetterDef { name: "dalet",  glyph: 'ד', t: [0,1,3,0,0,0,0,2,0,0,0,0] },  // ד O_0
    LetterDef { name: "hei",    glyph: 'ה', t: [3,4,2,3,2,2,2,3,1,3,2,2] },  // ה O_2
    LetterDef { name: "vav",    glyph: 'ו', t: [0,0,3,4,0,2,1,0,1,1,0,0] },  // ו O_inf
    LetterDef { name: "zayin",  glyph: 'ז', t: [0,0,3,0,0,0,0,2,0,0,0,0] },  // ז O_0
    LetterDef { name: "chet",   glyph: 'ח', t: [1,3,1,2,1,1,1,0,0,1,1,1] },  // ח O_0
    LetterDef { name: "tet",    glyph: 'ט', t: [1,1,3,0,0,2,1,2,0,1,0,0] },  // ט O_0
    LetterDef { name: "yod",    glyph: 'י', t: [0,3,0,3,2,2,2,0,0,1,0,0] },  // י O_0
    LetterDef { name: "kaf",    glyph: 'כ', t: [1,3,1,2,1,1,1,0,0,1,1,1] },  // כ O_0
    LetterDef { name: "lamed",  glyph: 'ל', t: [2,0,3,0,0,1,0,2,1,2,2,0] },  // ל O_1
    LetterDef { name: "mem",    glyph: 'מ', t: [1,1,2,4,2,2,2,3,1,2,1,2] },  // מ O_inf §2.1
    LetterDef { name: "nun",    glyph: 'נ', t: [0,0,3,0,0,0,0,2,0,0,0,0] },  // נ O_0
    LetterDef { name: "samech", glyph: 'ס', t: [1,3,1,3,1,1,1,0,0,1,1,1] },  // ס O_0
    LetterDef { name: "ayin",   glyph: 'ע', t: [3,4,2,2,2,2,2,3,1,2,2,2] },  // ע O_2
    LetterDef { name: "pei",    glyph: 'פ', t: [0,0,3,0,0,0,0,3,0,1,2,0] },  // פ O_0
    LetterDef { name: "tzadi",  glyph: 'צ', t: [0,1,3,0,0,0,0,2,0,0,0,0] },  // צ O_0
    LetterDef { name: "kuf",    glyph: 'ק', t: [1,3,1,3,1,2,1,0,1,2,1,1] },  // ק O_2
    LetterDef { name: "resh",   glyph: 'ר', t: [0,3,3,0,0,1,0,0,0,1,0,0] },  // ר O_0
    LetterDef { name: "shin",   glyph: 'ש', t: [1,2,2,4,2,2,2,3,1,3,1,2] },  // ש O_inf §2.1
    LetterDef { name: "tav",    glyph: 'ת', t: [1,3,1,3,1,2,1,0,1,3,1,2] },  // ת O_2
];

// Hebrew Unicode range for tokenizer
pub const HEBREW_LO: u32 = 0x0590;
pub const HEBREW_HI: u32 = 0x05EA;

/// Look up a letter by ASCII name (case-insensitive).
pub fn letter_by_name(name: &str) -> Option<&'static LetterDef> {
    for l in &LETTERS {
        if l.name.eq_ignore_ascii_case(name) {
            return Some(l);
        }
    }
    None
}

/// Look up a letter by Hebrew glyph.
pub fn letter_by_glyph(glyph: char) -> Option<&'static LetterDef> {
    for l in &LETTERS {
        if l.glyph == glyph {
            return Some(l);
        }
    }
    None
}

/// Resolve a letter by name, glyph, or alias.
pub fn resolve_letter(input: &str) -> Option<&'static LetterDef> {
    // Try as glyph first (single char)
    if input.chars().count() == 1 {
        if let Some(c) = input.chars().next() {
            if let Some(l) = letter_by_glyph(c) {
                return Some(l);
            }
        }
    }
    // Try as name
    letter_by_name(input)
}

/// Get the display glyph for a letter — ASCII transliteration for VGA text mode.
pub fn display_glyph(l: &LetterDef) -> &'static str {
    match l.name {
        "aleph"  => "A",
        "bet"    => "B",
        "gimel"  => "G",
        "dalet"  => "D",
        "hei"    => "H",
        "vav"    => "V",
        "zayin"  => "Z",
        "chet"   => "C",
        "tet"    => "T",
        "yod"    => "Y",
        "kaf"    => "K",
        "lamed"  => "L",
        "mem"    => "M",
        "nun"    => "N",
        "samech" => "S",
        "ayin"   => "E",
        "pei"    => "P",
        "tzadi"  => "Q",
        "kuf"    => "U",
        "resh"   => "R",
        "shin"   => "X",
        "tav"    => "O",
        _        => "?",
    }
}

// ── Ouroboricity tier (§ Ouroboricity tiers R1–R5) ──────────────────────────

pub fn compute_tier(t: &Tuple) -> Tier {
    let (d, _t, _r, p, _f, _k, _g, _ga, phi, _h, _s, omega) =
        (t[0], t[1], t[2], t[3], t[4], t[5], t[6], t[7], t[8], t[9], t[10], t[11]);
    // R1: Phi_c + P_pm_sym → O_inf
    if phi >= 1 && p == 4 { return Tier::OInf; }
    // R2: Phi_sub → O_0
    if phi == 0 { return Tier::O0; }
    // R3: Phi_c + Omega_0 → O_1
    if phi >= 1 && omega == 0 { return Tier::O1; }
    // R4: Phi_c + Omega≠0 + D in {wedge,triangle,holo}
    if phi >= 1 && omega > 0 && (d == 0 || d == 1 || d == 3) { return Tier::O2; }
    // R5: Phi_c + Omega≠0 + D_infty
    if phi >= 1 && omega > 0 && d == 2 { return Tier::O2d; }
    Tier::O0
}

// ── Lattice operations ───────────────────────────────────────────────────────

fn tensor_s(a: u8, b: u8) -> u8 {
    if a == 2 || b == 2 { 2 }           // n:m absorbs
    else if a == 0 && b == 0 { 0 }      // 1:1 ⊗ 1:1 = 1:1
    else { 1 }                           // mixed → n:n
}

/// ⊗ Tensor product: bottleneck (min) on P,F,K; union (max) elsewhere; special S.
pub fn tensor(a: &Tuple, b: &Tuple) -> Tuple {
    let mut r = [0u8; 12];
    for i in 0..12 {
        r[i] = match i {
            3 | 4 | 5 => a[i].min(b[i]),    // P, F, K: bottleneck
            10         => tensor_s(a[i], b[i]), // S: stoichiometry
            _          => a[i].max(b[i]),    // union
        };
    }
    r
}

/// ∨ Join: component-wise max (least upper bound).
pub fn join(a: &Tuple, b: &Tuple) -> Tuple {
    let mut r = [0u8; 12];
    for i in 0..12 { r[i] = a[i].max(b[i]); }
    r
}

/// ∧ Meet: component-wise min (greatest lower bound).
pub fn meet(a: &Tuple, b: &Tuple) -> Tuple {
    let mut r = [0u8; 12];
    for i in 0..12 { r[i] = a[i].min(b[i]); }
    r
}

/// Triadic mediation: witness ∨ (a ⊗ b).
/// Models Aleph as "the breath between" — contextualises without bottlenecking.
pub fn mediate(witness: &Tuple, a: &Tuple, b: &Tuple) -> Tuple {
    let comp = tensor(a, b);
    join(witness, &comp)
}

// ── Distance ─────────────────────────────────────────────────────────────────
//
// Weights × 10000 so that isqrt(sum) = dist × 100 (two decimal places).
// Original weights: [1.0,1.0,1.0,1.2,0.9,0.8,1.0,1.0,1.1,0.8,1.0,0.7]
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

/// Returns distance × 100 as u32 (e.g. 134 means d=1.34).
pub fn distance_scaled(a: &Tuple, b: &Tuple) -> u32 {
    let sum: u64 = WEIGHTS.iter().enumerate().map(|(i, &w)| {
        let d = if a[i] > b[i] { (a[i] - b[i]) as u64 } else { (b[i] - a[i]) as u64 };
        w * d * d
    }).sum();
    isqrt(sum) as u32
}

/// Returns distance as f64 (e.g. 1.34).
pub fn distance(a: &Tuple, b: &Tuple) -> f64 {
    distance_scaled(a, b) as f64 / 100.0
}

/// Conflict set: which primitives differ between two letters.
pub fn conflict_set(a: &Tuple, b: &Tuple) -> Vec<usize> {
    let mut set = Vec::new();
    for i in 0..12 {
        if a[i] != b[i] {
            set.push(i);
        }
    }
    set
}

/// Veracity class name from distance.
pub fn veracity_class(d: f64) -> &'static str {
    if d == 0.0 { "transparent" }
    else if d <= 1.4142 { "near-grounded" }       // sqrt(2)
    else if d <= 2.4495 { "partial-emergence" }   // sqrt(6)
    else { "aspirational" }
}

// ── System language ──────────────────────────────────────────────────────────

/// JOIN of all 22 canonical letters.
/// After §2.1 revision, achieves O_inf and ouroboric closure: L ⊗ L = L.
pub fn system_language() -> Tuple {
    let mut r = LETTERS[0].t;
    for l in &LETTERS[1..] {
        r = join(&r, &l.t);
    }
    r
}

/// Tier census: counts per tier across all 22 letters.
pub fn tier_census() -> [u8; 5] {
    // [O0, O1, O2, O2d, OInf]
    let mut counts = [0u8; 5];
    for l in &LETTERS {
        let idx = match compute_tier(&l.t) {
            Tier::O0   => 0,
            Tier::O1   => 1,
            Tier::O2   => 2,
            Tier::O2d  => 3,
            Tier::OInf => 4,
        };
        counts[idx] += 1;
    }
    counts
}

// ── Formatting helpers for REPL output ───────────────────────────────────────

/// Format a letter's tier, Phi, Omega, P for display.
pub fn format_letter(l: &LetterDef) -> String {
    let tier = compute_tier(&l.t);
    let phi_n = PHI_NAMES.get(l.t[8] as usize).copied().unwrap_or("?");
    let om_n  = OMEGA_NAMES.get(l.t[11] as usize).copied().unwrap_or("?");
    let p_n   = P_NAMES.get(l.t[3] as usize).copied().unwrap_or("?");
    format!("  -> {}\n    tier  {}\n    Phi  {}   Omega  {}   P  {}\n",
        display_glyph(l), tier_name(tier), phi_n, om_n, p_n)
}

/// Format full 12-primitive tuple with visual bars.
pub fn format_tuple(l: &LetterDef) -> String {
    let mut s = format!("  {} ({})  tier={}\n", display_glyph(l), l.name, compute_tier(&l.t));
    for i in 0..12 {
        let val = l.t[i] as usize;
        let max_val = match i {
            0 => 3, 1 => 4, 2 => 3, 3 => 4, 4 => 2, 5 => 3,
            6 => 2, 7 => 3, 8 => 4, 9 => 3, 10 => 2, 11 => 2,
            _ => 4,
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
            11 => OMEGA_NAMES.get(l.t[i] as usize).copied().unwrap_or("?"),
            _  => "?",
        };
        s += &format!("    {}={}  {}  {}\n", PRIM_NAMES[i], l.t[i], bar, val_name);
    }
    s
}

/// Format distance result.
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
    if d > 2.4495 {
        s += "  !! aspirational gap - insert vav-cast or promote tier\n";
    }
    s
}

/// Format detailed explanation of a letter (for :explain command).
pub fn format_explain(l: &LetterDef) -> String {
    let tier = compute_tier(&l.t);
    let phi_n = PHI_NAMES.get(l.t[8] as usize).copied().unwrap_or("?");
    let om_n = OMEGA_NAMES.get(l.t[11] as usize).copied().unwrap_or("?");
    let p_n = P_NAMES.get(l.t[3] as usize).copied().unwrap_or("?");
    let f_n = PRIM_NAMES[4];
    let k_n = PRIM_NAMES[5];
    let d = PRIM_NAMES[0];
    let t = PRIM_NAMES[1];
    let r = PRIM_NAMES[2];
    let g = PRIM_NAMES[6];
    let ga = PRIM_NAMES[7];
    let h = PRIM_NAMES[9];
    let s = PRIM_NAMES[10];

    let mut out = String::new();
    out += &format!("  {} ({})\n", display_glyph(l), l.name);
    out += &format!("  tier  {}\n", tier_name(tier));
    out += &format!("  {:<8} {:>2}  {}\n", d, l.t[0], "");
    out += &format!("  {:<8} {:>2}\n", t, l.t[1]);
    out += &format!("  {:<8} {:>2}\n", r, l.t[2]);
    out += &format!("  {:<8} {:>2}  {}\n", "P", l.t[3], p_n);
    out += &format!("  {:<8} {:>2}  {}\n", "F", l.t[4], f_n);
    out += &format!("  {:<8} {:>2}  {}\n", "K", l.t[5], k_n);
    out += &format!("  {:<8} {:>2}  {}\n", g, l.t[6], "");
    out += &format!("  {:<8} {:>2}  {}\n", ga, l.t[7], "");
    out += &format!("  {:<8} {:>2}  {}\n", "Phi", l.t[8], phi_n);
    out += &format!("  {:<8} {:>2}\n", h, l.t[9]);
    out += &format!("  {:<8} {:>2}  {}\n", s, l.t[10], "");
    out += &format!("  {:<8} {:>2}  {}\n", "Omega", l.t[11], om_n);
    out
}

/// Check if a character is a Hebrew letter.
pub fn is_hebrew(c: char) -> bool {
    let cp = c as u32;
    cp >= HEBREW_LO && cp <= HEBREW_HI
}

/// List all letter names (for autocomplete).
pub fn letter_names() -> &'static [&'static str] {
    &[
        "aleph", "bet", "gimel", "dalet", "hei", "vav",
        "zayin", "chet", "tet", "yod", "kaf", "lamed",
        "mem", "nun", "samech", "ayin", "pei", "tzadi",
        "kuf", "resh", "shin", "tav",
    ]
}

/// List all Hebrew glyphs (for autocomplete).
pub fn letter_glyphs() -> Vec<char> {
    LETTERS.iter().map(|l| l.glyph).collect()
}
