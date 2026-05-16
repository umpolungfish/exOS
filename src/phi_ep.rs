//! Phi_EP (Exceptional Point) Dynamics — Axiom P-596.
//!
//! Phi_EP represents a non-Hermitian degeneracy where the system
//! loses a dimensionality degree. As per Axiom P-596, this state
//! is "absorbing" relative to Phi_c:
//!
//!   Phi_c ⊗ Phi_EP -> Phi_EP
//!
//! This has the consequence of collapsing the self-modeling loop (C=0).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Criticality {
    /// Sub-critical
    Sub = 0,
    /// Phi_c - critical self-modeling gate
    C = 1,
    /// Complex-plane critical
    CComplex = 2,
    /// Phi_EP - exceptional point (absorbing)
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
