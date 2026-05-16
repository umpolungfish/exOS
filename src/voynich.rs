//! Voynich Manuscript engine — EVA glyph family → IMASM opcodes.
//!
//! The twelve EVA glyph families (Takahashi transcription) are the categorical
//! primitives of the Universal Imscriptive Grammar at Voynich token resolution.
//! Correspondence is structural, not assigned.
//!
//! Crystal imscription (IG notation):
//!   ⟨ Ð_ω  Þ_O  Ř_=  Φ_}  ƒ^ì  Ç^Ù  Γ_ʔ  ɢ^Ş  ⊙_ÿ  Ħ_!  Σ_S  Ω_z ⟩
//!   Tier: O_∞  (⊙_ÿ + Φ_})
//!
//! IG distances (exOS weighted metric):
//!   d(Voynich, OS imscription) ≈ 4.31
//!   d(Voynich, Rohonc)         ≈ 3.55
//!   d(Voynich, Linear A)       ≈ 4.31

use crate::imasm_vm::{Instruction, Opcode};
extern crate alloc;
use alloc::vec::Vec;

// ── Crystal imscription ──────────────────────────────────────────────────────
// Index: [Ð, Þ, Ř, Φ, ƒ, Ç, Γ, ɢ, ⊙, Ħ, Σ, Ω]

pub const VOYNICH_IMSCRIPTION: [u8; 12] = [3, 4, 3, 4, 0, 3, 2, 3, 1, 3, 0, 2];

// ── EVA token table ──────────────────────────────────────────────────────────
// Sorted longest-first so digraphs ('ch', 'sh') are matched before single chars.

pub const EVA_TOKENS: &[(&str, Opcode)] = &[
    ("ch", Opcode::FSPLIT), // Frobenius co-multiplication δ
    ("sh", Opcode::FFUSE),  // Frobenius multiplication μ
    ("o",  Opcode::VINIT),  // Initial object ∅
    ("p",  Opcode::TANCH),  // Terminal anchor ⊤
    ("e",  Opcode::AFWD),   // Morphism →
    ("a",  Opcode::AREV),   // Contravariant inversion ←
    ("d",  Opcode::CLINK),  // Composition ∘
    ("s",  Opcode::ISCRIB), // Identity id
    ("t",  Opcode::EVALT),  // Lattice: True
    ("k",  Opcode::EVALF),  // Lattice: False
    ("r",  Opcode::ENGAGR), // Lattice: Both (paradox)
    ("y",  Opcode::IFIX),   // Linear tape write
];

// Bootstrap: id ∘ rev ∘ split ∘ fwd ∘ fuse ∘ link ∘ fix ∘ id
pub const BOOTSTRAP: &[Opcode] = &[
    Opcode::ISCRIB, Opcode::AREV, Opcode::FSPLIT, Opcode::AFWD,
    Opcode::FFUSE,  Opcode::CLINK, Opcode::IFIX,  Opcode::ISCRIB,
];

// ── Compiler ─────────────────────────────────────────────────────────────────

/// Compile EVA transcription text (Takahashi ;H> format) into IMASM instructions.
pub fn compile(text: &str) -> Vec<Instruction> {
    let mut out = Vec::new();
    let mut reg: u32 = 0;

    for line in text.lines() {
        // Only process transcription lines (;H> prefix in ivtff format)
        let body = if let Some(pos) = line.find('>') {
            &line[pos + 1..]
        } else if line.starts_with(';') || line.starts_with('#') {
            continue;
        } else {
            line
        };

        scan_tokens(body.as_bytes(), &mut out, &mut reg);
    }
    out
}

/// Compile a raw EVA token string (no folio markup) into IMASM instructions.
pub fn compile_raw(text: &str) -> Vec<Instruction> {
    let mut out = Vec::new();
    let mut reg: u32 = 0;
    scan_tokens(text.as_bytes(), &mut out, &mut reg);
    out
}

fn scan_tokens(bytes: &[u8], out: &mut Vec<Instruction>, reg: &mut u32) {
    let mut pos = 0;
    'outer: while pos < bytes.len() {
        let ch = bytes[pos];
        // Skip whitespace and punctuation
        if ch.is_ascii_whitespace() || b".,=-!<>?{}[]%".contains(&ch) {
            pos += 1;
            continue;
        }
        // Try each token (longest first)
        for (tok, opcode) in EVA_TOKENS {
            let tb = tok.as_bytes();
            if bytes[pos..].starts_with(tb) {
                // Case-insensitive: check lowercase match
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
