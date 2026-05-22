// para_align_commands.rs — Dialetheic Alignment + P vs NP Bridge for exOS
//
// Usage:
//   para align             full DAT suite (bifurcation + seq algebra + P vs NP + Shor framing)
//   para align bifur       bifurcation point + DAT tri-equivalence
//   para align seq         measurement sequence algebra
//   para align pvsnp       P vs NP bridge (BelnapCircuit + one-way barrier)
//   para align shor <N> <a> dialetheicShor framing for single instance
//
// Lean reference: MillenniumAnkh/Imscribing/Paraconsistent/
//   DialetheicAlignment.lean, QCI_Sequences.lean, QCI_PvsNP_Bridge.lean,
//   Shor/DialetheicOperator.lean, Shor/FullPipeline.lean

#![allow(dead_code)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use crate::para_vm::{
    B4,
    dialetheic_image, b_is_only_bifurcation_point, dialetheic_alignment_tri,
    measure_cost, measure_step, collapse_irreversible,
    BelnapCircuit,
};
use crate::para_shor_commands;


// ── DAT block ─────────────────────────────────────────────────────────────────

fn dat_block() -> String {
    let mut s = String::new();
    s += "  ┌──────────────────────────────────────────────────────────────┐\n";
    s += "  │  DIALETHEIC ALIGNMENT THEOREM (DialetheicAlignment.lean)    │\n";
    s += "  ├──────────────────────────────────────────────────────────────┤\n";

    let arms = dialetheic_alignment_tri();
    let mark = |ok: bool| if ok { "✓" } else { "✗" };
    s += &format!("  │  {}  Arm 1 (Operational): Frobenius closure at B           │\n", mark(arms[0]));
    s += "  │      μ∘δ(B)=B  ·  δ(B)=(T,F) distinct                    │\n";
    s += &format!("  │  {}  Arm 2 (Logical):     only B is dialetheic             │\n", mark(arms[1]));
    s += "  │      B and ¬B both designated  ·  T,F,N are not          │\n";
    s += &format!("  │  {}  Arm 3 (Algebraic):   no explosion from B              │\n", mark(arms[2]));
    s += "  │      N undesignated  ·  B∧¬B=B (not void collapse)       │\n";
    s += "  ├──────────────────────────────────────────────────────────────┤\n";

    let bif = b_is_only_bifurcation_point();
    s += &format!("  │  {}  Bifurcation: only B produces distinct fsplit outputs   │\n", mark(bif));
    s += "  │      B→(T,F)  T→(T,T)  F→(F,F)  N→(N,N)                │\n";
    s += "  ├──────────────────────────────────────────────────────────────┤\n";

    s += "  │  dialetheicImage morphism (r0 ↦ Belnap tag):               │\n";
    for q in [B4::N, B4::T, B4::F, B4::B] {
        let img = dialetheic_image(q);
        s += &format!("  │    {} → {}                                                   │\n",
            q.name(), img.name());
    }
    s += "  └──────────────────────────────────────────────────────────────┘\n";
    s
}


// ── Measurement sequence algebra block ───────────────────────────────────────

fn seq_block() -> String {
    let mut s = String::new();
    s += "  ┌──────────────────────────────────────────────────────────────┐\n";
    s += "  │  MEASUREMENT SEQUENCE ALGEBRA (QCI_Sequences.lean)          │\n";
    s += "  ├──────────────────────────────────────────────────────────────┤\n";

    let mark = |ok: bool| if ok { "✓" } else { "✗" };

    let n_noop = measure_cost(B4::N, B4::T) == 0;
    let idemp  = measure_step(B4::T, B4::T) == B4::T;
    let pres   = measure_step(B4::B, B4::B) == B4::B;
    let t_cost = measure_cost(B4::B, B4::T) == 1;
    let b_cost = measure_cost(B4::B, B4::B) == 2;
    let col_ok = collapse_irreversible(B4::T)
              && collapse_irreversible(B4::F)
              && collapse_irreversible(B4::N);
    let wig_ok = measure_cost(B4::B, B4::B) + measure_cost(B4::B, B4::T) == 3;

    s += &format!("  │  {}  measure_N_noop:          cost(N, any)=0                │\n", mark(n_noop));
    s += &format!("  │  {}  measure_nonsuper_idemp:  measure(T, T-bias)=T          │\n", mark(idemp));
    s += &format!("  │  {}  B_bias_preserves_super:  step(B, B-bias)=B             │\n", mark(pres));
    s += &format!("  │  {}  T_bias_coherence:         cost(B, T-bias)=1            │\n", mark(t_cost));
    s += &format!("  │  {}  B_bias_coherence:         cost(B, B-bias)=2            │\n", mark(b_cost));
    s += &format!("  │  {}  collapse_irreversible:    T/F/N cannot reach B         │\n", mark(col_ok));
    s += &format!("  │  {}  wigner_then_collapse:     B-bias(2)+T-bias(1)=3        │\n", mark(wig_ok));
    s += "  ├──────────────────────────────────────────────────────────────┤\n";
    s += "  │  Wigner's Friend sequence (n qubits):                        │\n";
    s += "  │    B-bias path:   cost = 2n  (B preserved)                   │\n";
    s += "  │    T-bias path:   cost = n   (B collapsed → T)               │\n";
    s += "  │    combined:      cost = 3n  (B-bias then T-bias on B)        │\n";
    s += "  └──────────────────────────────────────────────────────────────┘\n";
    s
}


// ── P vs NP bridge block ──────────────────────────────────────────────────────

fn pvsnp_block() -> String {
    let mut s = String::new();
    s += "  ┌──────────────────────────────────────────────────────────────┐\n";
    s += "  │  P vs NP BRIDGE (QCI_PvsNP_Bridge.lean)                     │\n";
    s += "  ├──────────────────────────────────────────────────────────────┤\n";

    let mark = |ok: bool| if ok { "✓" } else { "✗" };

    // sustain_never_collapses: all-B size-4 circuit stable
    let c_all_b = BelnapCircuit::new(alloc::vec![B4::B; 4]);
    let sust = c_all_b.sustain_stable();
    s += &format!("  │  {}  sustain_never_collapses:   all-B stable under ops       │\n", mark(sust));

    // proj(all-B) = all-T; verify all-T circuit cannot self-join to B
    let c_cls = BelnapCircuit::new(alloc::vec![B4::T, B4::T, B4::T]);
    let barrier = c_cls.classical_cannot_become_b();
    s += &format!("  │  {}  classical_cannot_become_B: proj(B^3) barrier holds      │\n", mark(barrier));

    s += "  │                                                              │\n";
    s += "  │  Structural correspondences:                                 │\n";
    s += "  │    B   = NP certificate (dual-designated witness)            │\n";
    s += "  │    T   = classically verified (IFIX-stable)                  │\n";
    s += "  │    B→T projection = P verification step (classical check)    │\n";
    s += "  │    K_trap ↔ BelnapCircuit all-B (trapped, sustain-locked)    │\n";
    s += "  │    classical_cannot_become_B: structural P⊄NP direction      │\n";
    s += "  ├──────────────────────────────────────────────────────────────┤\n";
    s += "  │  belnap_ktrap_statement:                                      │\n";
    s += "  │    An all-B BelnapCircuit cannot be produced by any           │\n";
    s += "  │    classical (T/F) circuit via lattice ops alone.             │\n";
    s += "  │    (join_circuit_B_dominant: proved — foldl induction)        │\n";
    s += "  └──────────────────────────────────────────────────────────────┘\n";
    s
}


// ── DialetheicShor framing block ──────────────────────────────────────────────

fn dialetheic_shor_block(instances: &[(usize, u64, u64)]) -> String {
    let mut s = String::new();
    s += "  ┌──────────────────────────────────────────────────────────────┐\n";
    s += "  │  DIALETHEIC SHOR OPERATOR (DialetheicOperator.lean)         │\n";
    s += "  │  Frame: Φ_υ (P_psi, momentum) → Φ_} (P_pm_sym, Frobenius)  │\n";
    s += "  ├──────┬────┬───┬───────┬───────┬─────────────────────────────┤\n";
    s += "  │   N  │  a │ n │   r   │ ratio │ Φ_} gap status              │\n";
    s += "  ├──────┼────┼───┼───────┼───────┼─────────────────────────────┤\n";

    for &(n, a, cap_n) in instances {
        // Use the Shor run to get coherence data
        let b_cost = 2 * n as u64;
        let t_cost = n as u64;
        let ratio_ok = b_cost == 2 * t_cost;
        let r = para_shor_commands::period_of(a, cap_n);
        s += &format!("  │ {:4} │ {:2} │ {:1} │   {:3}   │   2:1  │  B-only extraction: open    │\n",
            cap_n, a, n, r);
        let _ = ratio_ok;
    }

    s += "  ├──────────────────────────────────────────────────────────────┤\n";
    s += "  │  Φ_} gap: extracting r from B-bias alone is the open problem │\n";
    s += "  │  shor_pipeline_tier proved O_1 (FullPipeline.lean)           │\n";
    s += "  │  Bottleneck: B-only period extraction without T collapse      │\n";
    s += "  └──────────────────────────────────────────────────────────────┘\n";
    s
}


// ── Full suite ────────────────────────────────────────────────────────────────

fn full_suite() -> String {
    let mut s = String::new();
    s += "\n";
    s += "╔══════════════════════════════════════════════════════════════════╗\n";
    s += "║  BELNAP DIALETHEIC ALIGNMENT  (DialetheicAlignment.lean)       ║\n";
    s += "╚══════════════════════════════════════════════════════════════════╝\n\n";
    s += &dat_block();
    s += "\n";
    s += &seq_block();
    s += "\n";
    s += &pvsnp_block();
    s += "\n";
    s += &dialetheic_shor_block(&[(4, 7, 15), (5, 5, 21), (6, 2, 35)]);
    s += "\n";
    s
}


// ── Shell entry point ─────────────────────────────────────────────────────────

pub fn handle(args: &str) -> String {
    match args.trim() {
        "" => full_suite(),
        "bifur" => {
            let mut s = String::from("\n");
            s += &dat_block();
            s
        }
        "seq" => {
            let mut s = String::from("\n");
            s += &seq_block();
            s
        }
        "pvsnp" => {
            let mut s = String::from("\n");
            s += &pvsnp_block();
            s
        }
        rest if rest.starts_with("shor") => {
            let tail = rest["shor".len()..].trim();
            let parts: Vec<&str> = tail.split_whitespace().collect();
            if parts.len() != 2 {
                return "para align shor: usage: para align shor <N> <a>\n".into();
            }
            let cap_n: u64 = match parts[0].parse() {
                Ok(v) => v,
                Err(_) => return "para align shor: N must be a number\n".into(),
            };
            let a: u64 = match parts[1].parse() {
                Ok(v) => v,
                Err(_) => return "para align shor: a must be a number\n".into(),
            };
            let n = (u64::BITS - cap_n.saturating_sub(1).leading_zeros()) as usize;
            let n = n.max(1);
            let mut s = String::from("\n");
            s += &dialetheic_shor_block(&[(n, a, cap_n)]);
            s
        }
        other => format!("para align: unknown subcommand '{}'. Try: bifur | seq | pvsnp | shor <N> <a>\n", other),
    }
}
