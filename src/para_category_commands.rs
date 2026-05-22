// para_category_commands.rs — Belnap Lattice as Category for exOS
//
// Usage:
//   para category        full suite (initial/terminal + key theorems)
//   para category obj    object structure (initial/terminal, arrows)
//   para category thm    key theorems (meet/join/Frobenius)
//
// Lean reference: MillenniumAnkh/Imscribing/Paraconsistent/BelnapCategory.lean

#![allow(dead_code)]

extern crate alloc;

use alloc::string::String;
use alloc::format;

use crate::para_vm::{B4, ParaKernel};


// ── Category structure ────────────────────────────────────────────────────────

fn category_terminal() -> bool {
    [B4::N, B4::T, B4::F, B4::B].iter().all(|&x| x.approx_le(B4::B))
}

fn category_initial() -> bool {
    [B4::N, B4::T, B4::F, B4::B].iter().all(|&x| B4::N.approx_le(x))
}

fn category_no_other_terminal() -> bool {
    let all = [B4::N, B4::T, B4::F, B4::B];
    for &c in &[B4::N, B4::T, B4::F] {
        if all.iter().all(|&x| x.approx_le(c)) { return false; }
    }
    true
}

fn category_no_other_initial() -> bool {
    for &c in &[B4::T, B4::F, B4::B] {
        let all = [B4::N, B4::T, B4::F, B4::B];
        if all.iter().all(|&x| c.approx_le(x)) { return false; }
    }
    true
}

fn band_b_idempotent() -> bool { B4::B.band(B4::B) == B4::B }

fn b_meet_is_id() -> bool {
    [B4::N, B4::T, B4::F, B4::B].iter().all(|&x| B4::B.meet(x) == x)
}

fn b_join_absorbs() -> bool {
    [B4::N, B4::T, B4::F, B4::B].iter().all(|&x| B4::B.join(x) == B4::B)
}

fn n_meet_annihilates() -> bool {
    [B4::N, B4::T, B4::F, B4::B].iter().all(|&x| B4::N.meet(x) == B4::N)
}

fn b_self_adjoint() -> bool { B4::B.bnot() == B4::B && B4::N.bnot() == B4::N }

fn frobenius_terminal_roundtrip() -> bool { ParaKernel::frobenius_invariant(B4::B) }

fn category_is_o_inf() -> bool {
    frobenius_terminal_roundtrip() && B4::B.bnot() == B4::B && B4::B.designated()
}


// ── Display blocks ────────────────────────────────────────────────────────────

fn obj_block() -> String {
    let mark = |ok: bool| if ok { "✓" } else { "✗" };
    let mut s = String::new();
    s += "  ┌──────────────────────────────────────────────────────────────┐\n";
    s += "  │  CATEGORY STRUCTURE (BelnapCategory.lean)                   │\n";
    s += "  ├──────────────────────────────────────────────────────────────┤\n";
    s += &format!("  │  {}  B is terminal: approx_le(x,B) ∀x                   │\n",
        mark(category_terminal()));
    s += &format!("  │  {}  N is initial:  approx_le(N,x) ∀x                   │\n",
        mark(category_initial()));
    s += &format!("  │  {}  B is the unique terminal object                      │\n",
        mark(category_no_other_terminal()));
    s += &format!("  │  {}  N is the unique initial object                       │\n",
        mark(category_no_other_initial()));
    s += "  ├──────────────────────────────────────────────────────────────┤\n";
    s += "  │  Approximation order (morphism arrows):                      │\n";
    for src in [B4::N, B4::T, B4::F, B4::B] {
        let targets: alloc::vec::Vec<&str> = [B4::N, B4::T, B4::F, B4::B]
            .iter()
            .filter(|&&t| src.approx_le(t))
            .map(|t| t.name())
            .collect();
        s += &format!("  │    {} → {}  │\n", src.name(), targets.join(", "));
    }
    s += "  └──────────────────────────────────────────────────────────────┘\n";
    s
}

fn thm_block() -> String {
    let mark = |ok: bool| if ok { "✓" } else { "✗" };
    let mut s = String::new();
    s += "  ┌──────────────────────────────────────────────────────────────┐\n";
    s += "  │  KEY THEOREMS                                                │\n";
    s += "  ├──────────────────────────────────────────────────────────────┤\n";
    s += &format!("  │  {}  band_B_idempotent: band(B,B)=B                       │\n",
        mark(band_b_idempotent()));
    s += &format!("  │  {}  B_meet_is_id: meet(B,x)=x ∀x                        │\n",
        mark(b_meet_is_id()));
    s += "  │       B is the meet-identity (SIC equiangular projection)   │\n";
    s += &format!("  │  {}  B_join_absorbs: join(B,x)=B ∀x                      │\n",
        mark(b_join_absorbs()));
    s += &format!("  │  {}  N_meet_annihilates: meet(N,x)=N ∀x                  │\n",
        mark(n_meet_annihilates()));
    s += &format!("  │  {}  B self-adjoint (dagger): bnot(B)=B, bnot(N)=N       │\n",
        mark(b_self_adjoint()));
    s += &format!("  │  {}  Frobenius = terminal roundtrip: μ∘δ(B)=B            │\n",
        mark(frobenius_terminal_roundtrip()));
    s += &format!("  │  {}  category_is_O_inf: Phi_c ∧ P_pm_sym                 │\n",
        mark(category_is_o_inf()));
    s += "  └──────────────────────────────────────────────────────────────┘\n";
    s
}

fn full_suite() -> String {
    let mut s = String::new();
    s += "\n";
    s += "╔══════════════════════════════════════════════════════════════════╗\n";
    s += "║  BELNAP LATTICE AS CATEGORY  (BelnapCategory.lean)             ║\n";
    s += "╚══════════════════════════════════════════════════════════════════╝\n\n";
    s += &obj_block();
    s += "\n";
    s += &thm_block();
    s += "\n";
    s
}


// ── Shell entry point ─────────────────────────────────────────────────────────

pub fn handle(args: &str) -> String {
    match args.trim() {
        "" => full_suite(),
        "obj" => { let mut s = String::from("\n"); s += &obj_block(); s }
        "thm" => { let mut s = String::from("\n"); s += &thm_block(); s }
        other => format!("para category: unknown '{}'. Try: obj | thm\n", other),
    }
}
