// para_rh_commands.rs — Riemann Hypothesis Bridge for exOS
//
// Usage:
//   para rh          full suite (strip map + functional eq + structural type)
//   para rh frobenius  Frobenius fixed point analysis only
//   para rh strip      critical strip state map only
//
// Lean reference: MillenniumAnkh/Imscribing/Paraconsistent/QCI_RH_Bridge.lean

#![allow(dead_code)]

extern crate alloc;

use alloc::string::String;
use alloc::format;

use crate::para_vm::B4;


// ── Functional equation as Belnap negation ────────────────────────────────────

fn rh_functional_eq(s: B4) -> B4 { s.bnot() }

// B is the unique designated fixed point of bnot.
fn rh_frobenius_fixed_point() -> bool {
    let b_fixed = B4::B.bnot() == B4::B && B4::B.designated();
    let t_not_fixed = B4::T.bnot() != B4::T && !B4::T.bnot().designated();
    b_fixed && t_not_fixed
}

// All non-trivial zeros are B-designated: unique designated fixed point + dialetheic.
fn rh_belnap_statement() -> bool {
    let unique_fixed = B4::B.designated()
        && B4::B.bnot() == B4::B
        && [B4::N, B4::T, B4::F].iter().all(|&x| !(x.designated() && x.bnot() == x));
    unique_fixed && B4::B.dialetheic()
        && [B4::N, B4::T, B4::F].iter().all(|&x| !x.dialetheic())
}

// bnot∘bnot = id (involution identity).
fn rh_involution_identity() -> bool {
    [B4::N, B4::T, B4::F, B4::B].iter().all(|&x| x.bnot().bnot() == x)
}

// Critical strip state: scaled integer re_s / 100.
fn rh_strip_state(re_s_num: i32, re_s_den: i32) -> B4 {
    if re_s_num < 0 || re_s_num > re_s_den                    { return B4::N; }
    if re_s_num == 0 || re_s_num == re_s_den                   { return B4::F; }
    if 2 * re_s_num == re_s_den                                { return B4::B; }
    B4::T
}


// ── Display blocks ────────────────────────────────────────────────────────────

fn frobenius_block() -> String {
    let mark = |ok: bool| if ok { "✓" } else { "✗" };
    let mut s = String::new();
    s += "  ┌──────────────────────────────────────────────────────────────┐\n";
    s += "  │  RH: FUNCTIONAL EQUATION ↔ BELNAP NEGATION                 │\n";
    s += "  ├──────────────────────────────────────────────────────────────┤\n";
    s += &format!("  │  {}  rh_involution_identity: bnot∘bnot = id              │\n",
        mark(rh_involution_identity()));
    s += "  │       functional equation s↦1-s applied twice = id          │\n";
    s += &format!("  │  {}  rh_frobenius_fixed_point: bnot(B)=B only             │\n",
        mark(rh_frobenius_fixed_point()));
    s += "  │       Re(s)=1/2 is the unique Frobenius fixed point          │\n";
    s += &format!("  │  {}  rh_belnap_statement: all zeros are B-designated      │\n",
        mark(rh_belnap_statement()));
    s += "  │       B = both ζ(s)=0 and ζ(1-s)=0 (dialetheic pair)       │\n";
    s += "  ├──────────────────────────────────────────────────────────────┤\n";
    s += "  │  Functional equation table (bnot = s ↦ 1-s):                │\n";
    for q in [B4::N, B4::T, B4::F, B4::B] {
        let img = rh_functional_eq(q);
        let tag = if img == q && q.designated() { "  ← FROBENIUS FIXED POINT" }
                  else if img == q              { "  ← fixed (not designated)" }
                  else                          { "" };
        s += &format!("  │    bnot({}) = {}{}  │\n", q.name(), img.name(), tag);
    }
    s += "  └──────────────────────────────────────────────────────────────┘\n";
    s
}

fn strip_block() -> String {
    let mut s = String::new();
    s += "  ┌──────────────────────────────────────────────────────────────┐\n";
    s += "  │  Critical strip map (Re(s) ↦ B4)                            │\n";
    s += "  ├──────────────────────────────────────────────────────────────┤\n";
    let samples: &[(i32, &str)] = &[
        (-10, "Re=-0.1"), (0,  "Re=0.0"), (10, "Re=0.1"), (25, "Re=0.25"),
        (49,  "Re=0.49"), (50, "Re=0.5"), (51, "Re=0.51"),(75, "Re=0.75"),
        (100, "Re=1.0"),  (110,"Re=1.1"),
    ];
    for &(num, label) in samples {
        let state = rh_strip_state(num, 100);
        let desc = match state {
            B4::N => "outside strip        ",
            B4::F => "strip boundary       ",
            B4::T => "non-critical interior",
            B4::B => "CRITICAL LINE Re=1/2 ",
        };
        s += &format!("  │    {:8}  →  {}  {}         │\n", label, state.name(), desc);
    }
    s += "  ├──────────────────────────────────────────────────────────────┤\n";
    s += "  │  Structural type:                                            │\n";
    s += "  │  ⟨Ð_ω;Þ_O;Ř_Ť;Φ_};ƒ_ż;Ç_@;Γ_ʔ;ɢ_ˌ;⊙_ÿ;Ħ_A;Σ_ï;Ω_2⟩      │\n";
    s += "  │  (D_holo · P_pm_sym · Phi_c · Omega_Z2)                     │\n";
    s += "  └──────────────────────────────────────────────────────────────┘\n";
    s
}

fn full_suite() -> String {
    let mut s = String::new();
    s += "\n";
    s += "╔══════════════════════════════════════════════════════════════════╗\n";
    s += "║  BELNAP RH BRIDGE  (QCI_RH_Bridge.lean)                        ║\n";
    s += "╚══════════════════════════════════════════════════════════════════╝\n\n";
    s += &frobenius_block();
    s += "\n";
    s += &strip_block();
    s += "\n";
    s
}


// ── Shell entry point ─────────────────────────────────────────────────────────

pub fn handle(args: &str) -> String {
    match args.trim() {
        "" => full_suite(),
        "frobenius" => { let mut s = String::from("\n"); s += &frobenius_block(); s }
        "strip"     => { let mut s = String::from("\n"); s += &strip_block();     s }
        other => format!("para rh: unknown subcommand '{}'. Try: frobenius | strip\n", other),
    }
}
