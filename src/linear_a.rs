//! Linear A (Minoan) engine — LATFF sign family → IMASM opcodes.
//!
//! The twelve Linear A visual-structural families (LATFF — Linear A Tablet Folio
//! Format, following GORILA classification) are the categorical primitives of the
//! Universal Imscriptive Grammar at Minoan token resolution.
//!
//! Crystal imscription (IG notation):
//!   ⟨ Ð_C  Þ_¨  Ř_Ť  Φ_}  ƒ^ż  Ç^W  Γ_ʔ  ɢ^ˌ  ⊙_ÿ  Ħ_A  Σ_ï  Ω_z ⟩
//!   Tier: O_∞  (⊙_ÿ + Φ_})
//!
//! Linear A = OS imscription exactly.
//! Adding Linear A as a sixth system to the exOS MEET leaves the invariant core
//! unchanged. The grammar was already complete. The Minoan system is not a
//! derivative of the five — it IS the structural core they all share.
//!
//! IG distances (exOS weighted metric):
//!   d(Linear A, OS imscription) = 0.00   — identical
//!   d(Linear A, Rohonc)         ≈ 2.10
//!   d(Linear A, Voynich)        ≈ 4.31

use crate::imasm_vm::{Instruction, Opcode};
extern crate alloc;
use alloc::vec::Vec;

// ── Crystal imscription ──────────────────────────────────────────────────────
// Index: [Ð, Þ, Ř, Φ, ƒ, Ç, Γ, ɢ, ⊙, Ħ, Σ, Ω]
// = OS_IMSCRIPTION exactly

pub const LINEAR_A_IMSCRIPTION: [u8; 12] = [1, 3, 2, 4, 2, 1, 2, 2, 1, 2, 2, 2];

// ── LATFF token table ────────────────────────────────────────────────────────
// Two-char codes for the twelve Linear A structural families (longest first).

pub const LATFF_TOKENS: &[(&str, Opcode)] = &[
    ("cu", Opcode::VINIT),  // cup/vessel forms         → Initial object ∅
    ("hk", Opcode::TANCH),  // hook/arm forms           → Terminal anchor ⊤
    ("fa", Opcode::AFWD),   // forward-arc forms        → Morphism →
    ("ba", Opcode::AREV),   // backward-arc forms       → Contravariant inversion ←
    ("lt", Opcode::CLINK),  // lattice/compound forms   → Composition ∘
    ("lp", Opcode::ISCRIB), // loop/knot forms          → Identity id
    ("br", Opcode::FSPLIT), // branching forms          → Frobenius co-multiplication δ
    ("cv", Opcode::FFUSE),  // convergent/triangular    → Frobenius multiplication μ
    ("vt", Opcode::EVALT),  // vertical-stroke forms    → Lattice: True
    ("hz", Opcode::EVALF),  // horizontal-stroke forms  → Lattice: False
    ("cl", Opcode::ENGAGR), // closed/circle forms      → Lattice: Both (paradox)
    ("dt", Opcode::IFIX),   // dot/fraction marks       → Linear tape write
];

// Bootstrap: id ∘ rev ∘ split ∘ fwd ∘ fuse ∘ link ∘ fix ∘ id
pub const BOOTSTRAP: &[Opcode] = &[
    Opcode::ISCRIB, Opcode::AREV, Opcode::FSPLIT, Opcode::AFWD,
    Opcode::FFUSE,  Opcode::CLINK, Opcode::IFIX,  Opcode::ISCRIB,
];

// ── Compiler ─────────────────────────────────────────────────────────────────

/// Compile LATFF-transcribed tablet text into IMASM instructions.
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

/// Compile a raw LATFF token string into IMASM instructions.
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
        for (tok, opcode) in LATFF_TOKENS {
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
