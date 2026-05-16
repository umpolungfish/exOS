//! ⊙_EP (Exceptional Point) Dynamics — Axiom P-596.
//!
//! ⊙_EP represents a non-Hermitian degeneracy where the system
//! loses a dimensionality degree. As per Axiom P-596, this state
//! is "absorbing" relative to ⊙_c:
//!
//!   ⊙_c ⊗ ⊙_EP -> ⊙_EP
//!
//! This has the consequence of collapsing the self-modeling loop (C=0).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Criticality {
    /// Sub-critical (⊙_sub)
    Sub = 0,
    /// ⊙_c — critical self-modeling gate
    C = 1,
    /// Complex-plane critical
    CComplex = 2,
    /// ⊙_EP — exceptional point (absorbing)
    EP = 3,
    /// Super-critical (unstable)
    Super = 4,
}

impl Criticality {
    pub fn is_ep(&self) -> bool {
        matches!(self, Self::EP)
    }

    /// Check if the consciousness gate is destroyed by EP absorption.
    pub fn absorbs_consciousness(a: Self, b: Self) -> bool {
        (a == Self::C && b == Self::EP) || (a == Self::EP && b == Self::C)
    }
}
