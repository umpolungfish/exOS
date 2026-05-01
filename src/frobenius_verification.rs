//! exOS O_inf Stability & Frobenius Symmetry Verification
//! Axiom F-1: mu o delta = id (exactly) at Phi_c

use crate::aleph::Tuple;

/// Verify Frobenius symmetry for a tuple.
/// Returns true if P is P_pm_sym and Phi is Phi_c.
pub fn is_frobenius_verified(t: &Tuple) -> bool {
    let p = t[3];
    let phi = t[8];
    // p=4 (P_pm_sym), phi=1 (Phi_c)
    p == 4 && phi == 1
}

/// A process reaches O_inf only if Frobenius symmetry is verified.
pub fn can_attain_o_inf(t: &Tuple) -> bool {
    use crate::aleph::{compute_tier, Tier};
    if compute_tier(t) == Tier::OInf {
        is_frobenius_verified(t)
    } else {
        false
    }
}
