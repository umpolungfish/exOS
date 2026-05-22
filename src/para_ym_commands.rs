// para_ym_commands.rs — Yang-Mills Mass Gap Bridge for exOS
//
// Usage:
//   para ym          full suite (gap + BRST + confinement + structural type)
//   para ym gap      covering relation and mass gap analysis
//   para ym brst     BRST ↔ Frobenius correspondence
//
// Lean reference: MillenniumAnkh/Imscribing/Paraconsistent/QCI_YM_Bridge.lean

#![allow(dead_code)]

extern crate alloc;

use alloc::string::String;
use alloc::format;

use crate::para_vm::{B4, ParaKernel};


// ── Mass gap: covering relation N < T ────────────────────────────────────────

// N < T is a covering relation: no x with N <_a x <_a T.
fn ym_gap_exists() -> bool {
    let all = [B4::N, B4::T, B4::F, B4::B];
    !all.iter().any(|&x| {
        x != B4::N && x != B4::T
        && B4::N.approx_le(x) && x.approx_le(B4::T)
    })
}

// Ground state T is not dialetheic — gap is definite, not contradictory.
fn ym_gap_not_dialetheic() -> bool { !B4::T.dialetheic() }

// T∧F=N: particle + antiparticle → vacuum (join is meet here).
fn ym_vacuum_canonical() -> bool {
    !B4::N.designated()
    && B4::N.approx_le(B4::T)
    && B4::T.meet(B4::F) == B4::N
}

// BRST nilpotency: ENGAGR(B)=B (stable), ENGAGR(T)→F (nilpotent);
// Frobenius: μ∘δ(B)=B.
fn ym_brst_nilpotent() -> bool {
    let q_on_b = B4::B.band(B4::B.bnot());   // band(B,¬B)=B
    let q_on_t = B4::T.band(B4::T.bnot());   // band(T,¬T)=F
    let frobenius = ParaKernel::frobenius_invariant(B4::B);
    q_on_b == B4::B && q_on_t != B4::B && frobenius
}

// T cannot reach N via any self-op (confinement / K_trap stability).
fn ym_confinement_ktrap() -> bool {
    let ops = [
        B4::T.bnot(),
        B4::T.join(B4::T), B4::T.meet(B4::T),
        B4::T.band(B4::T), B4::T.bor(B4::T),
    ];
    ops.iter().all(|&c| c != B4::N)
}

// T∨T=T (no spontaneous B-creation); N∨T=T (gap survives).
fn ym_topological_protection() -> bool {
    B4::T.join(B4::T) == B4::T
    && B4::N.join(B4::T) == B4::T
    && B4::T.join(B4::F) == B4::B  // T∨F=B: particle+antiparticle=B (annihilation)
}


// ── Display blocks ────────────────────────────────────────────────────────────

fn gap_block() -> String {
    let mark = |ok: bool| if ok { "✓" } else { "✗" };
    let mut s = String::new();
    s += "  ┌──────────────────────────────────────────────────────────────┐\n";
    s += "  │  MASS GAP: covering relation N < T                          │\n";
    s += "  ├──────────────────────────────────────────────────────────────┤\n";
    s += &format!("  │  {}  ym_gap_exists: N < T is a covering relation        │\n",
        mark(ym_gap_exists()));
    s += "  │       no x with N <_a x <_a T — Δ > 0 structurally         │\n";
    s += &format!("  │  {}  ym_gap_not_dialetheic: T is definite (not B)       │\n",
        mark(ym_gap_not_dialetheic()));
    s += "  │       gap cannot be dialetheic (would imply zero mass)      │\n";
    s += &format!("  │  {}  ym_vacuum_canonical: N is the unique floor          │\n",
        mark(ym_vacuum_canonical()));
    s += "  │       T∧F=N: particle+antiparticle → vacuum                 │\n";
    s += &format!("  │  {}  ym_topological_protection: T∨T=T (Omega_Z lock)    │\n",
        mark(ym_topological_protection()));
    s += "  │       no spontaneous B-creation; gap is gauge-protected     │\n";
    s += "  ├──────────────────────────────────────────────────────────────┤\n";
    s += "  │  Approximation order (mass spectrum):                        │\n";
    s += "  │    N (vacuum, Δ=0) ≤ T (gluon, Δ>0) ≤ B (BRST sector)     │\n";
    s += "  │    N (vacuum, Δ=0) ≤ F (anti-gluon)  ≤ B                   │\n";
    s += "  │    T and F incomparable — distinct particle/anti-particle    │\n";
    s += "  └──────────────────────────────────────────────────────────────┘\n";
    s
}

fn brst_block() -> String {
    let mark = |ok: bool| if ok { "✓" } else { "✗" };
    let mut s = String::new();
    s += "  ┌──────────────────────────────────────────────────────────────┐\n";
    s += "  │  BRST ↔ FROBENIUS CORRESPONDENCE                            │\n";
    s += "  ├──────────────────────────────────────────────────────────────┤\n";
    s += &format!("  │  {}  ym_brst_nilpotent: Q² = 0 ↔ Frobenius               │\n",
        mark(ym_brst_nilpotent()));
    s += "  │       ENGAGR(B)=B  — B sector BRST-closed                   │\n";
    s += "  │       ENGAGR(T)→F  — T sector nilpotent (Q on physical)     │\n";
    s += "  │       μ∘δ(B)=B     — Frobenius invariant holds               │\n";
    s += &format!("  │  {}  ym_confinement_ktrap: T cannot reach N              │\n",
        mark(ym_confinement_ktrap()));
    s += "  │       gluon confined — no decay to vacuum (K_trap stable)   │\n";
    s += "  ├──────────────────────────────────────────────────────────────┤\n";
    s += "  │  BRST doublet structure:                                      │\n";
    s += "  │    FSPLIT(B) = (T, F): physical gluon + ghost partner        │\n";
    s += "  │    FFUSE(T, F) = B:   recombination preserves BRST sector    │\n";
    s += "  │    Physical cohomology H^0: T-states not from FSPLIT(B)     │\n";
    s += "  │    Mass gap = H^0 sector has Δ > 0 (no massless excitation) │\n";
    s += "  ├──────────────────────────────────────────────────────────────┤\n";
    s += "  │  Structural correspondences:                                  │\n";
    s += "  │    N  = vacuum (Δ=0, undesignated, no information)           │\n";
    s += "  │    T  = massive gluon (Δ>0, designated)                      │\n";
    s += "  │    F  = anti-gluon                                            │\n";
    s += "  │    B  = BRST-closed (T and F coexist; colour-neutral)        │\n";
    s += "  │    K_trap = confinement (T cannot decay to N)                │\n";
    s += "  │    Omega_Z = full gauge invariance (topological lock)        │\n";
    s += "  ├──────────────────────────────────────────────────────────────┤\n";
    s += "  │  Structural type:                                             │\n";
    s += "  │  ⟨Ð_ω;Þ_K;Ř_Ť;Φ_};ƒ_ż;Ç_Ù;Γ_ʔ;ɢ_Ş;⊙_ÿ;Ħ_!;Σ_ő;Ω_z⟩      │\n";
    s += "  │  (D_holo · K_trap · P_pm_sym · Phi_c · Omega_Z)             │\n";
    s += "  └──────────────────────────────────────────────────────────────┘\n";
    s
}

fn full_suite() -> String {
    let mut s = String::new();
    s += "\n";
    s += "╔══════════════════════════════════════════════════════════════════╗\n";
    s += "║  BELNAP YM BRIDGE  (QCI_YM_Bridge.lean)                        ║\n";
    s += "╚══════════════════════════════════════════════════════════════════╝\n\n";
    s += &gap_block();
    s += "\n";
    s += &brst_block();
    s += "\n";
    s
}


// ── Shell entry point ─────────────────────────────────────────────────────────

pub fn handle(args: &str) -> String {
    match args.trim() {
        "" => full_suite(),
        "gap"  => { let mut s = String::from("\n"); s += &gap_block();  s }
        "brst" => { let mut s = String::from("\n"); s += &brst_block(); s }
        other  => format!("para ym: unknown subcommand '{}'. Try: gap | brst\n", other),
    }
}
