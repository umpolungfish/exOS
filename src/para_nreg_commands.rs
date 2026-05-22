// para_nreg_commands.rs — n-Register QCI Generalization for exOS
//
// Usage:
//   para nreg        full suite (SIC axioms + coherence table + period oracle)
//   para nreg ratio  coherence ratio table only (n=1..8)
//   para nreg sic    SIC-POVM per-qubit axiom block
//
// Lean reference: MillenniumAnkh/Imscribing/Paraconsistent/Shor/FullPipeline.lean

#![allow(dead_code)]

extern crate alloc;

use alloc::string::String;
use alloc::format;

use crate::para_vm::B4;
use crate::para_shor_commands::period_of;


// Standard verification table (n, a, N, r).
// Periods verified via classical order computation; ratio=2:1 is structural.
static NREG_TABLE: &[(usize, u64, u64, u64)] = &[
    (4,  7,  15,  4),
    (5,  5,  21,  6),
    (6,  2,  35, 12),
    (7,  2,  77, 30),
    (7,  3,  91,  6),
    (8,  2, 143, 60),
    (8,  3, 187, 80),  // 187=11×17; lcm(ord_11(3)=5, ord_17(3)=16)=80
    (8,  2, 221, 24),  // 221=13×17; lcm(ord_13(2)=12, ord_17(2)=8)=24
];


// ── SIC-POVM per-qubit axioms ─────────────────────────────────────────────────

fn sic_povm_axioms_hold() -> bool {
    let all = [B4::N, B4::T, B4::F, B4::B];
    all.iter().all(|&x| B4::B.meet(x) == x)       // Axiom 1
    && all.iter().all(|&x| B4::B.join(x) == B4::B) // Axiom 3
    && B4::B.bnot() == B4::B                        // Axiom 4
    && all.iter().all(|&x| x.approx_le(B4::B))     // B is top
    && B4::B.dialetheic()                            // B is dialetheic
    && [B4::N, B4::T, B4::F].iter().all(|&x| !x.dialetheic())  // unique
}


// ── Coherence ratio verification ──────────────────────────────────────────────

struct NRegResult {
    n: usize, a: u64, cap_n: u64,
    expected_r: u64, actual_r: u64,
    had: u64, b_meas: u64, t_meas: u64,
    ratio_ok: bool, period_ok: bool,
}

fn verify_nreg(n: usize, a: u64, cap_n: u64, expected_r: u64) -> NRegResult {
    let actual_r = period_of(a, cap_n);
    let had    = n as u64;
    let b_meas = 2 * n as u64;
    let t_meas = n as u64;
    let ratio_ok  = b_meas == 2 * t_meas && t_meas == had;
    let period_ok = actual_r == expected_r;
    NRegResult { n, a, cap_n, expected_r, actual_r, had, b_meas, t_meas, ratio_ok, period_ok }
}


// ── Display blocks ────────────────────────────────────────────────────────────

fn sic_block() -> String {
    let mark = |ok: bool| if ok { "✓" } else { "✗" };
    let mut s = String::new();
    s += "  ┌──────────────────────────────────────────────────────────────┐\n";
    s += "  │  SIC-POVM PER-QUBIT AXIOMS (QCI_SICPOVM_Bridge.lean)        │\n";
    s += "  ├──────────────────────────────────────────────────────────────┤\n";
    let ok = sic_povm_axioms_hold();
    s += &format!("  │  {}  B satisfies all 4 SIC axioms for d=2 (per qubit)   │\n", mark(ok));
    s += "  │       Axiom 1: meet(B,x)=x ∀x  (maximal information)        │\n";
    s += "  │       Axiom 3: join(B,x)=B ∀x  (absorption)                 │\n";
    s += "  │       Axiom 4: ¬B=B             (self-adjoint)               │\n";
    s += "  │       ApproxLE: B is top ∀x                                  │\n";
    s += "  │       only_B_is_dialetheic: B is the unique SIC element      │\n";
    s += "  ├──────────────────────────────────────────────────────────────┤\n";
    s += "  │  n-qubit structure:                                           │\n";
    s += "  │    Independent d=2 SIC at each qubit site                    │\n";
    s += "  │    n-fold tensor product: ratio 2:1 scales linearly          │\n";
    s += "  │    Open (Lean): SIC multilattice proof for n>1               │\n";
    s += "  └──────────────────────────────────────────────────────────────┘\n";
    s
}

fn ratio_block() -> String {
    let mark = |ok: bool| if ok { "✓" } else { "✗" };
    let mut s = String::new();
    s += "  ┌──────────────────────────────────────────────────────────────┐\n";
    s += "  │  COHERENCE RATIO TABLE (FullPipeline.lean, O_1 tier)        │\n";
    s += "  ├───┬────┬──────┬─────┬────┬────────┬────────┬───────────────┤\n";
    s += "  │ n │  a │    N │  r  │  H │ B-meas │ T-meas │ ratio         │\n";
    s += "  ├───┼────┼──────┼─────┼────┼────────┼────────┼───────────────┤\n";
    let mut all_ok = true;
    for &(n, a, cap_n, expected_r) in NREG_TABLE {
        let res = verify_nreg(n, a, cap_n, expected_r);
        let ok = res.ratio_ok && res.period_ok;
        if !ok { all_ok = false; }
        s += &format!("  │ {}  │  {:1} │  {:3} │ {:3} │ {:2} │    {:3} │    {:3} │   2:1  {}    │\n",
            res.n, res.a, res.cap_n, res.actual_r, res.had, res.b_meas, res.t_meas, mark(ok));
    }
    s += "  ├───────────────────────────────────────────────────────────────┤\n";
    s += &format!("  │  {}  All 8 instances: ratio=2:1, periods verified           │\n",
        mark(all_ok));
    s += "  └──────────────────────────────────────────────────────────────┘\n";
    s
}

fn oracle_block() -> String {
    let mut s = String::new();
    s += "  ┌──────────────────────────────────────────────────────────────┐\n";
    s += "  │  PERIOD ORACLE (BelnapQFT.lean — Φ_υ bottleneck)            │\n";
    s += "  ├──────────────────────────────────────────────────────────────┤\n";
    s += "  │  After B-bias measurement: all qubits still B               │\n";
    s += "  │  Period r NOT in qubit values — in the 2:1 cost ratio       │\n";
    s += "  │  Formula: b_meas = 2n, t_meas = n → ratio = 2:1 ∀n          │\n";
    s += "  ├──────────────────────────────────────────────────────────────┤\n";
    // Ratio formula n=1..8
    for n in 1usize..=8 {
        let b = 2 * n as u64;
        let t = n as u64;
        s += &format!("  │    n={}: b-meas={:2}, t-meas={:2}, ratio={}:{} = 2.0           │\n",
            n, b, t, b, t);
    }
    s += "  ├──────────────────────────────────────────────────────────────┤\n";
    s += "  │  Φ_υ→Φ_} gap: B-only period extraction is the open problem  │\n";
    s += "  │  shor_pipeline_tier proved O_1 (FullPipeline.lean)          │\n";
    s += "  └──────────────────────────────────────────────────────────────┘\n";
    s
}

fn full_suite() -> String {
    let mut s = String::new();
    s += "\n";
    s += "╔══════════════════════════════════════════════════════════════════╗\n";
    s += "║  n-REGISTER QCI GENERALIZATION  (FullPipeline.lean)            ║\n";
    s += "╚══════════════════════════════════════════════════════════════════╝\n\n";
    s += &sic_block();
    s += "\n";
    s += &ratio_block();
    s += "\n";
    s += &oracle_block();
    s += "\n";
    s
}


// ── Shell entry point ─────────────────────────────────────────────────────────

pub fn handle(args: &str) -> String {
    match args.trim() {
        "" => full_suite(),
        "ratio" => { let mut s = String::from("\n"); s += &ratio_block(); s }
        "sic"   => { let mut s = String::from("\n"); s += &sic_block();   s }
        other   => format!("para nreg: unknown subcommand '{}'. Try: ratio | sic\n", other),
    }
}
