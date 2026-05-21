// ParaASM VM — Belnap FOUR paraconsistent machine
// no_std; EMIT → serial output; READ → N (no stdin in kernel)

#![allow(dead_code)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use alloc::collections::BTreeMap;

// ── Belnap FOUR ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum B4 { N, T, F, B }

impl B4 {
    // Information-order join: N < T,F < B; T∨F = B
    pub fn join(self, other: B4) -> B4 {
        match (self, other) {
            (B4::B, _) | (_, B4::B)         => B4::B,
            (B4::T, B4::F) | (B4::F, B4::T) => B4::B,
            (B4::T, _) | (_, B4::T)         => B4::T,
            (B4::F, _) | (_, B4::F)         => B4::F,
            _                               => B4::N,
        }
    }

    pub fn name(self) -> &'static str {
        match self { B4::N => "N", B4::T => "T", B4::F => "F", B4::B => "B" }
    }
}

// ── ParaRegister ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct ParaRegister {
    pub belief: B4,
    pub is_fixed: bool,
    pub paradox_count: u64,
}

impl ParaRegister {
    pub fn new() -> Self {
        Self { belief: B4::N, is_fixed: false, paradox_count: 0 }
    }

    // ENGAGR: force Both; ignore is_fixed (Case A of IFIX stability)
    pub fn engage(&mut self) {
        let was_b = self.belief == B4::B;
        self.belief = B4::B;
        if !was_b { self.paradox_count += 1; }
    }

    // IFIX: collapse to T, mark fixed
    pub fn fix(&mut self) {
        self.belief = B4::T;
        self.is_fixed = true;
    }

    pub fn is_active(&self) -> bool {
        self.belief != B4::N || self.is_fixed
    }
}

// ── ISA ───────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Op {
    // Core
    Engagr, Fsplit, Ffuse, Ifix, Move, Clear,
    // Control flow
    Jmp, Jb, Jt, Jf, Jn, Call, Ret, Halt,
    // Stack
    Push, Pop,
    // I/O
    Emit, Read,
}

impl Op {
    pub fn from_str(s: &str) -> Option<Op> {
        match s {
            "ENGAGR" => Some(Op::Engagr), "FSPLIT" => Some(Op::Fsplit),
            "FFUSE"  => Some(Op::Ffuse),  "IFIX"   => Some(Op::Ifix),
            "MOVE"   => Some(Op::Move),   "CLEAR"  => Some(Op::Clear),
            "JMP"    => Some(Op::Jmp),    "JB"     => Some(Op::Jb),
            "JT"     => Some(Op::Jt),     "JF"     => Some(Op::Jf),
            "JN"     => Some(Op::Jn),     "CALL"   => Some(Op::Call),
            "RET"    => Some(Op::Ret),    "HALT"   => Some(Op::Halt),
            "PUSH"   => Some(Op::Push),   "POP"    => Some(Op::Pop),
            "EMIT"   => Some(Op::Emit),   "READ"   => Some(Op::Read),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Arg {
    Reg(usize),
    Label(String),
}

#[derive(Clone, Debug)]
pub struct Instr {
    pub op: Op,
    pub args: Vec<Arg>,
}

// ── Assembler ─────────────────────────────────────────────────────────────────

pub fn assemble(text: &str) -> (Vec<Instr>, BTreeMap<String, usize>) {
    let mut instrs: Vec<Instr> = Vec::new();
    let mut labels: BTreeMap<String, usize> = BTreeMap::new();

    for raw in text.lines() {
        // strip comment
        let line = raw.find(';').map(|p| &raw[..p]).unwrap_or(raw).trim();
        if line.is_empty() { continue; }

        // label prefix (.name: or name:)
        let (label, rest) = split_label(line);
        if let Some(name) = label { labels.insert(name, instrs.len()); }
        let rest = rest.trim();
        if rest.is_empty() { continue; }

        let mut toks = rest.split_whitespace();
        let op_str = match toks.next() { Some(s) => s, None => continue };
        let op = match Op::from_str(&op_str.to_uppercase()) {
            Some(o) => o,
            None => continue,
        };
        let args: Vec<Arg> = toks.map(parse_arg).collect();
        instrs.push(Instr { op, args });
    }

    (instrs, labels)
}

fn split_label(line: &str) -> (Option<String>, &str) {
    let b = line.as_bytes();
    let start = if b.first() == Some(&b'.') { 1 } else { 0 };
    let mut end = start;
    while end < b.len() && (b[end].is_ascii_alphanumeric() || b[end] == b'_') { end += 1; }
    if end < b.len() && b[end] == b':' {
        let name = String::from(line[..end].trim_start_matches('.'));
        (Some(name), &line[end + 1..])
    } else {
        (None, line)
    }
}

fn parse_arg(token: &str) -> Arg {
    let s = token.trim_start_matches('%').trim_start_matches('r');
    if let Ok(n) = s.parse::<usize>() { return Arg::Reg(n); }
    Arg::Label(String::from(token.trim_start_matches('.')))
}

// ── ParaVM ────────────────────────────────────────────────────────────────────

const NUM_REGS: usize = 16;

pub struct ParaVM {
    pub regs: [ParaRegister; NUM_REGS],
    pub program: Vec<Instr>,
    pub labels: BTreeMap<String, usize>,
    pub pc: usize,
    pub call_stack: Vec<usize>,
    pub data_stack: Vec<B4>,
    pub halted: bool,
    pub steps: u64,
    pub emit_count: u64,
}

impl ParaVM {
    pub fn new() -> Self {
        Self {
            regs: core::array::from_fn(|_| ParaRegister::new()),
            program: Vec::new(),
            labels: BTreeMap::new(),
            pc: 0,
            call_stack: Vec::new(),
            data_stack: Vec::new(),
            halted: false,
            steps: 0,
            emit_count: 0,
        }
    }

    pub fn load(&mut self, text: &str) {
        let (prog, labels) = assemble(text);
        self.program = prog;
        self.labels = labels;
        self.pc = 0;
        self.halted = false;
        self.steps = 0;
        self.call_stack.clear();
        self.data_stack.clear();
        self.emit_count = 0;
    }

    pub fn reset_state(&mut self) {
        for r in &mut self.regs { *r = ParaRegister::new(); }
        self.pc = 0;
        self.halted = false;
        self.steps = 0;
        self.call_stack.clear();
        self.data_stack.clear();
        self.emit_count = 0;
    }

    fn resolve(&self, name: &str) -> Option<usize> {
        self.labels.get(name).copied()
    }

    fn reg(args: &[Arg], i: usize) -> Option<usize> {
        match args.get(i) { Some(Arg::Reg(n)) => Some(*n), _ => None }
    }

    fn lbl<'a>(args: &'a [Arg], i: usize) -> Option<&'a str> {
        match args.get(i) { Some(Arg::Label(s)) => Some(s.as_str()), _ => None }
    }

    pub fn step(&mut self) -> bool {
        if self.halted || self.program.is_empty() { return false; }
        if self.pc >= self.program.len() { self.pc = 0; } // circular wrap
        let instr = self.program[self.pc].clone();
        self.pc += 1;
        self.steps += 1;
        self.execute(&instr);
        true
    }

    pub fn run(&mut self, n: u64) {
        for _ in 0..n { if !self.step() { break; } }
    }

    fn execute(&mut self, instr: &Instr) {
        use Op::*;
        match instr.op {
            Engagr => {
                if let Some(r) = Self::reg(&instr.args, 0) {
                    if r < NUM_REGS { self.regs[r].engage(); }
                }
            }
            Fsplit => {
                // δ: copy src into d1 and d2 (inherits src's is_fixed and belief)
                if let (Some(src), Some(d1), Some(d2)) = (
                    Self::reg(&instr.args, 0),
                    Self::reg(&instr.args, 1),
                    Self::reg(&instr.args, 2),
                ) {
                    if src < NUM_REGS && d1 < NUM_REGS && d2 < NUM_REGS {
                        let belief  = self.regs[src].belief;
                        let paradox = self.regs[src].paradox_count;
                        let fixed   = self.regs[src].is_fixed;
                        let bump    = if belief == B4::B { 1 } else { 0 };
                        self.regs[d1].belief = belief;
                        self.regs[d1].is_fixed = fixed;
                        self.regs[d1].paradox_count = paradox + bump;
                        self.regs[d2].belief = belief;
                        self.regs[d2].is_fixed = fixed;
                        self.regs[d2].paradox_count = paradox + bump;
                    }
                }
            }
            Ffuse => {
                // μ: Belnap join s1 ∨ s2 → dst
                if let (Some(s1), Some(s2), Some(dst)) = (
                    Self::reg(&instr.args, 0),
                    Self::reg(&instr.args, 1),
                    Self::reg(&instr.args, 2),
                ) {
                    if s1 < NUM_REGS && s2 < NUM_REGS && dst < NUM_REGS {
                        let b1 = self.regs[s1].belief;
                        let b2 = self.regs[s2].belief;
                        let joined = b1.join(b2);
                        let p = self.regs[s1].paradox_count + self.regs[s2].paradox_count;
                        self.regs[dst].belief = joined;
                        self.regs[dst].is_fixed = false;
                        self.regs[dst].paradox_count = p + if joined == B4::B { 1 } else { 0 };
                    }
                }
            }
            Ifix => {
                if let Some(r) = Self::reg(&instr.args, 0) {
                    if r < NUM_REGS { self.regs[r].fix(); }
                }
            }
            Move => {
                if let (Some(src), Some(dst)) = (
                    Self::reg(&instr.args, 0), Self::reg(&instr.args, 1),
                ) {
                    if src < NUM_REGS && dst < NUM_REGS {
                        let v = self.regs[src].clone();
                        self.regs[dst] = v;
                    }
                }
            }
            Clear => {
                if let Some(r) = Self::reg(&instr.args, 0) {
                    if r < NUM_REGS { self.regs[r] = ParaRegister::new(); }
                }
            }
            Jmp => {
                if let Some(lbl) = Self::lbl(&instr.args, 0) {
                    if let Some(pc) = self.resolve(lbl) { self.pc = pc; }
                }
            }
            Jb => self.cond_jump(&instr.args, B4::B),
            Jt => self.cond_jump(&instr.args, B4::T),
            Jf => self.cond_jump(&instr.args, B4::F),
            Jn => self.cond_jump(&instr.args, B4::N),
            Call => {
                if let Some(lbl) = Self::lbl(&instr.args, 0) {
                    if let Some(pc) = self.resolve(lbl) {
                        self.call_stack.push(self.pc);
                        self.pc = pc;
                    }
                }
            }
            Ret => {
                if let Some(ret) = self.call_stack.pop() { self.pc = ret; }
            }
            Halt => { self.halted = true; }
            Push => {
                if let Some(r) = Self::reg(&instr.args, 0) {
                    if r < NUM_REGS { self.data_stack.push(self.regs[r].belief); }
                }
            }
            Pop => {
                if let Some(r) = Self::reg(&instr.args, 0) {
                    if r < NUM_REGS {
                        if let Some(b) = self.data_stack.pop() { self.regs[r].belief = b; }
                    }
                }
            }
            Emit => {
                if let Some(r) = Self::reg(&instr.args, 0) {
                    if r < NUM_REGS {
                        let reg = &self.regs[r];
                        let tag = if reg.is_fixed { " [FIXED]" } else { "" };
                        let msg = format!("EMIT %r{} = {}{}\r\n", r, reg.belief.name(), tag);
                        crate::serial::write_str(&msg);
                        self.emit_count += 1;
                    }
                }
            }
            Read => {
                // No stdin in kernel; seed N (Neither = no information)
                if let Some(r) = Self::reg(&instr.args, 0) {
                    if r < NUM_REGS { self.regs[r].belief = B4::N; }
                }
            }
        }
    }

    fn cond_jump(&mut self, args: &[Arg], target: B4) {
        if let (Some(r), Some(lbl)) = (Self::reg(args, 0), Self::lbl(args, 1)) {
            if r < NUM_REGS && self.regs[r].belief == target {
                if let Some(pc) = self.resolve(lbl) { self.pc = pc; }
            }
        }
    }

    pub fn active_regs(&self) -> Vec<(usize, B4, u64, bool)> {
        self.regs.iter().enumerate()
            .filter(|(_, r)| r.is_active())
            .map(|(i, r)| (i, r.belief, r.paradox_count, r.is_fixed))
            .collect()
    }

    pub fn total_paradoxes(&self) -> u64 {
        self.regs.iter().map(|r| r.paradox_count).sum()
    }

    pub fn format_snapshot(&self) -> String {
        let mut s = format!(
            "ParaVM  pc={}  steps={}  halted={}  emits={}\n",
            self.pc, self.steps, self.halted, self.emit_count
        );
        let active = self.active_regs();
        if active.is_empty() {
            s += "  (all registers N)\n";
        } else {
            for (i, belief, paradoxes, fixed) in &active {
                let tag = if *fixed { " [FIXED]" } else { "" };
                s += &format!("  %r{:<2} = {}{:<8}  paradoxes={}\n",
                    i, belief.name(), tag, paradoxes);
            }
        }
        s += &format!("  total_paradoxes={}  labels={}", self.total_paradoxes(), self.labels.len());
        s
    }
}
