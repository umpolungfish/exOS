//! Rohonc Codex engine — RTFF glyph family → IMASM opcodes.
//!
//! The twelve Rohonc paleographic families (RTFF — Rohonc Transcription Folio
//! Format) are the categorical primitives of the Universal Imscriptive Grammar
//! at Rohonc token resolution. Correspondence is structural, not assigned.
//!
//! Crystal imscription (IG notation):
//!   ⟨ Ð_C  Þ_¨  Ř_Ť  Φ_}  ƒ^ì  Ç^@  Γ_ʔ  ɢ^ˌ  ⊙_ÿ  Ħ_A  Σ_ï  Ω_z ⟩
//!   Tier: O_∞  (⊙_ÿ + Φ_})
//!
//! Differs from OS imscription in two primitives:
//!   ƒ^ì (classical fidelity) vs ƒ^ż — no quantum coherence in the symbol surface
//!   Ç^@ (equilibrium)        vs Ç^W — frozen kinetics
//!
//! IG distances (exOS weighted metric):
//!   d(Rohonc, OS imscription) ≈ 2.10
//!   d(Rohonc, Voynich)        ≈ 3.55
//!   d(Rohonc, Linear A)       ≈ 2.10

use crate::imasm_vm::{Instruction, Opcode};
extern crate alloc;
use alloc::vec::Vec;

// ── Crystal imscription ──────────────────────────────────────────────────────
// Index: [Ð, Þ, Ř, Φ, ƒ, Ç, Γ, ɢ, ⊙, Ħ, Σ, Ω]

pub const ROHONC_IMSCRIPTION: [u8; 12] = [1, 3, 2, 4, 0, 2, 2, 2, 1, 2, 2, 2];

// ── RTFF token table ─────────────────────────────────────────────────────────
// Two-char codes for the twelve Rohonc paleographic families (longest first).

pub const RTFF_TOKENS: &[(&str, Opcode)] = &[
    ("cr", Opcode::VINIT),  // cross/crucifix family    → Initial object ∅
    ("hk", Opcode::TANCH),  // hook/fishhook family     → Terminal anchor ⊤
    ("fa", Opcode::AFWD),   // forward-arc family       → Morphism →
    ("ba", Opcode::AREV),   // backward-arc family      → Contravariant inversion ←
    ("lg", Opcode::CLINK),  // ligature/compound family → Composition ∘
    ("lp", Opcode::ISCRIB), // loop/mirror family       → Identity id
    ("br", Opcode::FSPLIT), // branch/fork family       → Frobenius co-multiplication δ
    ("cv", Opcode::FFUSE),  // convergent family        → Frobenius multiplication μ
    ("vt", Opcode::EVALT),  // vertical-stroke family   → Lattice: True
    ("hz", Opcode::EVALF),  // horizontal-stroke family → Lattice: False
    ("cl", Opcode::ENGAGR), // closed-loop family       → Lattice: Both (paradox)
    ("dt", Opcode::IFIX),   // dot/point family         → Linear tape write
];

// Bootstrap: id ∘ rev ∘ split ∘ fwd ∘ fuse ∘ link ∘ fix ∘ id
pub const BOOTSTRAP: &[Opcode] = &[
    Opcode::ISCRIB, Opcode::AREV, Opcode::FSPLIT, Opcode::AFWD,
    Opcode::FFUSE,  Opcode::CLINK, Opcode::IFIX,  Opcode::ISCRIB,
];

// ── Compiler ─────────────────────────────────────────────────────────────────

/// Compile RTFF-transcribed text into IMASM instructions.
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

/// Compile a raw RTFF token string into IMASM instructions.
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
        for (tok, opcode) in RTFF_TOKENS {
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
