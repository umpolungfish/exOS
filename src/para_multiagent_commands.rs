// para_multiagent_commands.rs — Multi-Agent Belnap Protocol for exOS
//
// Usage:
//   para multiagent          full suite (checks + network state)
//   para multiagent init     initial state (emerald bootstrap)
//   para multiagent step     after 4 steps
//
// Lean reference: MillenniumAnkh/Imscribing/Paraconsistent/MultiAgentBelnap.lean

#![allow(dead_code)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use crate::para_vm::{B4, ParaKernel};


// ── Multi-agent state ─────────────────────────────────────────────────────────

#[derive(Clone)]
struct AgentState {
    r0: B4,
    r1: B4,
    r2: B4,
}

#[derive(Clone)]
struct MultiState {
    agents:   Vec<AgentState>,
    channels: Vec<B4>,
}

fn init_multi(n: usize) -> MultiState {
    let agents = (0..n).map(|_| AgentState { r0: B4::B, r1: B4::B, r2: B4::B }).collect();
    let channels = (0..n.saturating_sub(1)).map(|_| B4::B).collect();
    MultiState { agents, channels }
}

fn agent_step(a: &AgentState) -> AgentState {
    let mut k = ParaKernel::initial();
    k.r0 = a.r0; k.r1 = a.r1; k.r2 = a.r2;
    k = k.run(1);
    AgentState { r0: k.r0, r1: k.r1, r2: k.r2 }
}

fn multi_step(state: &MultiState) -> MultiState {
    let new_agents: Vec<AgentState> = state.agents.iter().map(agent_step).collect();
    let new_channels: Vec<B4> = (0..new_agents.len().saturating_sub(1))
        .map(|i| new_agents[i].r0.join(new_agents[i + 1].r0))
        .collect();
    MultiState { agents: new_agents, channels: new_channels }
}

fn multi_run(mut state: MultiState, steps: usize) -> MultiState {
    for _ in 0..steps { state = multi_step(&state); }
    state
}


// ── Theorem checks ────────────────────────────────────────────────────────────

fn multi_allb_init(n: usize) -> bool {
    let s = init_multi(n);
    s.agents.iter().all(|a| a.r0 == B4::B && a.r1 == B4::B && a.r2 == B4::B)
    && s.channels.iter().all(|&c| c == B4::B)
}

fn multi_allb_preserved(n: usize, steps: usize) -> bool {
    let s = multi_run(init_multi(n), steps);
    s.agents.iter().all(|a| a.r0 == B4::B && a.r1 == B4::B && a.r2 == B4::B)
    && s.channels.iter().all(|&c| c == B4::B)
}

fn channel_join_stable(n: usize) -> bool {
    B4::B.join(B4::B) == B4::B && n > 1
}

fn multi_agent_is_o_inf() -> bool {
    let phi_c = B4::B.bnot() == B4::B && B4::B.designated();
    phi_c && ParaKernel::frobenius_invariant(B4::B)
}


// ── Display blocks ────────────────────────────────────────────────────────────

fn fmt_agent(i: usize, a: &AgentState, ch: Option<B4>) -> String {
    let ch_str = match ch {
        Some(c) => format!("  ─[{}]─▶", c.name()),
        None    => String::from("          "),
    };
    format!("  │    agent[{}]: r0={} r1={} r2={}{}  │\n",
        i, a.r0.name(), a.r1.name(), a.r2.name(), ch_str)
}

fn checks_block(n: usize, steps: usize) -> String {
    let mark = |ok: bool| if ok { "✓" } else { "✗" };
    let mut s = String::new();
    s += "  ┌──────────────────────────────────────────────────────────────┐\n";
    s += "  │  MULTI-AGENT CHECKS (MultiAgentBelnap.lean)                 │\n";
    s += "  ├──────────────────────────────────────────────────────────────┤\n";
    s += &format!("  │  {}  multi_allB_init: all agents start all-B              │\n",
        mark(multi_allb_init(n)));
    s += &format!("  │  {}  multi_allB_preserved: stays all-B ({} steps)         │\n",
        mark(multi_allb_preserved(n, steps)), steps);
    s += &format!("  │  {}  channel_join_stable: join(B,B)=B always              │\n",
        mark(channel_join_stable(n)));
    s += &format!("  │  {}  multi_agent_is_O_inf: Phi_c ∧ P_pm_sym               │\n",
        mark(multi_agent_is_o_inf()));
    s += "  ├──────────────────────────────────────────────────────────────┤\n";
    s += &format!("  │  Network: {} agents · {} channels · all Belnap FOUR          │\n",
        n, n.saturating_sub(1));
    s += "  │  Channel belief = join(agent[i].r0, agent[i+1].r0)          │\n";
    s += "  │  Emerald bootstrap: all-B initial                           │\n";
    s += "  └──────────────────────────────────────────────────────────────┘\n";
    s
}

fn state_block(state: &MultiState, label: &str) -> String {
    let mut s = String::new();
    s += &format!("  ┌──────────────────────────────────────────────────────────────┐\n");
    s += &format!("  │  {}  │\n", label);
    s += "  ├──────────────────────────────────────────────────────────────┤\n";
    for (i, a) in state.agents.iter().enumerate() {
        let ch = state.channels.get(i).copied();
        s += &fmt_agent(i, a, ch);
    }
    s += "  └──────────────────────────────────────────────────────────────┘\n";
    s
}

fn full_suite(n: usize, steps: usize) -> String {
    let mut s = String::new();
    s += "\n";
    s += "╔══════════════════════════════════════════════════════════════════╗\n";
    s += "║  MULTI-AGENT BELNAP PROTOCOL  (MultiAgentBelnap.lean)          ║\n";
    s += "╚══════════════════════════════════════════════════════════════════╝\n\n";
    s += &checks_block(n, steps);
    s += "\n";
    s += &state_block(&init_multi(n), "Initial state (emerald bootstrap, all-B)              ");
    s += "\n";
    let after = multi_run(init_multi(n), steps);
    s += &state_block(&after, &format!("After {} steps                                             ", steps));
    s += "\n";
    s
}


// ── Shell entry point ─────────────────────────────────────────────────────────

pub fn handle(args: &str) -> String {
    let n = 4usize;
    let steps = 4usize;
    match args.trim() {
        ""     => full_suite(n, steps),
        "init" => { let mut s = String::from("\n"); s += &state_block(&init_multi(n), "Initial state (emerald bootstrap, all-B)              "); s }
        "step" => { let mut s = String::from("\n"); s += &state_block(&multi_run(init_multi(n), steps), &format!("After {} steps                                             ", steps)); s }
        other  => format!("para multiagent: unknown '{}'. Try: init | step\n", other),
    }
}
