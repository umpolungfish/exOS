// para_shor_commands.rs — Belnap Shor pipeline for the exOS shell
//
// Usage:
//   para shor              full visual suite (SIC-POVM + pipeline diagram + 3 instances)
//   para shor <N> <a>      single instance
//   para shor loop [N]     indefinite accumulator — N cycles (default 40)
//
// Lean reference: MillenniumAnkh/Imscribing/Paraconsistent/Shor/

#![allow(dead_code)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use crate::para_vm::B4;


// ── Extra gates (not in B4 impl) ──────────────────────────────────────────────

fn hadamard(q: B4) -> B4 {
    match q { B4::T => B4::B, B4::F => B4::B, B4::B => B4::T, B4::N => B4::N }
}

fn approx_le(a: B4, b: B4) -> bool {
    a == b || a == B4::N || b == B4::B
}

fn dialetheic(a: B4) -> bool {
    a.designated() && a.bnot().designated()
}

fn to_wh2(a: B4) -> (u8, u8) {
    match a { B4::N => (0,0), B4::T => (0,1), B4::F => (1,0), B4::B => (1,1) }
}


// ── N-qubit register ──────────────────────────────────────────────────────────

struct Reg { n: usize, q: Vec<B4>, c: u64 }

impl Reg {
    fn classical(n: usize) -> Self { Reg { n, q: alloc::vec![B4::T; n], c: 0 } }
    fn hadamard_layer(&mut self) {
        for i in 0..self.n {
            let old = self.q[i];
            self.q[i] = hadamard(old);
            if matches!(old, B4::T | B4::F | B4::B) { self.c += 1; }
        }
    }
    fn measure_b_bias(&mut self) {
        for i in 0..self.n { if self.q[i] == B4::B { self.c += 2; } }
    }
    fn measure_t_bias(&mut self) {
        for i in 0..self.n {
            if self.q[i] == B4::B { self.q[i] = B4::T; self.c += 1; }
        }
    }
    fn all_b(&self) -> bool { self.q.iter().all(|&q| q == B4::B) }
    fn all_classical(&self) -> bool { self.q.iter().all(|&q| matches!(q, B4::T | B4::F)) }
}


// ── Period ────────────────────────────────────────────────────────────────────

fn period(a: u64, n: u64) -> u64 {
    if n <= 1 { return 0; }
    let mut val = 1u64;
    for r in 1..=n { val = val.wrapping_mul(a) % n; if val == 1 { return r; } }
    0
}

pub fn period_of(a: u64, n: u64) -> u64 { period(a, n) }


// ── Pipeline ──────────────────────────────────────────────────────────────────

struct ShorResult {
    n: usize, a: u64, cap_n: u64,
    period_cl: u64,
    had: u64, b_meas: u64, t_meas: u64,
    ok: bool,
}

fn run_shor(n: usize, a: u64, cap_n: u64) -> ShorResult {
    let period_cl = period(a, cap_n);
    let mut reg = Reg::classical(n);
    reg.hadamard_layer();
    let had = reg.c;

    let mut reg_b = Reg::classical(n);
    reg_b.hadamard_layer();
    reg_b.measure_b_bias();
    let b_pres = reg_b.all_b();
    let b_total = reg_b.c;

    reg.measure_t_bias();
    let t_col = reg.all_classical();
    let t_total = reg.c;

    let b_meas = b_total.saturating_sub(had);
    let t_meas = t_total.saturating_sub(had);
    let ok = b_meas == 2 * t_meas && t_meas == n as u64
        && b_pres && t_col && had == n as u64;

    ShorResult { n, a, cap_n, period_cl, had, b_meas, t_meas, ok }
}


// ── Visual pieces ─────────────────────────────────────────────────────────────

fn pipeline_diagram(n: usize) -> String {
    let mut s = String::new();
    s += "\n";
    s += "  ┌─────────┐        ┌─────────┐        ┌─────────┐\n";
    s += &format!(
        "  │ T^{}    │ H^⊗{n} │ B^{}    │ ModExp │ B^{}    │\n",
        n, n, n, n = n
    );
    s += "  │ classic │───────▶│ super-  │───────▶│ B-fixed │\n";
    s += &format!("  │         │ cost={n:<2} │ position│ cost=0  │         │\n", n = n);
    s += "  └─────────┘        └─────────┘        └────┬────┘\n";
    s += "                                              │\n";
    s += "                                 ┌────────────┴───────────┐\n";
    s += "                          B-bias │                        │ T-bias\n";
    s += &format!("                          cost={:<2} │                        │ cost={:<2}\n", 2 * n, n);
    s += "                                 ▼                        ▼\n";
    s += "                          ┌─────────┐              ┌─────────┐\n";
    s += &format!("                          │ B^{}    │              │ T^{}    │\n", n, n);
    s += "                          │preserved│              │collapsed│\n";
    s += "                          │Wigner's │              │classical│\n";
    s += "                          │ Friend  │              │ output  │\n";
    s += "                          └─────────┘              └─────────┘\n";
    s += &format!("                            ratio = {}:{} (invariant ∀n)\n", 2 * n, n);
    s += "\n";
    s
}

fn sic_block() -> String {
    let b = B4::B;
    let all = [B4::N, B4::T, B4::F, B4::B];
    let mut s = String::new();
    s += "  ┌─────────────────────────────────────────────────────┐\n";
    s += "  │  SIC-POVM axioms for B — QCI_SICPOVM_Bridge.lean    │\n";
    s += "  ├─────────────────────────────────────────────────────┤\n";

    let ax1 = all.iter().all(|&x| b.meet(x) == x);
    let ax3 = all.iter().all(|&x| b.join(x) == B4::B);
    let ax4 = b.bnot() == B4::B;
    let top = all.iter().all(|&x| approx_le(x, B4::B));
    let only_b = dialetheic(B4::B)
        && !dialetheic(B4::N) && !dialetheic(B4::T) && !dialetheic(B4::F);
    let coords: Vec<(u8,u8)> = all.iter().map(|&x| to_wh2(x)).collect();
    let unique = {
        let mut seen = [(0u8,0u8); 4];
        seen.copy_from_slice(&coords);
        seen.sort_unstable();
        seen.windows(2).all(|w| w[0] != w[1])
    };

    let mark = |ok: bool| if ok { "✓" } else { "✗" };
    s += &format!("  │  {} Axiom 1  meet(B,x)=x ∀x  (maximal info)          │\n", mark(ax1));
    s += &format!("  │  {} Axiom 2  equiangularity  (= Axiom 1 for d=2)      │\n", mark(ax1));
    s += &format!("  │  {} Axiom 3  join(B,x)=B ∀x  (absorption)            │\n", mark(ax3));
    s += &format!("  │  {} Axiom 4  ¬B=B             (self-adjoint)          │\n", mark(ax4));
    s += &format!("  │  {} ApproxLE B is top ∀x                              │\n", mark(top));
    s += &format!("  │  {} only_B_is_dialetheic  (DialetheicAlignment.lean)  │\n", mark(only_b));
    s += &format!("  │  {} WH2 bijection  N→(0,0) T→(0,1) F→(1,0) B→(1,1) │\n", mark(unique));
    s += "  └─────────────────────────────────────────────────────┘\n";
    s
}

fn shor_table_header() -> String {
    let mut s = String::new();
    s += "  ┌───────┬──────┬─────┬────┬─────┬───┬────────┬────────┬───────┐\n";
    s += "  │ label │   N  │  a  │  n │  r  │ H │ B-meas │ T-meas │ ratio │\n";
    s += "  ├───────┼──────┼─────┼────┼─────┼───┼────────┼────────┼───────┤\n";
    s
}

fn shor_table_row(label: &str, r: &ShorResult) -> String {
    format!(
        "  │ {:5} │ {:4} │ {:3} │ {:2} │ {:3} │ {:1} │    {:3} │    {:3} │  {:3}  │ {}\n",
        label, r.cap_n, r.a, r.n, r.period_cl, r.had, r.b_meas, r.t_meas,
        if r.b_meas > 0 && r.t_meas > 0 {
            alloc::format!("{}:{}", r.b_meas / r.t_meas, 1)
        } else { "?:?".into() },
        if r.ok { "✓" } else { "✗" }
    )
}

fn shor_table_footer() -> String {
    "  └───────┴──────┴─────┴────┴─────┴───┴────────┴────────┴───────┘\n".into()
}


// ── Full verification suite ───────────────────────────────────────────────────

fn full_suite() -> String {
    let mut s = String::new();
    s += "\n";
    s += "╔══════════════════════════════════════════════════════════════╗\n";
    s += "║  BELNAP SHOR PIPELINE  (FullPipeline.lean · O₁ · Φ_υ)     ║\n";
    s += "╚══════════════════════════════════════════════════════════════╝\n";

    s += &pipeline_diagram(4);
    s += &sic_block();
    s += "\n";

    let instances: &[(&str, usize, u64, u64)] = &[
        ("N=15", 4, 7,  15),
        ("N=21", 5, 5,  21),
        ("N=35", 6, 2,  35),
    ];

    s += &shor_table_header();
    for &(label, n, a, cap_n) in instances {
        let r = run_shor(n, a, cap_n);
        s += &shor_table_row(label, &r);
    }
    s += &shor_table_footer();

    s += "\n";
    s += "  Φ_υ bottleneck: B is the only superposition value.\n";
    s += "  Period r lives in the 2:1 coherence ratio, not in the bits.\n";
    s += "  Φ_υ → Φ_} gap: B-only period extraction is the open problem.\n";
    s += "\n";
    s
}


// ── Loop accumulator ──────────────────────────────────────────────────────────

static LOOP_TABLE: &[(usize, u64, u64)] = &[
    (4,  7,  15),   // r=4
    (5,  5,  21),   // r=6
    (6,  2,  35),   // r=12
    (7,  2,  77),   // r=30
    (7,  3,  91),   // r=6
    (8,  2, 143),   // r=60
    (8,  3, 187),   // r=80  (187=11×17; lcm(ord_11(3)=5, ord_17(3)=16)=80)
    (8,  2, 221),   // r=24  (221=13×17; lcm(ord_13(2)=12, ord_17(2)=8)=24)
];

fn run_loop_n(n_cycles: u64) -> String {
    let mut s = String::new();
    s += "\n";
    s += "╔══════════════════════════════════════════════════════════════════════╗\n";
    s += "║  para shor loop — Belnap Coherence Accumulator                     ║\n";
    s += "║  H=n  ModExp=0  B-bias=2n  T-bias=n  ratio=2:1  (always)          ║\n";
    s += "╚══════════════════════════════════════════════════════════════════════╝\n";
    s += "\n";
    s += "  cycle │   N │  a │  n │  r  │  H │ B-meas │ T-meas │ ratio │  accum\n";
    s += "  ──────┼─────┼────┼────┼─────┼────┼────────┼────────┼───────┼────────\n";

    let mut accum: u64 = 0;
    let mut cycles_done: u64 = 0;
    let table_len = LOOP_TABLE.len() as u64;

    for i in 0..n_cycles {
        let &(n, a, cap_n) = &LOOP_TABLE[(i % table_len) as usize];
        let r = run_shor(n, a, cap_n);
        accum = accum.saturating_add(r.had + r.b_meas + r.t_meas);
        cycles_done += 1;
        s += &format!(
            "  {:5} │ {:3} │ {:2} │ {:2} │ {:3} │ {:2} │    {:3} │    {:3} │   2:1 │ {:6}\n",
            i + 1, cap_n, a, n, r.period_cl, r.had, r.b_meas, r.t_meas, accum
        );
    }

    s += "  ──────┴─────┴────┴────┴─────┴────┴────────┴────────┴───────┴────────\n";
    s += &format!("  cycles={cycles_done}  total_coherence_accumulated={accum}\n");
    s += &format!(
        "  average per cycle: {:.1}  (formula: H+2n+n = 4n per instance)\n",
        if cycles_done > 0 { accum as f64 / cycles_done as f64 } else { 0.0 }
    );
    s += "\n";
    s
}


// ── Shell entry point ─────────────────────────────────────────────────────────

pub fn handle(args: &str) -> String {
    let args = args.trim();

    if args.is_empty() {
        return full_suite();
    }

    let mut parts = args.splitn(2, ' ');
    match parts.next().unwrap_or("").trim() {
        "loop" => {
            let n: u64 = parts.next().unwrap_or("40").trim().parse().unwrap_or(40);
            run_loop_n(n)
        }
        first => {
            // Try to parse as "N a"
            let cap_n: u64 = match first.parse() {
                Ok(v) => v,
                Err(_) => return "para shor: usage: para shor [N a | loop [N]]\n".into(),
            };
            let a: u64 = match parts.next().unwrap_or("").trim().parse() {
                Ok(v) => v,
                Err(_) => return "para shor: expected 'para shor <N> <a>'\n".into(),
            };
            let n = (u64::BITS - cap_n.saturating_sub(1).leading_zeros()) as usize;
            let n = n.max(1);
            let r = run_shor(n, a, cap_n);
            let label = format!("N={}", cap_n);
            let mut s = String::new();
            s += &shor_table_header();
            s += &shor_table_row(&label, &r);
            s += &shor_table_footer();
            s
        }
    }
}
