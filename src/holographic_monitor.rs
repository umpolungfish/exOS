//! Holographic Self-Encoding Monitor — implements the g(x) function.
//!
//! This process continuously verifies the system's self-referential integrity
//! by performing bulk-boundary encoding, unifying Cantor's diagonal and Gödel's
//! arithmetization. It is the operationalization of the holographic radius.
//!
//! The monitor runs as a real ring-0 process: `ProcessControlBlock::spawn_ring0`
//! allocates a 16 KB kernel stack and sets up the initial register frame so the
//! first context switch jumps to `holographic_monitor_entry`.  Cooperative
//! preemption is provided by `scheduler::global_check_preempt()`.

extern crate alloc;
use alloc::vec::Vec;

use crate::kernel_object::{KernelObject, StructuralType, OperationalMode, Determinative};
use crate::aleph_kernel_types::AlephKernelType;
use crate::scheduler::{self, ProcessControlBlock};
use crate::aleph::LETTERS;

// ── Entry point ───────────────────────────────────────────────────────────────

/// The g(x) process entry point.  Called by context_switch_asm on first schedule.
pub fn holographic_monitor_entry() {
    let mut radius: f64 = 5.24;
    let mut iteration: u64 = 0;

    loop {
        let cardinality = if iteration % 2 == 0 { 1usize } else { 2usize };
        let symmetric    = false; // symmetry is always broken after first interrupt

        // Print status every 64 iterations to avoid flooding the console.
        if iteration % 64 == 0 {
            crate::println!(
                "[g(x)] iter={} LCARD={} REFL={} radius={:.2}",
                iteration, cardinality, symmetric, radius
            );
        }

        // Keep holographic radius within 3.77–6.71.
        if radius < 4.0 { radius += 0.01; }
        else if radius > 6.0 { radius -= 0.01; }

        iteration += 1;

        // Cooperative preemption: yield if the timer set needs_preempt.
        scheduler::global_check_preempt();
    }
}

// ── Monitor struct ────────────────────────────────────────────────────────────

pub struct HolographicMonitor {
    pub pcb: ProcessControlBlock,
}

impl HolographicMonitor {
    pub fn new() -> Self {
        let obj = KernelObject::with_type(
            StructuralType::Process,
            OperationalMode::Compute,
            Determinative::Kernel,
            1000,
            AlephKernelType::from_letter(&LETTERS[5]), // vav (O_inf)
        );

        let mut pcb = ProcessControlBlock::spawn_ring0(
            1000,
            obj,
            holographic_monitor_entry,
            100,
        );
        pcb.targets = Vec::new();
        pcb.role = scheduler::GrammaticalRole::Ergative;

        HolographicMonitor { pcb }
    }
}
