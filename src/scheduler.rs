//! Ergative-absolutive process model (from Basque grammar).
//!
//! In Basque, grammatical role is NOT defined by agency alone but by transitivity:
//! - **Ergative**: subject of a transitive verb — the entity that acts ON something
//! - **Absolutive**: subject of an intransitive verb OR the object of a transitive verb —
//!   the entity that runs standalone or receives action
//!
//! The same entity shifts role depending on whether it acts alone or acts on something.
//! This is context-dependent role encoding — R_lr (left-right asymmetric).
//!
//! Scheduling consequences:
//! - Ergative processes receive higher **interrupt priority** (they cause cascading effects)
//! - Absolutive processes receive higher **cache affinity** (they are self-contained)
//! - The same process can shift grammatical role depending on transitive context

use crate::kernel_object::KernelObject;
use alloc::collections::VecDeque;
use alloc::vec::Vec;

/// Grammatical role in the ergative-absolutive model
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrammaticalRole {
    /// Acts on another process (transitive) — higher interrupt priority
    Ergative,
    /// Runs standalone or receives action (intransitive/object) — higher cache affinity
    Absolutive,
}

/// Process control block with grammatical role
#[derive(Debug)]
pub struct ProcessControlBlock {
    pub id: u64,
    pub obj: KernelObject,
    pub role: GrammaticalRole,
    pub priority: u8,
    pub stack_pointer: u64,
    /// Processes this one acts upon (transitive targets)
    pub targets: Vec<u64>,
}

impl ProcessControlBlock {
    /// Determine role from transitivity: if this process has targets, it's ergative.
    /// Otherwise it's absolutive — same as Basque morphology.
    pub fn determine_role(&mut self) {
        if self.targets.is_empty() {
            self.role = GrammaticalRole::Absolutive;
        } else {
            self.role = GrammaticalRole::Ergative;
        }
    }

    /// Priority based on grammatical role:
    /// - Ergative: higher interrupt priority (cascading effects)
    /// - Absolutive: higher cache affinity (self-contained)
    pub fn effective_priority(&self) -> u8 {
        match self.role {
            GrammaticalRole::Ergative => self.priority + 10, // boost for interrupt
            GrammaticalRole::Absolutive => self.priority,     // base priority, cache affinity handled elsewhere
        }
    }
}

/// The ergative scheduler — boots in P_±_sym (no process distinguished),
/// activates after symmetry-breaking interrupt.
pub struct ErgativeScheduler {
    ready_queue: VecDeque<ProcessControlBlock>,
    running: Option<ProcessControlBlock>,
    /// Has the symmetry been broken yet?
    /// false = P_±_sym (all pooled, nothing distinguished)
    /// true  = P_asym (ergative model active)
    symmetry_broken: bool,
}

impl ErgativeScheduler {
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
            running: None,
            symmetry_broken: false,
        }
    }

    /// The symmetry-breaking event — analogous to δχ becoming nonzero.
    /// Called by the first interrupt handler.
    pub fn break_symmetry(&mut self) {
        self.symmetry_broken = true;
    }

    pub fn is_symmetric(&self) -> bool {
        !self.symmetry_broken
    }

    /// Add a process to the ready queue
    pub fn spawn(&mut self, pcb: ProcessControlBlock) {
        self.ready_queue.push_back(pcb);
    }

    /// Schedule the next process — ergative processes get priority boost
    pub fn schedule_next(&mut self) -> Option<&ProcessControlBlock> {
        if !self.symmetry_broken {
            return None; // P_±_sym: nothing distinguished, no scheduling
        }

        if let Some(current) = self.running.take() {
            // Re-evaluate role based on current targets (transitivity may have changed)
            let mut current = current;
            current.determine_role();
            self.ready_queue.push_back(current);
        }

        // Sort by effective priority (ergative processes rise to top)
        let mut tasks: Vec<_> = self.ready_queue.drain(..).collect();
        tasks.sort_by_key(|pcb| core::cmp::Reverse(pcb.effective_priority()));
        for task in tasks {
            self.ready_queue.push_back(task);
        }

        self.running = self.ready_queue.pop_front();
        self.running.as_ref()
    }
}
