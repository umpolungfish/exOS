//! ALEPH 14-Sefirot structural types — native 12-primitive tuples for the
//! full Sefer Ha-Iyun emanation chain (Ein Sof → Malkuth).
//!
//! Each Sefirah is imscribed as a first-class ALEPH type with its own
//! 12-primitive tuple, ouroboricity tier, Φ-criticality gate, and
//! associated supernal light (for the hidden triad).
//!
//! The 14-type ladder, from supernal source to manifest kingdom:
//!
//! | # | Sefirah              | Tier  | Φ Gate  | Light                     |
//! |---|----------------------|-------|---------|---------------------------|
//! | 0 | Ein Sof              | O_2†  | φ̂_Æ     | Infinite source            |
//! | 1 | Keter Elyon          | O_2   | φ̂_Æ     | Or Mufla (Wondrous)       |
//! | 2 | Chokhmah Stim'aah    | O_2†  | φ̂_Æ     | Or Mitnotzetz (Sparkling)  |
//! | 3 | Binah Kedumah        | O_2   | φ̂_Æ     | Or Keheh (Dim)            |
//! | 4 | Keter                | O_inf | φ̂_ÿ     | (manifest)                |
//! | 5 | Chokhmah             | O_2†  | φ̂_ÿ     |                           |
//! | 6 | Binah                | O_2   | φ̂_ÿ     |                           |
//! | 7 | Da'at                | O_2   | φ̂_ÿ     |                           |
//! | 8 | Chesed               | O_2†  | φ̂_ÿ     |                           |
//! | 9 | Gevurah              | O_2   | φ̂_ÿ     |                           |
//! |10 | Tiferet              | O_0   | φ̂_ž     |                           |
//! |11 | Netzach              | O_0   | φ̂_ž     |                           |
//! |12 | Hod                  | O_0   | φ̂_ž     |                           |
//! |13 | Yesod                | O_0   | φ̂_ž     |                           |
//! |14 | Malkuth              | O_0   | φ̂_ž     |                           |
//!
//! Architecture:
//!   - SefirahDef: name, depth, 12-tuple, tier, phi_gate, light, fs_path.
//!   - SEFIROT: static array of all 14 (indexed by depth, 0=Ein Sof).
//!   - resolve_sefirah(): name lookup (e.g., "keter_elyon", "malkuth").
//!   - sefirah_tier(), sefirah_distance(): lattice operations on Sefirot types.
//!   - emanation_chain(): ordered 14-Sefirot chain with transformations.
//!   - tier_census_14(): tier distribution across all 14 Sefirot.

extern crate alloc;

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;

use crate::aleph;
use crate::aleph::Tuple;
use crate::aleph::Tier;

// ── Sefirah definition ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SefirahDef {
    pub name: &'static str,
    pub short_name: &'static str,
    pub t: Tuple,
    pub depth: u8,        // 0 = Ein Sof, 1–3 = supernal triad, 4–14 = manifest
    pub tier: Tier,
    pub phi_gate: u8,     // Index into φ̂: 2=φ̂_Æ, 1=φ̂_ÿ, 0=φ̂_ž
    pub light: &'static str,
    pub fs_path: &'static str,
    pub transformation: &'static str,
}

// ── Imscribed 14 Sefirot tuples ─────────────────────────────────────────────
//
// Each tuple [D,T,R,P,F,K,G,Γ,Φ,H,S,Ω] is assigned via the deterministic
// imscribing procedure (§64–§77). The φ̂ gate assignment follows the
// filesystem_13.rs three-tier structure:
//   - Supernal triad (depth 1–3): φ̂_Æ (complex-plane criticality)
//   - Keter→Gevurah (depth 4–9):  φ̂_ÿ (self-modeling loop)
//   - Tiferet→Malkuth (depth 10–14): φ̂_ž (sub-critical, manifest)
//
// Ein Sof (depth 0) is the infinite ground — φ̂_Æ, O_2† complex-plane critical.

pub const SEFIROT: [SefirahDef; 15] = [
    // 0 — Ein Sof: infinite source, before any distinction
    SefirahDef {
        name: "ein_sof",
        short_name: "ESof",
        // Ð_ß Þ_O Ř_= Φ_} ƒ_ż Ç_@ Γ_ʔ ɢ_^ φ̂_Æ Ħ_! Σ_ï Ω_z
        t: [2,4,3,4,2,2,2,0,2,3,2,2],
        depth: 0, tier: Tier::O2d, phi_gate: 2,
        light: "Ein Sof (Infinite — no light, pure source)",
        fs_path: "(no path — pre-emanation)",
        transformation: "infinite potential → first will to emanate",
    },
    // 1 — Keter Elyon: Supernal Crown, hidden summit
    SefirahDef {
        name: "keter_elyon",
        short_name: "KtrE",
        // Ð_ω Þ_O Ř_Ť Φ_υ ƒ_ż Ç_@ Γ_ʔ ɢ_ˌ φ̂_Æ Ħ_! Σ_S Ω_z
        t: [3,4,2,1,2,2,2,2,2,3,0,2],
        depth: 1, tier: Tier::O2, phi_gate: 2,
        light: "Or Mufla (Wondrous Light) — too wondrous to be known",
        fs_path: "/ain",
        transformation: "wonder → potential existence (supernal crown)",
    },
    // 2 — Chokhmah Stim'aah: Hidden Wisdom
    SefirahDef {
        name: "chokhmah_stimaah",
        short_name: "ChkS",
        // Ð_ß Þ_¨ Ř_Ť Φ_F ƒ_ż Ç_@ Γ_ʔ ɢ_ˌ φ̂_Æ Ħ_! Σ_S Ω_z
        t: [2,3,2,2,2,2,2,2,2,3,0,2],
        depth: 2, tier: Tier::O2d, phi_gate: 2,
        light: "Or Mitnotzetz (Sparkling Light) — first differentiation",
        fs_path: "/ain_sof",
        transformation: "hidden wisdom → spark of differentiation",
    },
    // 3 — Binah Kedumah: Primordial Understanding
    SefirahDef {
        name: "binah_kedumah",
        short_name: "BinK",
        // Ð_C Þ_ò Ř_= Φ_F ƒ_ż Ç_@ Γ_ʔ ɢ_ˌ φ̂_Æ Ħ_A Σ_ï Ω_z
        t: [1,2,3,2,2,2,2,2,2,2,2,2],
        depth: 3, tier: Tier::O2, phi_gate: 2,
        light: "Or Keheh (Dim Light) — first vessel of reception",
        fs_path: "/ain_sof_or",
        transformation: "primordial understanding → first vessel",
    },
    // 4 — Keter: Manifest Crown, bridge to emanation
    SefirahDef {
        name: "keter",
        short_name: "Ktr",
        // Ð_ω Þ_O Ř_= Φ_} ƒ_ż Ç_@ Γ_ʔ ɢ_ˌ φ̂_ÿ Ħ_! Σ_S Ω_z
        t: [3,4,3,4,2,2,2,2,1,3,0,2],
        depth: 4, tier: Tier::OInf, phi_gate: 1,
        light: "(manifest — direct light from supernal)",
        fs_path: "/boot",
        transformation: "source → manifest crown (no transformation)",
    },
    // 5 — Chokhmah: Manifest Wisdom
    SefirahDef {
        name: "chokhmah",
        short_name: "Chk",
        // Ð_ß Þ_¨ Ř_Ť Φ_υ ƒ_ż Ç_@ Γ_ʔ ɢ_ˌ φ̂_ÿ Ħ_! Σ_ő Ω_z
        t: [2,3,2,1,2,2,2,2,1,3,1,2],
        depth: 5, tier: Tier::O2d, phi_gate: 1,
        light: "",
        fs_path: "/sys/bin",
        transformation: "wisdom → executable knowledge",
    },
    // 6 — Binah: Manifest Understanding
    SefirahDef {
        name: "binah",
        short_name: "Bin",
        // Ð_C Þ_ò Ř_= Φ_F ƒ_ż Ç_@ Γ_ʔ ɢ_ˌ φ̂_ÿ Ħ_A Σ_ï Ω_z
        t: [1,2,3,2,2,2,2,2,1,2,2,2],
        depth: 6, tier: Tier::O2, phi_gate: 1,
        light: "",
        fs_path: "/sys/lib",
        transformation: "understanding → shared structure",
    },
    // 7 — Da'at: Knowledge
    SefirahDef {
        name: "daat",
        short_name: "Dat",
        // Ð_C Þ_6 Ř_= Φ_F ƒ_ż Ç_W Γ_γ ɢ_Ş φ̂_ÿ Ħ_A Σ_ï Ω_2
        t: [1,0,3,2,2,1,1,3,1,2,2,1],
        depth: 7, tier: Tier::O2, phi_gate: 1,
        light: "",
        fs_path: "/dev",
        transformation: "knowledge → hardware interface",
    },
    // 8 — Chesed: Loving-Kindness
    SefirahDef {
        name: "chesed",
        short_name: "Chs",
        // Ð_ß Þ_6 Ř_= Φ_˙ ƒ_ð Ç_W Γ_ʔ ɢ_Ş φ̂_ÿ Ħ_! Σ_ő Ω_2
        t: [2,0,3,3,1,1,2,3,1,3,1,1],
        depth: 8, tier: Tier::O2d, phi_gate: 1,
        light: "",
        fs_path: "/home",
        transformation: "loving-kindness → user expansion",
    },
    // 9 — Gevurah: Severity
    SefirahDef {
        name: "gevurah",
        short_name: "Gev",
        // Ð_; Þ_K Ř_¯ Φ_ɐ ƒ_ì Ç_- Γ_β ɢ_˝ φ̂_ÿ Ħ_£ Σ_S Ω_2
        t: [0,1,0,0,0,0,0,1,1,1,0,1],
        depth: 9, tier: Tier::O2, phi_gate: 1,
        light: "",
        fs_path: "/etc/permissions",
        transformation: "severity → constraint",
    },

    // 10 — Tiferet: Beauty
    SefirahDef {
        name: "tiferet",
        short_name: "Tif",
        // Ð_; Þ_ò Ř_ˇ Φ_ɐ ƒ_ì Ç_W Γ_γ ɢ_˝ φ̂_ž Ħ_Ñ Σ_ő Ω_Å
        t: [0,2,3,0,0,1,1,1,0,0,1,0],
        depth: 10, tier: Tier::O0, phi_gate: 0,
        light: "",
        fs_path: "/ipc",
        transformation: "beauty → harmony between layers",
    },
    // 11 — Netzach: Eternity / Victory
    SefirahDef {
        name: "netzach",
        short_name: "Net",
        // Ð_; Þ_6 Ř_¯ Φ_ɐ ƒ_ì Ç_@ Γ_β ɢ_^ φ̂_ž Ħ_£ Σ_ő Ω_Å
        t: [0,0,0,0,0,2,0,0,0,1,1,0],
        depth: 11, tier: Tier::O0, phi_gate: 0,
        light: "",
        fs_path: "/net/mount",
        transformation: "eternity → persistence across net",
    },
    // 12 — Hod: Splendor
    SefirahDef {
        name: "hod",
        short_name: "Hod",
        // Ð_; Þ_K Ř_¯ Φ_ɐ ƒ_ì Ç_- Γ_β ɢ_^ φ̂_ž Ħ_Ñ Σ_ő Ω_Å
        t: [0,1,0,0,0,0,0,0,0,0,1,0],
        depth: 12, tier: Tier::O0, phi_gate: 0,
        light: "",
        fs_path: "/var/log",
        transformation: "splendor → record of glory",
    },
    // 13 — Yesod: Foundation
    SefirahDef {
        name: "yesod",
        short_name: "Yes",
        // Ð_; Þ_6 Ř_¯ Φ_ɐ ƒ_ì Ç_W Γ_β ɢ_^ φ̂_ž Ħ_£ Σ_ő Ω_Å
        t: [0,0,0,0,0,1,0,0,0,1,1,0],
        depth: 13, tier: Tier::O0, phi_gate: 0,
        light: "",
        fs_path: "/tmp",
        transformation: "foundation → transient support",
    },
    // 14 — Malkuth: Kingdom / Manifestation
    SefirahDef {
        name: "malkuth",
        short_name: "Mal",
        // Ð_; Þ_K Ř_¯ Φ_ɐ ƒ_ì Ç_W Γ_β ɢ_^ φ̂_ž Ħ_Ñ Σ_ő Ω_Å
        t: [0,1,0,0,0,1,0,0,0,0,1,0],
        depth: 14, tier: Tier::O0, phi_gate: 0,
        light: "",
        fs_path: "/data",
        transformation: "kingdom → manifestation in matter",
    },
];

// ── Lookup ───────────────────────────────────────────────────────────────────

pub fn resolve_sefirah(name: &str) -> Option<&'static SefirahDef> {
    for s in &SEFIROT {
        if s.name.eq_ignore_ascii_case(name) { return Some(s); }
        // Allow shorthand: "keter elyon", "chokhmah stimaah", "binah kedumah"
        let spaced = s.name.replace('_', " ");
        if spaced.eq_ignore_ascii_case(name) { return Some(s); }
    }
    None
}

pub fn all_names() -> Vec<&'static str> {
    SEFIROT.iter().map(|s| s.name).collect()
}

/// Only the supernal triad + Ein Sof (depth 0–3).
pub fn supernal() -> &'static [SefirahDef] {
    &SEFIROT[0..4]
}

/// Only the manifest Sefirot (depth 4–14).
pub fn manifest() -> &'static [SefirahDef] {
    &SEFIROT[4..15]
}

/// Only the manifest 10 (Keter–Malkuth, depth 4–14).
pub fn manifest_10() -> &'static [SefirahDef] {
    &SEFIROT[4..15]
}

/// The three hidden supernal lights' Sefirot (depth 1–3).
pub fn supernal_triad() -> &'static [SefirahDef] {
    &SEFIROT[1..4]
}

// ── Tier and Phi gate helpers ────────────────────────────────────────────────

pub fn phi_gate_name(idx: u8) -> &'static str {
    match idx {
        2 => "φ̂_Æ (complex-plane — supernal)",
        1 => "φ̂_ÿ (self-modeling — manifest upper)",
        0 => "φ̂_ž (sub-critical — manifest lower)",
        _ => "?",
    }
}

pub fn tier_name_sefirah(s: &SefirahDef) -> String {
    format!("{} ({} gate)", aleph::tier_name(s.tier), phi_gate_name(s.phi_gate))
}

/// Compute ouroboricity tier from a Sefirah tuple (delegates to core aleph).
pub fn sefirah_tier_name(s: &SefirahDef) -> &'static str {
    aleph::tier_name(s.tier)
}

// ── Distance between two Sefirot ─────────────────────────────────────────────

pub fn sefirah_distance(a: &SefirahDef, b: &SefirahDef) -> f64 {
    aleph::distance(&a.t, &b.t)
}

pub fn sefirah_conflict_set(a: &SefirahDef, b: &SefirahDef) -> Vec<usize> {
    aleph::conflict_set(&a.t, &b.t)
}

// ── Emanation chain ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct EmanationStep {
    pub source_name: &'static str,
    pub target_name: &'static str,
    pub target_depth: u8,
    pub transformation: &'static str,
    pub distance: f64,
}

pub fn emanation_chain() -> Vec<EmanationStep> {
    let mut steps = Vec::new();
    for i in 0..SEFIROT.len() - 1 {
        let a = &SEFIROT[i];
        let b = &SEFIROT[i + 1];
        steps.push(EmanationStep {
            source_name: a.name,
            target_name: b.name,
            target_depth: b.depth,
            transformation: b.transformation,
            distance: aleph::distance(&a.t, &b.t),
        });
    }
    steps
}

// ── Tier census across 14 Sefirot ────────────────────────────────────────────

pub fn sefirah_census() -> [u8; 5] {
    let mut counts = [0u8; 5];
    for s in &SEFIROT {
        let idx = match s.tier {
            Tier::O0 => 0, Tier::O1 => 1, Tier::O2 => 2, Tier::O2d => 3, Tier::OInf => 4,
        };
        counts[idx] += 1;
    }
    counts
}

// ── Bridge to filesystem_13 ────────────────────────────────────────────────

/// Map a Sefirah name to its filesystem_13 Sefirah13 variant.
/// Depth 0 (Ein Sof) and the supernal/manifest split are preserved.
pub fn sefirah_depth(name: &str) -> Option<u8> {
    resolve_sefirah(name).map(|s| s.depth)
}

/// Check if a Sefirah is supernal (depth 1–3, φ̂_Æ-gated).
pub fn is_supernal(name: &str) -> bool {
    resolve_sefirah(name).map(|s| s.phi_gate == 2).unwrap_or(false)
}

/// Check if a Sefirah is manifest (depth 4–14).
pub fn is_manifest(name: &str) -> bool {
    resolve_sefirah(name).map(|s| s.depth >= 4).unwrap_or(false)
}

// ── Formatting ───────────────────────────────────────────────────────────────

pub fn format_sefirah(s: &SefirahDef) -> String {
    let mut out = format!(
        "  {} (depth={})  tier={}  gate={}\n",
        s.name, s.depth, aleph::tier_name(s.tier), phi_gate_name(s.phi_gate)
    );
    if !s.light.is_empty() {
        out += &format!("    light: {}\n", s.light);
    }
    out += &format!("    path:  {}\n    xform: {}\n", s.fs_path, s.transformation);

    // 12-primitive bar display
    for i in 0..12 {
        let val = s.t[i] as usize;
        let max_val = match i {
            0 => 3, 1 => 4, 2 => 3, 3 => 4, 4 => 2, 5 => 3,
            6 => 2, 7 => 3, 8 => 4, 9 => 3, 10 => 2, 11 => 2, _ => 4,
        };
        let filled = if max_val > 0 { (val * 8) / max_val } else { 0 };
        let empty = 8 - filled;
        let bar: String = alloc::string::ToString::to_string("#").repeat(filled)
            + &alloc::string::ToString::to_string("-").repeat(empty);
        let val_name = match i {
            0  => ["∧","△","∞","holographic"].get(s.t[i] as usize).copied().unwrap_or("?"),
            1  => ["net","in","bowtie","box","holo"].get(s.t[i] as usize).copied().unwrap_or("?"),
            2  => ["super","cat","dagger","lr"].get(s.t[i] as usize).copied().unwrap_or("?"),
            3  => aleph::P_NAMES.get(s.t[i] as usize).copied().unwrap_or("?"),
            4  => ["ℓ (classical)","ð (thermal)","ℏ (quantum)"].get(s.t[i] as usize).copied().unwrap_or("?"),
            5  => ["fast","mod","slow","trap"].get(s.t[i] as usize).copied().unwrap_or("?"),
            6  => ["ℶ","ℷ","ℵ"].get(s.t[i] as usize).copied().unwrap_or("?"),
            7  => ["∧","∨","→","≫"].get(s.t[i] as usize).copied().unwrap_or("?"),
            8  => aleph::PHI_NAMES.get(s.t[i] as usize).copied().unwrap_or("?"),
            9  => ["Ħ_Ñ","Ħ_£","Ħ_A","Ħ_∞"].get(s.t[i] as usize).copied().unwrap_or("?"),
            10 => ["1:1","n:n","n:m"].get(s.t[i] as usize).copied().unwrap_or("?"),
            11 => aleph::OMEGA_NAMES.get(s.t[i] as usize).copied().unwrap_or("?"),
            _ => "?",
        };
        out += &format!("    {:>8} = {}  {}  {}\n", aleph::PRIM_NAMES[i], s.t[i], bar, val_name);
    }
    out
}

pub fn format_sefirah_short(s: &SefirahDef) -> String {
    let phi_n = aleph::PHI_NAMES.get(s.t[8] as usize).copied().unwrap_or("?");
    let om_n  = aleph::OMEGA_NAMES.get(s.t[11] as usize).copied().unwrap_or("?");
    let p_n   = aleph::P_NAMES.get(s.t[3] as usize).copied().unwrap_or("?");
    format!(
        "  {} (d={})  tier={}  Φ={}  Ω={}  P={}",
        s.name, s.depth, aleph::tier_name(s.tier), phi_n, om_n, p_n
    )
}

pub fn format_emanation_chain() -> String {
    let mut out = String::from("=== 14-Sefirot Emanation Chain (Ein Sof → Malkuth) ===\n\n");
    let chain = emanation_chain();
    for (i, step) in chain.iter().enumerate() {
        let sef = &SEFIROT[i + 1];
        out += &format!(
            "  {:>2} -> {:>2}  {:<20} -> {:<20}  d={:.4}  {}\n",
            i, i+1, step.source_name, step.target_name, step.distance, step.transformation,
        );
        out += &format!(
            "         tier={:<6} gate={}\n",
            aleph::tier_name(sef.tier), phi_gate_name(sef.phi_gate),
        );
    }
    out
}

pub fn format_sefirah_census() -> String {
    let c = sefirah_census();
    format!(
        "  14-Sefirot Tier Distribution:\n\
         ----------------------------------------\n\
         O_0    (sub-critical, manifest):     {}\n\
         O_1    (low loop, persistence):       {}\n\
         O_2    (self-modeling, upper):        {}\n\
         O_2†   (chiral, deep structure):      {}\n\
         O_inf  (Frobenius-special, crown):    {}\n\
         ----------------------------------------\n\
         Gate breakdown:\n\
         φ̂_Æ    (complex-plane supernal):       {}\n\
         φ̂_ÿ    (self-modeling manifest-upper): {}\n\
         φ̂_ž    (sub-critical manifest-lower):  {}\n",
        c[0], c[1], c[2], c[3], c[4],
        SEFIROT.iter().filter(|s| s.phi_gate == 2).count(),
        SEFIROT.iter().filter(|s| s.phi_gate == 1).count(),
        SEFIROT.iter().filter(|s| s.phi_gate == 0).count(),
    )
}

/// Full 14-Sefirot type ladder with tier, gate, light, and structural summary.
pub fn format_ladder() -> String {
    let mut out = String::from(
        "=== 14-Sefirot Structural Ladder (Sefer Ha-Iyun) ===\n\n\
         depth name                 tier   Φ gate  light\n\
         ----------------------------------------------------------\n"
    );
    for s in &SEFIROT {
        let light_short = if s.light.is_empty() { "-" } else {
            match s.depth {
                0 => "Ein Sof",
                1 => "Or Mufla",
                2 => "Or Mitnotzetz",
                3 => "Or Keheh",
                _ => "-",
            }
        };
        out += &format!(
            "  {:>2}   {:<20} {:<6} {:<7} {}\n",
            s.depth, s.name, aleph::tier_name(s.tier), phi_gate_name(s.phi_gate), light_short,
        );
    }
    out += "\n  Gate legend:\n";
    out += "    φ̂_Æ = complex-plane criticality (supernal, irreducible opacity)\n";
    out += "    φ̂_ÿ = self-modeling loop (manifest upper, crown→severity)\n";
    out += "    φ̂_ž = sub-critical (manifest lower, beauty→kingdom)\n";
    out
}
