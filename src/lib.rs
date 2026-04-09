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

// ── ℵ-OS λ_ℵ Type System ─────────────────────────────────────────────────────
// Hebrew letter 12-primitive tuples, lattice operations, REPL, and shell commands.

pub mod aleph;
pub mod aleph_parser;
pub mod aleph_eval;
pub mod aleph_repl;
pub mod aleph_commands;

// ── Type-System Bridge ───────────────────────────────────────────────────────
// Operationalizes the 12-primitive type lattice: makes ALEPH types constrain
// kernel object behavior (IPC, memory, scheduling, filesystem).

pub mod aleph_kernel_types;
pub mod programs;

/// Global allocator — initialized by the kernel entry point
#[global_allocator]
pub static ALLOCATOR: linked_list_allocator::LockedHeap =
    linked_list_allocator::LockedHeap::empty();
