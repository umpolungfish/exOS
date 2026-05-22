#![no_std]
#![feature(abi_x86_interrupt)]

extern crate alloc;

pub mod vga;
pub mod framebuffer;
pub mod font_renderer;
pub mod kernel_object;
pub mod scheduler;
pub mod memory;
pub mod filesystem;
pub mod filesystem_13;
pub mod ipc;
pub mod command;
pub mod interrupts;
pub mod keyboard;
pub mod serial;
pub mod history;
pub mod bench;
pub mod ata;
pub mod alfs;
pub mod vga_font_data;
pub mod holographic_monitor;

// ── ℵ-OS λ_ℵ Type System ─────────────────────────────────────────────────────
// Hebrew letter 12-primitive tuples, lattice operations, REPL, and shell commands.

pub mod aleph;
pub mod aleph_parser;
pub mod aleph_eval;
pub mod aleph_repl;
pub mod aleph_commands;

// ── 14-Sefirot Structural Types ──────────────────────────────────────────────
// Full Sefer Ha-Iyun emanation chain as first-class ALEPH types.

pub mod aleph_sefirot;

// ── Type-System Bridge ───────────────────────────────────────────────────────
// Operationalizes the 12-primitive type lattice: makes ALEPH types constrain
// kernel object behavior (IPC, memory, scheduling, filesystem).

pub mod aleph_kernel_types;
pub mod programs;

// ── Tri-Phase Script Engines ──────────────────────────────────────────────────
// Shared IMASM VM + four script-specific front-ends (Voynich, Rohonc, Linear A,
// Emerald Tablet). All four reduce to the same 12 categorical opcodes on the same
// TriPhase registers. Crystal imscription distances computed via the OS weighted
// IG metric. The Emerald Tablet is the only compiled manuscript with both gates
// open and quantum-coherent fidelity (C = 1.0) — it is the grammar's self-statement.

pub mod imasm_vm;
pub mod voynich;
pub mod rohonc;
pub mod linear_a;
pub mod emerald_tablet;
pub mod imasm_commands;

// ── ParaASM VM — Belnap FOUR paraconsistent machine ──────────────────────────
// Full ParaASM ISA (ENGAGR/FSPLIT/FFUSE/IFIX + control flow + stack + I/O),
// text assembler, and kernel shell handler. EMIT → serial; READ → N.

pub mod para_vm;
pub mod para_commands;
pub mod para_shor_commands;
pub mod para_align_commands;
pub mod para_wasm;
pub mod para_wasm_commands;
// ── Stub modules integrated into IPC/scheduler/aleph ──────────────────
pub mod interaction_grammar;
pub mod frobenius_verification;
pub mod stoichiometry;
pub mod phi_ep;
pub mod resource_isolation;


/// Global allocator — initialized by the kernel entry point
#[global_allocator]
pub static ALLOCATOR: linked_list_allocator::LockedHeap =
    linked_list_allocator::LockedHeap::empty();