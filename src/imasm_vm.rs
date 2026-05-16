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
    EmeraldTablet,
}

impl ScriptSystem {
    pub fn name(self) -> &'static str {
        match self {
            ScriptSystem::Voynich       => "Voynich",
            ScriptSystem::Rohonc        => "Rohonc",
            ScriptSystem::LinearA       => "Linear A",
            ScriptSystem::EmeraldTablet => "Emerald Tablet",
        }
    }
}

// ── Voynich section context ──────────────────────────────────────────────────
//
// Each VMS section is a distinct memory segment type with its own execution
// semantics. The section tag is set by physical folio context (Ð_ω dispatch)
// and changes how the engine interprets FSPLIT, VINIT, TANCH, and CLINK.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoynichSection {
    /// Þ_6 branching network — data segment. Standard execution.
    Botanical,
    /// Þ_O orbital — state register file. Cyclic closure on PC wrap.
    Astronomical,
    /// Þ_K nested containment — heap / runtime state snapshot.
    /// Broken Frobenius: FSPLIT preferentially resets to VINIT rather than
    /// completing to FFUSE. Empirical: p(VINIT|FSPLIT) = 0.622 in this section.
    Biological,
    /// Þ_O orbital, clock-indexed — structurally identical to Astronomical.
    Cosmological,
    /// Þ_6, type declarations — structurally identical to Botanical.
    Pharmaceutical,
    /// Ř_Ť sequential + Ħ_£ one-step memory — instruction memory / microcode.
    Recipes,
}

impl VoynichSection {
    pub fn name(self) -> &'static str {
        match self {
            VoynichSection::Botanical      => "Botanical",
            VoynichSection::Astronomical   => "Astronomical",
            VoynichSection::Biological     => "Biological",
            VoynichSection::Cosmological   => "Cosmological",
            VoynichSection::Pharmaceutical => "Pharmaceutical",
            VoynichSection::Recipes        => "Recipes",
        }
    }

    /// Whether this section has broken Frobenius (FSPLIT→VINIT reset).
    pub fn broken_frobenius(self) -> bool {
        self == VoynichSection::Biological
    }

    /// Whether this section enforces one-step sequential memory (Ħ_£).
    pub fn sequential_memory(self) -> bool {
        self == VoynichSection::Recipes
    }
}

// ── Snapshot ─────────────────────────────────────────────────────────────────

pub struct Snapshot {
    pub step:               u64,
    pub pc:                 usize,
    pub active:             u32,
    pub fixed:              u32,
    pub paradoxes:          u32,
    pub script:             ScriptSystem,
    pub section:            Option<VoynichSection>,
    pub pending_splits:     u32,
    pub cross_segment_refs: u32,
}

// ── Universal Engine ─────────────────────────────────────────────────────────

pub struct UniversalEngine {
    registers:               BTreeMap<u32, TriPhaseRegister>,
    pub program:             Vec<Instruction>,
    pub pc:                  usize,
    pub steps:               u64,
    pub script:              ScriptSystem,
    /// Active VMS section context — determines execution semantics.
    pub section:             Option<VoynichSection>,
    /// Open FSPLIT operations not yet closed by FFUSE.
    /// In biological section, VINIT resets this counter (broken Frobenius).
    pub pending_splits:      u32,
    /// Count of CLINK (cross-segment pointer dereference) operations.
    pub cross_segment_refs:  u32,
}

impl UniversalEngine {
    pub fn new(script: ScriptSystem) -> Self {
        UniversalEngine {
            registers:            BTreeMap::new(),
            program:              Vec::new(),
            pc:                   0,
            steps:                0,
            script,
            section:              None,
            pending_splits:       0,
            cross_segment_refs:   0,
        }
    }

    pub fn new_sectioned(script: ScriptSystem, section: VoynichSection) -> Self {
        let mut e = Self::new(script);
        e.section = Some(section);
        e
    }

    pub fn load(&mut self, program: Vec<Instruction>) {
        self.program             = program;
        self.pc                  = 0;
        self.steps               = 0;
        self.pending_splits      = 0;
        self.cross_segment_refs  = 0;
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
        let active    = self.registers.values().filter(|r| r.is_active()).count() as u32;
        let fixed     = self.registers.values().filter(|r| r.fixed).count() as u32;
        let paradoxes = self.registers.values().map(|r| r.paradox_count).sum();
        Snapshot {
            step: self.steps, pc: self.pc, active, fixed, paradoxes,
            script: self.script, section: self.section,
            pending_splits: self.pending_splits,
            cross_segment_refs: self.cross_segment_refs,
        }
    }

    pub fn format_snapshot(&self) -> String {
        let s = self.snapshot();
        let section_str = match s.section {
            Some(sec) => sec.name(),
            None      => "—",
        };
        let frobenius = if s.pending_splits == 0 {
            "closed  (μ∘δ=id)"
        } else {
            "open    (pending splits)"
        };
        format!(
            "=== {} ENGINE ===\n\
             Section    : {}\n\
             Steps      : {}\n\
             PC         : {}\n\
             Active regs: {}\n\
             Fixed regs : {}\n\
             Paradoxes  : {}\n\
             Frobenius  : {}  [open: {}]\n\
             Cross-segs : {}\n\
             Entropy Δ  : 0.00000000 J/K\n\
             Status     : BOOTSTRAP_COMPLETE",
            s.script.name(), section_str,
            s.step, s.pc, s.active, s.fixed, s.paradoxes,
            frobenius, s.pending_splits,
            s.cross_segment_refs,
        )
    }

    fn execute(&mut self, instr: Instruction) {
        let broken_frobenius = self.section.map(|s| s.broken_frobenius()).unwrap_or(false);

        match instr.opcode {
            Opcode::FSPLIT => {
                self.registers.entry(instr.dst).or_insert_with(TriPhaseRegister::new).engage();
                self.pending_splits += 1;
            }
            Opcode::FFUSE => {
                // Frobenius μ: close the most recent open δ.
                // In biological section the dominant path resets to VINIT instead,
                // so FFUSE arriving here means a minority completion — still valid.
                if self.pending_splits > 0 {
                    self.pending_splits -= 1;
                }
            }
            Opcode::VINIT => {
                // In biological section: VINIT resets all pending splits.
                // This is the empirical FSPLIT→VINIT dominant transition
                // (p=0.622) that breaks Frobenius closure in the heap segment.
                if broken_frobenius {
                    self.pending_splits = 0;
                }
            }
            Opcode::CLINK => {
                // Cross-segment pointer dereference. No Flux change; ΔS preserved.
                self.cross_segment_refs += 1;
            }
            Opcode::TANCH => {
                // Terminal anchor: closes an address resolution chain.
                // TANCH never self-chains (p(TANCH|FFUSE)=0 empirically) and
                // invariably delivers control to VINIT on the next instruction.
                // No Flux change; ΔS preserved.
            }
            Opcode::ENGAGR => {
                self.registers.entry(instr.dst).or_insert_with(TriPhaseRegister::new).engage();
            }
            Opcode::IFIX => {
                self.registers.entry(instr.dst).or_insert_with(TriPhaseRegister::new).fix();
            }
            // AFWD, AREV, ISCRIB, EVALT, EVALF: address navigation and lattice
            // assertions. Structurally present; ΔS = 0 by linear type constraint.
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
