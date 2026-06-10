// Self-Verifying WASM Runtime — Belnap-tagged WebAssembly execution.
// Matches SelfVerifyingWASM.lean exactly: checkpoint/verify/assert_invariant
// protocol, frobTagBin = meet in approximation order.
// no_std; all output via serial.

#![allow(dead_code)]

extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;

use crate::para_vm::B4;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WasmType { I32, I64 }

#[derive(Clone, Copy, Debug)]
pub struct WasmValue { pub ty: WasmType, pub val: u64 }

#[derive(Clone, Debug)]
pub struct TaggedValue { pub value: WasmValue, pub tag: B4 }

impl TaggedValue {
    pub fn designated(&self) -> bool {
        matches!(self.tag, B4::T | B4::B)
    }
}

// ── frobTagBin = meet in approximation order ──────────────────────────────────
// N < T, F < B; meet gives greatest lower bound.
// frobenius_mu_delta_id_tag: meet(t, B) = t  ∀ t  (B is top → identity)

pub fn frob_tag_bin(t1: B4, t2: B4) -> B4 {
    match (t1, t2) {
        (B4::N, _) | (_, B4::N)         => B4::N,
        (B4::B, x) | (x, B4::B)         => x,
        (B4::T, B4::F) | (B4::F, B4::T) => B4::N,
        (B4::T, B4::T)                   => B4::T,
        (B4::F, B4::F)                   => B4::F,
    }
}

// ── Instructions ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum WasmInstr {
    I32Const(u64, B4),  // tag: T for well-parsed input, N for malformed
    I64Const(u64, B4),
    Drop,
    Nop,
    Unreachable,
    Checkpoint,
    Verify,
    AssertInvariant,
    AttestClean,        // B if stack empty and invariant=B; N otherwise
}

// ── WasmState ─────────────────────────────────────────────────────────────────

pub struct WasmState {
    pub stack:                Vec<TaggedValue>,
    pub ip:                   usize,
    pub frob_snapshot:        Vec<TaggedValue>,
    pub frob_invariant_holds: B4,
    pub verified_steps:       u64,
    pub total_steps:          u64,
    // O(1) verify fast-path: count of non-designated items currently on stack.
    // Maintained incrementally on push and drop.
    pub non_designated_count: usize,
    // Result of last AttestClean: B if clean context boundary, N otherwise.
    pub attest_result:        B4,
}

impl WasmState {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            ip: 0,
            frob_snapshot: Vec::new(),
            frob_invariant_holds: B4::N,
            verified_steps: 0,
            total_steps: 0,
            non_designated_count: 0,
            attest_result: B4::N,
        }
    }
}

// ── exec_one ──────────────────────────────────────────────────────────────────

pub fn exec_one(s: &mut WasmState, instr: &WasmInstr) {
    s.ip += 1;
    s.total_steps += 1;

    match instr {
        WasmInstr::I32Const(n, tag) => {
            if !matches!(tag, B4::T | B4::B) { s.non_designated_count += 1; }
            s.stack.insert(0, TaggedValue {
                value: WasmValue { ty: WasmType::I32, val: *n },
                tag: *tag,
            });
        }
        WasmInstr::I64Const(n, tag) => {
            if !matches!(tag, B4::T | B4::B) { s.non_designated_count += 1; }
            s.stack.insert(0, TaggedValue {
                value: WasmValue { ty: WasmType::I64, val: *n },
                tag: *tag,
            });
        }
        WasmInstr::Drop => {
            if !s.stack.is_empty() {
                let dropped = s.stack.remove(0);
                if !dropped.designated() { s.non_designated_count = s.non_designated_count.saturating_sub(1); }
            }
        }
        WasmInstr::Nop => {}
        WasmInstr::Unreachable => {
            s.frob_invariant_holds = B4::F;
        }
        WasmInstr::Checkpoint => {
            s.frob_snapshot = s.stack.clone();
        }
        WasmInstr::Verify => {
            // O(1) fast-path: if no non-designated items tracked, skip full scan.
            let all_designated = if s.non_designated_count == 0 {
                true
            } else {
                s.stack.iter().all(|tv| tv.designated())
            };
            if all_designated {
                s.frob_invariant_holds = B4::B;
                s.verified_steps += 1;
            } else {
                s.frob_invariant_holds = B4::F;
            }
        }
        WasmInstr::AssertInvariant => {
            let rhs = if s.frob_invariant_holds == B4::F { B4::F } else { B4::B };
            s.frob_invariant_holds = frob_tag_bin(s.frob_invariant_holds, rhs);
        }
        WasmInstr::AttestClean => {
            // Context-switch boundary check: clean iff stack empty and invariant=B.
            s.attest_result = attest_clean_state(s);
        }
    }
}

// ── Attestation ───────────────────────────────────────────────────────────────
// attest_clean_state: returns B4::B iff stack is empty and invariant is B.
// Use at context-switch boundaries: a clean-slate guarantee before tier promotion.

pub fn attest_clean_state(s: &WasmState) -> B4 {
    if s.stack.is_empty() && s.frob_invariant_holds == B4::B {
        B4::B
    } else {
        B4::N
    }
}

// ── Runtime ───────────────────────────────────────────────────────────────────

pub struct WasmRuntime {
    pub state:   WasmState,
    pub program: Vec<WasmInstr>,
}

impl WasmRuntime {
    pub fn new() -> Self {
        Self { state: WasmState::new(), program: Vec::new() }
    }

    pub fn load(&mut self, program: Vec<WasmInstr>) {
        self.program = program;
        self.state = WasmState::new();
    }

    pub fn step(&mut self) -> bool {
        if self.state.ip >= self.program.len() { return false; }
        let instr = self.program[self.state.ip].clone();
        exec_one(&mut self.state, &instr);
        true
    }

    pub fn run(&mut self) {
        while self.step() {}
    }

    pub fn format_snapshot(&self) -> String {
        let s = &self.state;
        let inv = s.frob_invariant_holds.name();
        let mut out = format!(
            "ip={}  steps={}  verified={}  frob_invariant={}\n",
            s.ip, s.total_steps, s.verified_steps, inv
        );
        out += &format!("stack ({} items):\n", s.stack.len());
        if s.stack.is_empty() {
            out += "  (empty)\n";
        } else {
            for tv in &s.stack {
                let ty = match tv.value.ty { WasmType::I32 => "i32", WasmType::I64 => "i64" };
                out += &format!("  {}({}) [{}]\n", ty, tv.value.val, tv.tag.name());
            }
        }
        out += &format!("snapshot_depth={}", s.frob_snapshot.len());
        out
    }

    pub fn format_attest(&self) -> String {
        let result = attest_clean_state(&self.state);
        format!(
            "attest_clean: {}  (stack={} items, frob_invariant={})",
            result.name(), self.state.stack.len(),
            self.state.frob_invariant_holds.name()
        )
    }
}

// ── Demo: frobenius_empty_stack (matches Lean theorem) ───────────────────────

pub fn demo_frobenius_empty_stack(n: u64) -> String {
    let mut rt = WasmRuntime::new();
    rt.load(alloc::vec![
        WasmInstr::Checkpoint,
        WasmInstr::I32Const(n, B4::T),
        WasmInstr::Verify,
    ]);
    rt.run();
    let inv = rt.state.frob_invariant_holds;
    format!(
        "checkpoint + i32_const {} + verify → frob_invariant={}\nfrobenius_empty_stack: {}",
        n, inv.name(),
        if inv == B4::B { "OK" } else { "FAIL" }
    )
}
