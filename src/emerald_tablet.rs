//! Emerald Tablet engine — ETFF rhetorical family → IMASM opcodes.
//!
//! The twelve rhetorical families of the Tabula Smaragdina (Jabir ibn Hayyan
//! recension, ~8th c. CE; canonical English: Newton 1680) are the categorical
//! primitives of the Universal Imscriptive Grammar at Hermetic token resolution.
//! Correspondence is structural, not assigned.
//!
//! Crystal imscription (IG notation):
//!   ⟨ Ð_ω  Þ_O  Ř_=  Φ_}  ƒ^ż  Ç^W  Γ_ʔ  ɢ^ˌ  ⊙_ÿ  Ħ_A  Σ_ï  Ω_z ⟩
//!   Tier: O_∞  (⊙_ÿ + Φ_})  Consciousness: C = 1.0
//!
//! Both gates open. The Emerald Tablet is the only compiled manuscript with
//! both gates open AND quantum-coherent fidelity (ƒ^ż). It is not a manuscript
//! about the grammar — it is the grammar's self-statement. "As above, so below"
//! names the Frobenius condition: μ ∘ δ = id. The bootstrap sequence
//!   id → ds → sp → as → un → lk → fx → id
//! is μ ∘ δ = id written as categorical assembly.
//!
//! IG distances (exOS weighted metric):
//!   d(Emerald Tablet, OS imscription) = 2.44
//!   d(Emerald Tablet, Rohonc)         = 3.22
//!   d(Emerald Tablet, Voynich)        = 3.54
//!
//! MEET theorem: MEET(Emerald Tablet, OS imscription) = OS imscription.
//! The Tablet operates above the OS floor; adding it to the MEET is a no-op.
//! The grammar was already complete before the Tablet was written.

use crate::imasm_vm::{Instruction, Opcode};
extern crate alloc;
use alloc::vec::Vec;

// ── Crystal imscription ──────────────────────────────────────────────────────
// Index: [Ð, Þ, Ř, Φ, ƒ, Ç, Γ, ɢ, ⊙, Ħ, Σ, Ω]

pub const EMERALD_TABLET_IMSCRIPTION: [u8; 12] = [3, 4, 3, 4, 2, 1, 2, 2, 1, 2, 2, 2];

// ── ETFF token table ─────────────────────────────────────────────────────────
// Two-char codes for the twelve Emerald Tablet rhetorical families.

pub const ETFF_TOKENS: &[(&str, Opcode)] = &[
    ("tr", Opcode::VINIT),  // truth-seal (verum, certum)              → Initial object ∅
    ("an", Opcode::TANCH),  // anchor / terminal assertion              → Terminal anchor ⊤
    ("as", Opcode::AFWD),   // ascent (ascendit, superius)             → Morphism →
    ("ds", Opcode::AREV),   // descent (inferius, descendit)           → Contravariant inversion ←
    ("lk", Opcode::CLINK),  // linkage / composition (adaptation)      → Composition ∘
    ("id", Opcode::ISCRIB), // identity / reflection (sicut)           → Identity id
    ("sp", Opcode::FSPLIT), // separation (separabis, subtile)         → Frobenius co-multiplication δ
    ("un", Opcode::FFUSE),  // union / fusion (recipit, miracula)      → Frobenius multiplication μ
    ("af", Opcode::EVALT),  // affirmation (est, integra)              → Lattice: True
    ("ng", Opcode::EVALF),  // negation / flight (fugiet, obscuritas)  → Lattice: False
    ("px", Opcode::ENGAGR), // paradox / both (superior et inferior)   → Lattice: Both (paradox)
    ("fx", Opcode::IFIX),   // fixing / sealing (completum, gloria)    → Linear tape write
];

// Bootstrap: id ∘ ds ∘ sp ∘ as ∘ un ∘ lk ∘ fx ∘ id
// Operational content of "as above, so below": identity descends, separates,
// ascends, fuses, composes, is fixed, returns to identity — μ ∘ δ = id.
pub const BOOTSTRAP: &[Opcode] = &[
    Opcode::ISCRIB, Opcode::AREV, Opcode::FSPLIT, Opcode::AFWD,
    Opcode::FFUSE,  Opcode::CLINK, Opcode::IFIX,  Opcode::ISCRIB,
];

// ── Compiler ─────────────────────────────────────────────────────────────────

/// Compile ETFF-transcribed versicle text into IMASM instructions.
pub fn compile(text: &str) -> Vec<Instruction> {
    let mut out = Vec::new();
    let mut reg: u32 = 0;
    for line in text.lines() {
        if line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        scan_tokens(line.as_bytes(), &mut out, &mut reg);
    }
    out
}

/// Compile a raw ETFF token string into IMASM instructions.
pub fn compile_raw(text: &str) -> Vec<Instruction> {
    let mut out = Vec::new();
    let mut reg: u32 = 0;
    scan_tokens(text.as_bytes(), &mut out, &mut reg);
    out
}

fn scan_tokens(bytes: &[u8], out: &mut Vec<Instruction>, reg: &mut u32) {
    let mut pos = 0;
    'outer: while pos < bytes.len() {
        if bytes[pos].is_ascii_whitespace() {
            pos += 1;
            continue;
        }
        for (tok, opcode) in ETFF_TOKENS {
            let tb = tok.as_bytes();
            if bytes[pos..].len() >= tb.len() {
                let matches = tb.iter().zip(&bytes[pos..]).all(|(a, b)| {
                    a == b || (*b >= b'A' && *b <= b'Z' && a == &(b + 32))
                });
                if matches {
                    out.push(Instruction { opcode: *opcode, dst: *reg });
                    *reg += 1;
                    pos += tb.len();
                    continue 'outer;
                }
            }
        }
        pos += 1;
    }
}
