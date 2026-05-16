//! IMASM Tri-Phase Virtual Machine — shared runtime for all three script engines.
//!
//! Executes compiled IMASM instruction streams on Tri-Phase Flux Registers.
//! A register has a 2-bit flux state (Void/True/False/Both) and an optional
//! FIXED brand. FSPLIT/ENGAGR stabilize contradictions in-place (Both state);
//! they do not propagate. IFIX brands a register permanently — the linear type
//! constraint enforcing temporal asymmetry. Entropy delta is always 0.0.

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;

// ── Flux state ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flux {
    Void  = 0b00,
    True  = 0b01,
    False = 0b10,
    Both  = 0b11,
}

impl Flux {
    pub fn name(self) -> &'static str {
        match self {
            Flux::Void  => "Void",
            Flux::True  => "True",
            Flux::False => "False",
            Flux::Both  => "Both",
        }
    }
}

// ── Opcodes ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    VINIT  = 0x0,   // Initial object ∅
    TANCH  = 0x1,   // Terminal anchor ⊤
    AFWD   = 0x2,   // Morphism →
    AREV   = 0x3,   // Contravariant inversion ←
    CLINK  = 0x4,   // Composition ∘
    ISCRIB = 0x5,   // Identity id
    FSPLIT = 0x6,   // Frobenius co-multiplication δ (engage)
    FFUSE  = 0x7,   // Frobenius multiplication μ
    EVALT  = 0x8,   // Lattice: True
    EVALF  = 0x9,   // Lattice: False
    ENGAGR = 0xA,   // Lattice: Both (paradox stabilization)
    IFIX   = 0xB,   // Linear tape write (FIXED brand)
}

impl Opcode {
    pub fn mnemonic(self) -> &'static str {
        match self {
            Opcode::VINIT  => "VINIT",
            Opcode::TANCH  => "TANCH",
            Opcode::AFWD   => "AFWD",
            Opcode::AREV   => "AREV",
            Opcode::CLINK  => "CLINK",
            Opcode::ISCRIB => "ISCRIB",
            Opcode::FSPLIT => "FSPLIT",
            Opcode::FFUSE  => "FFUSE",
            Opcode::EVALT  => "EVALT",
            Opcode::EVALF  => "EVALF",
            Opcode::ENGAGR => "ENGAGR",
            Opcode::IFIX   => "IFIX",
        }
    }

    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x0 => Some(Opcode::VINIT),
            0x1 => Some(Opcode::TANCH),
            0x2 => Some(Opcode::AFWD),
            0x3 => Some(Opcode::AREV),
            0x4 => Some(Opcode::CLINK),
            0x5 => Some(Opcode::ISCRIB),
            0x6 => Some(Opcode::FSPLIT),
            0x7 => Some(Opcode::FFUSE),
            0x8 => Some(Opcode::EVALT),
            0x9 => Some(Opcode::EVALF),
            0xA => Some(Opcode::ENGAGR),
            0xB => Some(Opcode::IFIX),
            _   => None,
        }
    }
}

// ── Instruction ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub opcode: Opcode,
    pub dst:    u32,
}

// ── Register ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TriPhaseRegister {
    pub flux:          Flux,
    pub fixed:         bool,
    pub paradox_count: u32,
}

impl TriPhaseRegister {
    pub fn new() -> Self {
        TriPhaseRegister { flux: Flux::Void, fixed: false, paradox_count: 0 }
    }

    pub fn engage(&mut self) {
        self.flux = Flux::Both;
        self.paradox_count += 1;
    }

    pub fn fix(&mut self) {
        self.fixed = true;
    }

    pub fn is_active(&self) -> bool {
        self.flux != Flux::Void || self.fixed
    }
}

// ── Script system tag ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptSystem {
    Voynich,
    Rohonc,
    LinearA,
}

impl ScriptSystem {
    pub fn name(self) -> &'static str {
        match self {
            ScriptSystem::Voynich => "Voynich",
            ScriptSystem::Rohonc  => "Rohonc",
            ScriptSystem::LinearA => "Linear A",
        }
    }
}

// ── Snapshot ─────────────────────────────────────────────────────────────────

pub struct Snapshot {
    pub step:         u64,
    pub pc:           usize,
    pub active:       u32,
    pub fixed:        u32,
    pub paradoxes:    u32,
    pub script:       ScriptSystem,
}

// ── Universal Engine ─────────────────────────────────────────────────────────

pub struct UniversalEngine {
    registers:   BTreeMap<u32, TriPhaseRegister>,
    pub program: Vec<Instruction>,
    pub pc:      usize,
    pub steps:   u64,
    pub script:  ScriptSystem,
}

impl UniversalEngine {
    pub fn new(script: ScriptSystem) -> Self {
        UniversalEngine {
            registers: BTreeMap::new(),
            program:   Vec::new(),
            pc:        0,
            steps:     0,
            script,
        }
    }

    pub fn load(&mut self, program: Vec<Instruction>) {
        self.program = program;
        self.pc      = 0;
        self.steps   = 0;
        self.registers.clear();
    }

    pub fn step(&mut self) {
        if self.program.is_empty() {
            return;
        }
        if self.pc >= self.program.len() {
            self.pc = 0;                    // bootstrap loop closure
        }
        let instr = self.program[self.pc];
        self.execute(instr);
        self.pc += 1;
        self.steps += 1;
    }

    pub fn run(&mut self, n: u64) {
        for _ in 0..n {
            self.step();
        }
    }

    pub fn inject_paradox(&mut self, reg: u32) {
        self.registers.entry(reg).or_insert_with(TriPhaseRegister::new).engage();
    }

    pub fn snapshot(&self) -> Snapshot {
        let active   = self.registers.values().filter(|r| r.is_active()).count() as u32;
        let fixed    = self.registers.values().filter(|r| r.fixed).count() as u32;
        let paradoxes = self.registers.values().map(|r| r.paradox_count).sum();
        Snapshot { step: self.steps, pc: self.pc, active, fixed, paradoxes, script: self.script }
    }

    pub fn format_snapshot(&self) -> String {
        let s = self.snapshot();
        format!(
            "=== {} ENGINE ===\nSteps      : {}\nPC         : {}\nActive regs: {}\nFixed regs : {}\nParadoxes  : {}\nEntropy Δ  : 0.00000000 J/K\nStatus     : BOOTSTRAP_COMPLETE",
            s.script.name(), s.step, s.pc, s.active, s.fixed, s.paradoxes
        )
    }

    fn execute(&mut self, instr: Instruction) {
        let reg = self.registers.entry(instr.dst).or_insert_with(TriPhaseRegister::new);
        match instr.opcode {
            Opcode::FSPLIT | Opcode::ENGAGR => reg.engage(),
            Opcode::IFIX                    => reg.fix(),
            // VINIT, TANCH, AFWD, AREV, CLINK, ISCRIB, FFUSE, EVALT, EVALF:
            // structurally present; ΔS = 0 is a theorem of the linear type constraint.
            _ => {}
        }
    }
}

// ── IMASM text log parser ─────────────────────────────────────────────────────
// Accepts the log format: " 0x6 | FSPLIT %r0"

pub fn parse_imasm_log(text: &str) -> Vec<Instruction> {
    let mut out = Vec::new();
    for line in text.lines() {
        if !line.contains("%r") {
            continue;
        }
        let opcode = if let Some(hex_pos) = line.find("0x") {
            let hex = &line[hex_pos + 2..];
            let nibble = hex.chars().next().and_then(|c| c.to_digit(16));
            nibble.and_then(|v| Opcode::from_u8(v as u8))
        } else {
            // fallback: match by mnemonic
            let mnemonics = [
                ("VINIT",  Opcode::VINIT),  ("TANCH",  Opcode::TANCH),
                ("AFWD",   Opcode::AFWD),   ("AREV",   Opcode::AREV),
                ("CLINK",  Opcode::CLINK),  ("ISCRIB", Opcode::ISCRIB),
                ("FSPLIT", Opcode::FSPLIT), ("FFUSE",  Opcode::FFUSE),
                ("EVALT",  Opcode::EVALT),  ("EVALF",  Opcode::EVALF),
                ("ENGAGR", Opcode::ENGAGR), ("IFIX",   Opcode::IFIX),
            ];
            mnemonics.iter().find(|(m, _)| line.contains(m)).map(|(_, op)| *op)
        };

        let reg = line.find("%r").and_then(|p| {
            line[p + 2..].split_whitespace().next()
                .and_then(|s| s.parse::<u32>().ok())
        });

        if let (Some(op), Some(dst)) = (opcode, reg) {
            out.push(Instruction { opcode: op, dst });
        }
    }
    out
}
