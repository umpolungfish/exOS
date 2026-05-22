// para_temporal_commands.rs — Belnap Temporal Logic for exOS
//
// Usage:
//   para temporal        full suite (modalities + trajectory + winding)
//   para temporal traj   trajectory display only
//   para temporal modal  modality checks only
//
// Lean reference: MillenniumAnkh/Imscribing/Paraconsistent/BelnapTemporal.lean

#![allow(dead_code)]

extern crate alloc;

use alloc::string::String;
use alloc::format;

use crate::para_vm::{B4, ParaKernel};


// ── Temporal checks ───────────────────────────────────────────────────────────

// □(r0=B ∧ r1=B ∧ r2=B): verify over n_cycles.
fn always_b_registers(n_cycles: u64) -> bool {
    let mut s = ParaKernel::initial();
    for _ in 0..n_cycles {
        if s.r0 != B4::B || s.r1 != B4::B || s.r2 != B4::B { return false; }
        s = s.run(1);
    }
    true
}

// bnot∘r0 = r0 at every step: the trajectory is winding-invariant.
fn winding_invariant(n_cycles: u64) -> bool {
    let mut s = ParaKernel::initial();
    for _ in 0..n_cycles {
        if s.r0.bnot() != s.r0 { return false; }
        s = s.run(1);
    }
    true
}

// Phi_c ∧ P_pm_sym: the temporal B-state is O_inf.
fn temporal_is_o_inf() -> bool {
    let phi_c = B4::B.bnot() == B4::B && B4::B.designated();
    let frobenius = ParaKernel::frobenius_invariant(B4::B);
    phi_c && frobenius
}


// ── Display blocks ────────────────────────────────────────────────────────────

fn modal_block() -> String {
    let mark = |ok: bool| if ok { "✓" } else { "✗" };
    let mut s = String::new();
    s += "  ┌──────────────────────────────────────────────────────────────┐\n";
    s += "  │  TEMPORAL MODALITY CHECKS (BelnapTemporal.lean)             │\n";
    s += "  ├──────────────────────────────────────────────────────────────┤\n";
    s += &format!("  │  {}  □(r0=B ∧ r1=B ∧ r2=B) — always_B_registers        │\n",
        mark(always_b_registers(8)));
    s += "  │       B holds at every cycle (generalises run_B3)           │\n";
    s += &format!("  │  {}  winding_invariant: bnot(r0(t)) = r0(t) ∀t           │\n",
        mark(winding_invariant(8)));
    s += "  │       r0 is always B; bnot(B)=B (self-negating trajectory) │\n";
    s += &format!("  │  {}  temporal_is_O_inf: Phi_c ∧ P_pm_sym                  │\n",
        mark(temporal_is_o_inf()));
    s += "  ├──────────────────────────────────────────────────────────────┤\n";
    s += "  │  Temporal modalities (B-locus):                              │\n";
    s += "  │    □B: B at every cycle                (run_B3 → □B)        │\n";
    s += "  │    ◇B: B at some cycle                 (trivially since □B) │\n";
    s += "  │    ○B: B at the next step              (trivially since □B) │\n";
    s += "  │  Winding: bnot∘r0 = r0 means no temporal phase shift        │\n";
    s += "  └──────────────────────────────────────────────────────────────┘\n";
    s
}

fn traj_block() -> String {
    let mut s = String::new();
    s += "  ┌──────────────────────────────────────────────────────────────┐\n";
    s += "  │  Kernel trajectory (8 cycles, initial r0=r1=r2=B)           │\n";
    s += "  ├──────────────────────────────────────────────────────────────┤\n";
    s += "  │   t │ r0 │ r1 │ r2 │ bnot(r0) │ wind │                     │\n";
    s += "  │  ───┼────┼────┼────┼──────────┼──────┤                     │\n";

    let mut state = ParaKernel::initial();
    for t in 0u64..=8 {
        let neg_r0 = state.r0.bnot();
        let wind = if neg_r0 == state.r0 { "✓" } else { "✗" };
        s += &format!("  │  {:3} │  {} │  {} │  {} │    {}     │  {}   │                     │\n",
            t, state.r0.name(), state.r1.name(), state.r2.name(), neg_r0.name(), wind);
        state = state.run(1);
    }
    s += "  └──────────────────────────────────────────────────────────────┘\n";
    s
}

fn full_suite() -> String {
    let mut s = String::new();
    s += "\n";
    s += "╔══════════════════════════════════════════════════════════════════╗\n";
    s += "║  BELNAP TEMPORAL LOGIC  (BelnapTemporal.lean)                  ║\n";
    s += "╚══════════════════════════════════════════════════════════════════╝\n\n";
    s += &modal_block();
    s += "\n";
    s += &traj_block();
    s += "\n";
    s
}


// ── Shell entry point ─────────────────────────────────────────────────────────

pub fn handle(args: &str) -> String {
    match args.trim() {
        "" => full_suite(),
        "traj"  => { let mut s = String::from("\n"); s += &traj_block();  s }
        "modal" => { let mut s = String::from("\n"); s += &modal_block(); s }
        other   => format!("para temporal: unknown '{}'. Try: traj | modal\n", other),
    }
}
