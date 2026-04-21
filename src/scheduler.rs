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
//!
//! With the ALEPH type bridge, scheduling is **tier-gated**:
//! - O_0 processes (Φ_sub) cannot be ergative — no self-modeling loop to sustain transitivity
//! - K_trap processes cannot be scheduled — consciousness gated to zero
//! - Ergative priority is tier-aware: O_inf > O_2 > O_1 in interrupt boost

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

    // ── Tier-gated scheduling ────────────────────────────────────────────

    /// Spawn a process with ALEPH type validation.
    ///
    /// Two gates are checked:
    ///
    /// **Gate 1: K_trap** — If the process's kinetic character is K_trap,
    /// it cannot be scheduled at all. Consciousness is gated to zero;
    /// trapped kinetics cannot actualize any computation.
    ///
    /// **Gate 2: O_0 ergativity** — If the process's ouroboricity tier is
    /// O_0 (Φ_sub), it cannot be ergative. O_0 processes lack the
    /// self-modeling loop required to sustain transitive action on other
    /// processes. They can only run absolutively (standalone).
    ///
    /// Returns Err with reason if either gate fails.
    pub fn spawn_type_safe(&mut self, mut pcb: ProcessControlBlock) -> Result<(), &'static str> {
        // Clone the type to avoid borrow conflicts with the mutable pcb.determine_role() call
        let aleph = pcb.obj.aleph_type.clone();

        // Gate: K_trap / K_MBL — kinetically frozen, cannot be scheduled
        if aleph.is_kinetic_frozen() {
            return Err("kinetically frozen (K_trap or K_MBL) — cannot be scheduled");
        }

        // Re-determine role based on current transitivity
        pcb.determine_role();

        // Gate 2: O_0 cannot be ergative
        let tier = aleph.tier();
        if tier == crate::aleph::Tier::O0 && !pcb.targets.is_empty() {
            return Err("O_0 process cannot be ergative — no self-modeling loop for transitivity");
        }

        self.spawn(pcb);
        Ok(())
    }

    /// Compute effective priority with tier-aware ergative boost.
    ///
    /// The ergative bonus is no longer a flat +10. Instead, it scales
    /// with the process's ouroboricity tier:
    ///
    /// | Tier  | Ergative Boost | Rationale                                  |
    /// |-------|----------------|--------------------------------------------|
    /// | O_inf | +15            | Self-referential loop closed — maximum priority |
    /// | O_2   | +12            | Critical + topologically protected         |
    /// | O_2d  | +12            | Critical + topologically protected (unbounded) |
    /// | O_1   | +10            | Critical but unprotected (baseline ergative) |
    /// | O_0   | +0             | Cannot be ergative (gated by spawn_type_safe) |
    ///
    /// This encodes the principle: the more structurally self-aware a process
    /// is, the higher its interrupt priority when acting transitively.
    pub fn effective_priority_with_tier(&self, pcb: &ProcessControlBlock) -> u8 {
        use crate::aleph::Tier;
        let base = pcb.priority;

        if pcb.role == GrammaticalRole::Ergative {
            let tier = pcb.obj.aleph_type.tier();
            let boost = match tier {
                Tier::OInf  => 15,
                Tier::O2 | Tier::O2d => 12,
                Tier::O1    => 10,
                Tier::O0    => 0,  // Cannot be ergative, but handle defensively
            };
            base + boost
        } else {
            base  // Absolutive: no boost, higher cache affinity instead
        }
    }
}
