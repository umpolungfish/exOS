//! Phase 1 Expansion: ⊙_EP Dynamics, Stoichiometry, and IPC Grammars.
//!
//! 1. ⊙_EP: ordinal 3. Absorption rule: tensor(⊙_c, ⊙_EP) -> ⊙_EP.
//! 2. Stoichiometry (Σ): ordinal 10. 1:1 (0), n:n (1), n:m (2).
//! 3. Interaction Grammar (ɢ): ordinal 7. seq (2), broad (3).

pub const S_1_ONE: u8 = 0;
pub const S_N_N: u8 = 1;
pub const S_N_M: u8 = 2;

pub const GAMMA_SEQ: u8 = 2;
pub const GAMMA_BROAD: u8 = 3;

pub const PHI_EP: u8 = 3;

/// P-596: Coupling Destruction Rule
/// ⊙_c ⊗ ⊙_EP → ⊙_EP
pub fn tensor_phi(a: u8, b: u8) -> u8 {
    let max = if a > b { a } else { b };
    if (a == 1 && b == 3) || (a == 3 && b == 1) {
        3 // ⊙_EP absorbs ⊙_c
    } else {
        max
    }
}
