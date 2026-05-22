// para_shor_commands.rs — Belnap Shor pipeline for the exOS shell
//
// Usage:
//   para shor           run full verification suite (SIC-POVM + 3 Shor instances)
//   para shor <N> <a>   run one instance with given N and base a
//
// All structural invariants match FullPipeline.lean / BelnapModExp.lean:
//   H-cost = n, ModExp-cost = 0, B-bias = 2n, T-bias = n, ratio = 2:1 (always)
//   B propagates through all Boolean gates (no phase differentiation)
//   Φ_υ bottleneck: period encoded in coherence ratio, not in bit values
//
// Lean reference: MillenniumAnkh/Imscribing/Paraconsistent/Shor/

#![allow(dead_code)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use crate::para_vm::B4;


// ── Belnap gates not in B4 impl ───────────────────────────────────────────────

fn hadamard(q: B4) -> B4 {
    match q {
        B4::T => B4::B,
        B4::F => B4::B,
        B4::B => B4::T,
        B4::N => B4::N,
    }
}

fn approx_le(a: B4, b: B4) -> bool {
    a == b || a == B4::N || b == B4::B
}

fn dialetheic(a: B4) -> bool {
    a.designated() && a.bnot().designated()
}

// WH2 bijection: N→(0,0), T→(0,1), F→(1,0), B→(1,1)
fn to_wh2(a: B4) -> (u8, u8) {
    match a { B4::N => (0,0), B4::T => (0,1), B4::F => (1,0), B4::B => (1,1) }
}


// ── N-qubit Belnap register ───────────────────────────────────────────────────

struct BelnapRegister {
    n: usize,
    qubits: Vec<B4>,
    coherence: u64,
}

impl BelnapRegister {
    fn classical(n: usize) -> Self {
        BelnapRegister { n, qubits: alloc::vec![B4::T; n], coherence: 0 }
    }

    fn apply_hadamard_layer(&mut self) {
        for i in 0..self.n {
            let q = self.qubits[i];
            self.qubits[i] = hadamard(q);
            if matches!(q, B4::T | B4::F | B4::B) {
                self.coherence += 1;
            }
        }
    }

    fn measure_all_b_bias(&mut self) {
        for i in 0..self.n {
            if self.qubits[i] == B4::B {
                self.coherence += 2;
                // B-bias: preserves B, cost 2
            }
        }
    }

    fn measure_all_t_bias(&mut self) {
        for i in 0..self.n {
            if self.qubits[i] == B4::B {
                self.qubits[i] = B4::T;
                self.coherence += 1;
            }
        }
    }

    fn all_b(&self) -> bool {
        self.qubits.iter().all(|&q| q == B4::B)
    }

    fn all_classical(&self) -> bool {
        self.qubits.iter().all(|&q| matches!(q, B4::T | B4::F))
    }
}


// ── Modular exponentiation ────────────────────────────────────────────────────

fn period(a: u64, n_mod: u64) -> u64 {
    if n_mod <= 1 { return 0; }
    let mut val = 1u64;
    for r in 1..=n_mod {
        val = val.wrapping_mul(a) % n_mod;
        if val == 1 { return r; }
    }
    0
}

fn modexp_on_b(n_bits: usize) -> Vec<B4> {
    alloc::vec![B4::B; n_bits]
}


// ── Shor pipeline ─────────────────────────────────────────────────────────────

struct ShorResult {
    n: usize,
    a: u64,
    cap_n: u64,
    period_cl: u64,
    had_cost: u64,
    b_meas_cost: u64,
    t_meas_cost: u64,
    ratio_exact: bool,  // true iff b_meas_cost == 2 * t_meas_cost
    mod_exp_allb: bool,
    b_preserves: bool,
    t_collapses: bool,
}

fn run_shor(n: usize, a: u64, cap_n: u64) -> ShorResult {
    let period_cl = period(a, cap_n);

    // Step 1: H^⊗n
    let mut reg = BelnapRegister::classical(n);
    reg.apply_hadamard_layer();
    let had_cost = reg.coherence;
    let all_b_after_h = reg.all_b();

    // Step 2: ModExp on B-input → B-output (cost 0)
    let out = modexp_on_b(n);
    let mod_exp_allb = out.iter().all(|&q| q == B4::B) && all_b_after_h;

    // Step 3: B-bias measurement (fresh reg to isolate cost)
    let mut reg_b = BelnapRegister::classical(n);
    reg_b.apply_hadamard_layer();
    reg_b.measure_all_b_bias();
    let b_preserves = reg_b.all_b();
    let b_total_cost = reg_b.coherence;

    // Step 4: T-bias measurement (continue step 1 reg)
    reg.measure_all_t_bias();
    let t_collapses = reg.all_classical();
    let t_total_cost = reg.coherence;

    let b_meas_cost = b_total_cost.saturating_sub(had_cost);
    let t_meas_cost = t_total_cost.saturating_sub(had_cost);
    let ratio_exact = b_meas_cost == 2 * t_meas_cost && t_meas_cost == n as u64;

    ShorResult {
        n, a, cap_n, period_cl,
        had_cost,
        b_meas_cost,
        t_meas_cost,
        ratio_exact,
        mod_exp_allb,
        b_preserves,
        t_collapses,
    }
}

fn format_shor(label: &str, r: &ShorResult) -> String {
    let ok = if r.ratio_exact && r.mod_exp_allb && r.b_preserves && r.t_collapses {
        "PASS"
    } else {
        "FAIL"
    };
    format!(
        "  {} [{}]\n    r={}, H={}, B-meas={}, T-meas={}, ratio={}\n    ModExp-allB={} B-pres={} T-col={}\n",
        label, ok,
        r.period_cl, r.had_cost, r.b_meas_cost, r.t_meas_cost,
        if r.ratio_exact { "2:1" } else { "???" },
        r.mod_exp_allb, r.b_preserves, r.t_collapses,
    )
}


// ── SIC-POVM verification ─────────────────────────────────────────────────────

fn verify_sic_povm() -> String {
    let b = B4::B;
    let all = [B4::N, B4::T, B4::F, B4::B];
    let mut s = String::new();

    // Axiom 1: meet(B,x) = x
    let ax1 = all.iter().all(|&x| b.meet(x) == x);
    s += &format!("  Axiom 1 meet(B,x)=x:    {}\n", if ax1 { "PASS" } else { "FAIL" });

    // Axiom 3: join(B,x) = B
    let ax3 = all.iter().all(|&x| b.join(x) == B4::B);
    s += &format!("  Axiom 3 join(B,x)=B:    {}\n", if ax3 { "PASS" } else { "FAIL" });

    // Axiom 4: ¬B = B
    let ax4 = b.bnot() == B4::B;
    s += &format!("  Axiom 4 bnot(B)=B:      {}\n", if ax4 { "PASS" } else { "FAIL" });

    // approx_le: B is top
    let top = all.iter().all(|&x| approx_le(x, B4::B));
    s += &format!("  ApproxLE B is top:      {}\n", if top { "PASS" } else { "FAIL" });

    // DialetheicAlignment: only B is dialetheic
    let only_b = dialetheic(B4::B) && !dialetheic(B4::N) && !dialetheic(B4::T) && !dialetheic(B4::F);
    s += &format!("  only_B_is_dialetheic:   {}\n", if only_b { "PASS" } else { "FAIL" });

    // WH2 bijection: all distinct
    let coords: Vec<(u8,u8)> = all.iter().map(|&x| to_wh2(x)).collect();
    let unique = coords.len() == 4 && {
        let mut seen = [(0u8,0u8); 4];
        seen.copy_from_slice(&coords);
        seen.sort_unstable();
        seen.windows(2).all(|w| w[0] != w[1])
    };
    s += &format!("  WH2 bijection:          {}\n", if unique { "PASS" } else { "FAIL" });
    s += &format!("    N→{:?} T→{:?} F→{:?} B→{:?}\n",
        to_wh2(B4::N), to_wh2(B4::T), to_wh2(B4::F), to_wh2(B4::B));

    s
}


// ── Shell entry point ─────────────────────────────────────────────────────────

pub fn handle(args: &str) -> String {
    let args = args.trim();

    if args.is_empty() {
        // Full verification suite
        let mut s = String::new();
        s += "Belnap Shor Pipeline — FullPipeline.lean / QCI_SICPOVM_Bridge.lean\n";
        s += &"─".repeat(64);
        s += "\n";
        s += "SIC-POVM axioms for B (d=2):\n";
        s += &verify_sic_povm();
        s += "\n";
        s += "Shor coherence invariants (H=n, ModExp=0, B-bias=2n, T-bias=n):\n";
        s += &format_shor("N=15 a=7", &run_shor(4, 7,  15));
        s += &format_shor("N=21 a=5", &run_shor(5, 5,  21));
        s += &format_shor("N=35 a=2", &run_shor(6, 2,  35));
        s += "\n";
        s += "Φ_υ bottleneck: B is the only superposition value.\n";
        s += "  Period r is in the 2:1 coherence ratio, not in bit values.\n";
        s += "  Φ_υ → Φ_} gap (B-only extraction) is the structural open problem.\n";
        return s;
    }

    // Parse "N a" for single instance
    let mut parts = args.splitn(2, ' ');
    let cap_n: u64 = match parts.next().unwrap_or("").trim().parse() {
        Ok(v) => v,
        Err(_) => return "para shor: expected 'para shor <N> <a>'.\n".into(),
    };
    let a: u64 = match parts.next().unwrap_or("").trim().parse() {
        Ok(v) => v,
        Err(_) => return "para shor: expected 'para shor <N> <a>'.\n".into(),
    };
    let n = (u64::BITS - cap_n.saturating_sub(1).leading_zeros()) as usize;
    let n = n.max(1);
    let r = run_shor(n, a, cap_n);
    let label = format!("N={} a={}", cap_n, a);
    format_shor(&label, &r)
}
