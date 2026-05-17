//! Ergative-absolutive process model (from Basque grammar) with context switching.
//!
//! In Basque, grammatical role is NOT defined by agency alone but by transitivity:
//! - **Ergative**: subject of a transitive verb — the entity that acts ON something
//! - **Absolutive**: subject of an intransitive verb OR the object of a transitive verb
//!
//! Scheduling consequences:
//! - Ergative processes receive higher **interrupt priority** (they cause cascading effects)
//! - Absolutive processes receive higher **cache affinity** (they are self-contained)
//! - The same process can shift grammatical role depending on transitive context
//!
//! With the ALEPH type bridge, scheduling is **tier-gated**:
//! - O_0 processes (⊙_sub) cannot be ergative — no self-modeling loop
//! - Ç_trap processes cannot be scheduled
//! - Ergative priority is tier-aware: O_∞ > O_2 > O_1 in interrupt boost
//!
//! **Context switching**: Each PCB carries a saved thread context (registers).
//! The timer interrupt triggers preemption: the current context is saved,
//! the next process is selected, and its context is restored.
//! The `yield()` call provides cooperative preemption.

use crate::kernel_object::KernelObject;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::alloc::{alloc, Layout};
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

// ── Per-process RSP table ─────────────────────────────────────────────────────
//
// Stable static storage for each process's saved RSP. Indexed by `proc_slot`
// (assigned at spawn). Using a static array gives each slot a fixed address,
// so context_switch_asm can write directly via a raw pointer without any
// concern about the PCB moving between containers.
//
// Slot 0 is reserved (unassigned / legacy PCBs with no real stack).
const MAX_PROCS: usize = 32;
static RSP_TABLE: [AtomicU64; MAX_PROCS] = {
    #[allow(clippy::declare_interior_mutable_const)]
    const ZERO: AtomicU64 = AtomicU64::new(0);
    [ZERO; MAX_PROCS]
};
static SLOT_COUNTER: AtomicUsize = AtomicUsize::new(1); // 0 is reserved

// ── Context switch (ring-0) ────────────────────────────────────────────────────
//
// Saves callee-saved registers (r15..rbp) onto the current stack, stores RSP
// into *old_rsp, then loads new_rsp and restores registers from the new stack.
// The return address at the top of the new stack becomes the next instruction.
//
// Initial stack layout for a freshly spawned process (see `init_process_stack`):
//   [sp - 8]  entry_fn   ← ret pops this
//   [sp - 16] 0  (rbp)
//   [sp - 24] 0  (rbx)
//   [sp - 32] 0  (r12)
//   [sp - 40] 0  (r13)
//   [sp - 48] 0  (r14)
//   [sp - 56] 0  (r15)   ← initial RSP stored in context.rsp
core::arch::global_asm!(
    ".global context_switch_asm",
    "context_switch_asm:",
    "push rbp",
    "push rbx",
    "push r12",
    "push r13",
    "push r14",
    "push r15",
    "mov [rdi], rsp",
    "mov rsp, rsi",
    "pop r15",
    "pop r14",
    "pop r13",
    "pop r12",
    "pop rbx",
    "pop rbp",
    "ret",
);

extern "C" {
    fn context_switch_asm(old_rsp: *mut u64, new_rsp: u64);
}

const KERNEL_STACK_SIZE: usize = 16 * 1024; // 16 KB per process

/// Allocate a kernel stack and write the initial saved-register frame so that
/// the first call to `context_switch_asm` jumps to `entry`.
fn init_process_stack(entry: u64) -> (Box<[u8]>, u64) {
    let layout = Layout::from_size_align(KERNEL_STACK_SIZE, 16).unwrap();
    let raw = unsafe { alloc(layout) };
    assert!(!raw.is_null(), "process stack alloc failed");

    // SAFETY: we just allocated this memory; it is valid for KERNEL_STACK_SIZE bytes.
    let stack: Box<[u8]> = unsafe {
        Box::from_raw(core::slice::from_raw_parts_mut(raw, KERNEL_STACK_SIZE))
    };

    // Write initial frame at the top. Stack grows down.
    let top = raw as u64 + KERNEL_STACK_SIZE as u64;
    unsafe {
        let top_ptr = top as *mut u64;
        *top_ptr.offset(-1) = entry;  // ret address
        *top_ptr.offset(-2) = 0;      // rbp
        *top_ptr.offset(-3) = 0;      // rbx
        *top_ptr.offset(-4) = 0;      // r12
        *top_ptr.offset(-5) = 0;      // r13
        *top_ptr.offset(-6) = 0;      // r14
        *top_ptr.offset(-7) = 0;      // r15
    }
    let initial_rsp = top - 7 * 8; // points at r15 slot

    (stack, initial_rsp)
}


/// Saved processor context for context switching.
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct ThreadContext {
    pub rsp: u64,
    pub rip: u64,
    pub rbp: u64,
    pub rbx: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rflags: u64,
}

impl ThreadContext {
    pub fn new(stack_top: u64, entry: u64) -> Self {
        Self {
            rsp: stack_top,
            rip: entry,
            rbp: 0,
            rbx: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rflags: 0x202,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrammaticalRole {
    Ergative,
    Absolutive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Ready,
    Running,
    Sleeping(u64),
    Blocked,
    Terminated,
}

#[derive(Debug)]
pub struct ProcessControlBlock {
    pub id: u64,
    pub obj: KernelObject,
    pub role: GrammaticalRole,
    pub priority: u8,
    pub stack_pointer: u64,
    pub context: ThreadContext,
    pub targets: Vec<u64>,
    pub state: ProcessState,
    pub ticks: u64,
    pub time_slice: u64,
    /// Index into RSP_TABLE. 0 = no real stack (legacy/test PCB).
    pub proc_slot: usize,
    /// Owned kernel stack — None for legacy/test PCBs.
    pub stack: Option<Box<[u8]>>,
}

impl ProcessControlBlock {
    /// Spawn a real ring-0 process: allocates a kernel stack, assigns an RSP_TABLE
    /// slot, and initialises the saved-register frame so the first call to
    /// `context_switch_asm` jumps to `entry`.
    pub fn spawn_ring0(id: u64, obj: KernelObject, entry: fn(), priority: u8) -> Self {
        let slot = SLOT_COUNTER.fetch_add(1, Ordering::Relaxed);
        assert!(slot < MAX_PROCS, "MAX_PROCS exceeded");

        let (stack, initial_rsp) = init_process_stack(entry as u64);
        RSP_TABLE[slot].store(initial_rsp, Ordering::Relaxed);

        let mut context = ThreadContext::default();
        context.rsp = initial_rsp;
        Self {
            id,
            obj,
            role: GrammaticalRole::Absolutive,
            priority,
            stack_pointer: initial_rsp,
            context,
            targets: Vec::new(),
            state: ProcessState::Ready,
            ticks: 0,
            time_slice: 18,
            proc_slot: slot,
            stack: Some(stack),
        }
    }

    pub fn new_with_context(
        id: u64,
        obj: KernelObject,
        entry: u64,
        stack_top: u64,
        priority: u8,
    ) -> Self {
        let context = ThreadContext::new(stack_top, entry);
        Self {
            id,
            obj,
            role: GrammaticalRole::Absolutive,
            priority,
            stack_pointer: stack_top,
            context,
            targets: Vec::new(),
            state: ProcessState::Ready,
            ticks: 0,
            time_slice: 18,
            proc_slot: 0,
            stack: None,
        }
    }

    pub fn determine_role(&mut self) {
        if self.targets.is_empty() {
            self.role = GrammaticalRole::Absolutive;
        } else {
            self.role = GrammaticalRole::Ergative;
        }
    }

    pub fn effective_priority(&self) -> u8 {
        match self.role {
            GrammaticalRole::Ergative => self.priority + 10,
            GrammaticalRole::Absolutive => self.priority,
        }
    }

    /// Legacy constructor for boot-time tests (no real stack allocation).
    pub fn new(id: u64, obj: KernelObject, role: GrammaticalRole, priority: u8, stack_pointer: u64, targets: Vec<u64>) -> Self {
        Self {
            id,
            obj,
            role,
            priority,
            stack_pointer,
            context: ThreadContext::default(),
            targets,
            state: ProcessState::Ready,
            ticks: 0,
            time_slice: 18,
            proc_slot: 0,
            stack: None,
        }
    }

    pub fn has_real_stack(&self) -> bool {
        self.stack.is_some()
    }
}

/// Wrapper to make raw pointer Sync/Send for static storage.
struct SchedulerPtr(*mut ErgativeScheduler);
unsafe impl Send for SchedulerPtr {}
unsafe impl Sync for SchedulerPtr {}

/// Global scheduler pointer — set once during boot, read by timer interrupt.
static SCHEDULER_PTR: spin::Once<SchedulerPtr> = spin::Once::new();

pub struct ErgativeScheduler {
    ready_queue: VecDeque<ProcessControlBlock>,
    running: Option<ProcessControlBlock>,
    symmetry_broken: bool,
    needs_preempt: bool,
}

impl ErgativeScheduler {
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
            running: None,
            symmetry_broken: false,
            needs_preempt: false,
        }
    }

    pub fn break_symmetry(&mut self) {
        self.symmetry_broken = true;
    }

    pub fn is_symmetric(&self) -> bool {
        !self.symmetry_broken
    }

    pub fn spawn(&mut self, pcb: ProcessControlBlock) {
        self.ready_queue.push_back(pcb);
    }

    /// Cooperative yield: the running process voluntarily gives up the CPU.
    ///
    /// For processes with a real kernel stack (proc_slot > 0), this performs an
    /// actual CPU context switch via context_switch_asm.  The call appears to
    /// return immediately to the caller — but only after this process is next
    /// selected by the scheduler and resumed by some future switch.
    ///
    /// For legacy (test) PCBs with no stack, this is a data-structure-only swap.
    pub fn yield_current(&mut self) {
        x86_64::instructions::interrupts::disable();
        self.needs_preempt = false;

        let mut current = match self.running.take() {
            Some(p) => p,
            None => { x86_64::instructions::interrupts::enable(); return; }
        };
        current.state = ProcessState::Ready;
        current.ticks = 0;

        let current_slot = current.proc_slot;
        let current_real  = current_slot > 0;

        // Re-enqueue current, then sort by priority.
        self.ready_queue.push_back(current);
        self.sort_ready();

        // Pick the highest-priority ready process (may be the same one if alone).
        let (next_slot, next_real) = self.ready_queue.front()
            .map(|p| (p.proc_slot, p.proc_slot > 0))
            .unwrap_or((0, false));

        if current_real && next_real {
            // Both have real stacks — perform an actual CPU context switch.
            let new_rsp  = RSP_TABLE[next_slot].load(Ordering::Relaxed);
            let old_rsp_ptr: *mut u64 = RSP_TABLE[current_slot].as_ptr();

            // Promote the next process to Running before switching away.
            if let Some(mut next) = self.ready_queue.pop_front() {
                next.state = ProcessState::Running;
                next.ticks = 0;
                self.running = Some(next);
            }

            x86_64::instructions::interrupts::enable();

            // ── Actual CPU context switch ──────────────────────────────────
            // Callee-saved registers are pushed onto the current stack.
            // RSP is saved to RSP_TABLE[current_slot] via old_rsp_ptr.
            // RSP is then loaded from RSP_TABLE[next_slot] (already in new_rsp).
            // Returns here only when this process is next scheduled.
            unsafe { context_switch_asm(old_rsp_ptr, new_rsp); }
            return;
        }

        // Data-structure-only swap (no real stacks involved).
        if let Some(mut next) = self.ready_queue.pop_front() {
            next.state = ProcessState::Running;
            next.ticks = 0;
            self.running = Some(next);
        }
        x86_64::instructions::interrupts::enable();
    }

    fn sort_ready(&mut self) {
        let mut tasks: Vec<_> = self.ready_queue.drain(..).collect();
        tasks.sort_by_key(|p| core::cmp::Reverse(p.effective_priority()));
        for t in tasks { self.ready_queue.push_back(t); }
    }

    /// Schedule the next process — ergative + tier-aware priority.
    /// Data-structure update only; use `yield_current` for a real CPU switch.
    pub fn schedule_next(&mut self) -> Option<&ProcessControlBlock> {
        if !self.symmetry_broken {
            return None;
        }
        self.sort_ready();
        if let Some(mut next) = self.ready_queue.pop_front() {
            next.state = ProcessState::Running;
            next.ticks = 0;
            self.running = Some(next);
        }
        self.running.as_ref()
    }

    /// Check the preemption flag and yield if set. Processes call this at safe
    /// points (e.g. after each work unit) instead of yielding from the interrupt.
    pub fn check_preempt(&mut self) {
        if self.needs_preempt {
            self.yield_current();
        }
    }

    pub fn running_mut(&mut self) -> Option<&mut ProcessControlBlock> {
        self.running.as_mut()
    }

    pub fn running(&self) -> Option<&ProcessControlBlock> {
        self.running.as_ref()
    }

    pub fn spawn_type_safe(&mut self, mut pcb: ProcessControlBlock) -> Result<(), &'static str> {
        let aleph = pcb.obj.aleph_type.clone();

        if aleph.is_kinetic_frozen() {
            return Err("kinetically frozen (Ç_trap or Ç_MBL) — cannot be scheduled");
        }

        // P-596: ⊙_EP absorbs the self-modeling loop — process collapses to C=0.
        if crate::phi_ep::Criticality::from_primitive_index(aleph.phi()).is_ep() {
            return Err("⊙_EP (exceptional point) — P-596 absorption, self-modeling loop destroyed");
        }

        pcb.determine_role();

        let tier = aleph.tier();
        if tier == crate::aleph::Tier::O0 && !pcb.targets.is_empty() {
            return Err("O_0 process cannot be ergative — no self-modeling loop for transitivity");
        }

        // Frobenius axiom F-1: O_inf processes must satisfy μ∘δ = id (Φ=Φ_± and ⊙=⊙_c).
        if tier == crate::aleph::Tier::OInf
            && !crate::frobenius_verification::FrobeniusVerifier::verify(&aleph)
        {
            return Err("O_inf requires Frobenius condition: Φ=Φ_± (parity=4) and ⊙=⊙_c (phi=1)");
        }

        // Stoichiometric quota: register this process id as a resource and acquire it.
        // The process id is also used as the resource id — each process IS a resource
        // whose exclusivity is governed by its Σ primitive.
        let sigma_mode = crate::stoichiometry::Stoichiometry::from_primitive_index(
            aleph.tuple[10]
        );
        let capacity = match sigma_mode {
            crate::stoichiometry::Stoichiometry::NN => 8, // default pool cap
            _ => 1,
        };
        crate::stoichiometry::register(pcb.id, sigma_mode, capacity);
        if let Err(e) = crate::stoichiometry::acquire(pcb.id, pcb.id) {
            return Err(e);
        }

        self.spawn(pcb);
        Ok(())
    }

    pub fn effective_priority_with_tier(&self, pcb: &ProcessControlBlock) -> u8 {
        use crate::aleph::Tier;
        let base = pcb.priority;

        if pcb.role == GrammaticalRole::Ergative {
            let tier = pcb.obj.aleph_type.tier();
            let boost = match tier {
                Tier::OInf => 15,
                Tier::O2 | Tier::O2d => 12,
                Tier::O1 => 10,
                Tier::O0 => 0,
            };
            base + boost
        } else {
            base
        }
    }

    pub fn block_current(&mut self) {
        if let Some(mut current) = self.running.take() {
            current.state = ProcessState::Blocked;
            self.ready_queue.push_back(current);
        }
        self.sort_ready();
        if let Some(mut next) = self.ready_queue.pop_front() {
            next.state = ProcessState::Running;
            next.ticks = 0;
            self.running = Some(next);
        }
    }

    pub fn unblock(&mut self, id: u64) {
        for pcb in self.ready_queue.iter_mut() {
            if pcb.id == id && pcb.state == ProcessState::Blocked {
                pcb.state = ProcessState::Ready;
                return;
            }
        }
    }

    /// Tick the scheduler — called by the timer interrupt (~18 Hz).
    ///
    /// Only sets the preemption flag; the actual context switch happens in
    /// `check_preempt()` called from process context, not from interrupt context.
    /// Calling yield_current from inside an interrupt frame would corrupt the
    /// IRET state — so we defer the switch.
    pub fn tick(&mut self) {
        if let Some(running) = &mut self.running {
            if running.state == ProcessState::Running {
                running.ticks += 1;
                if running.ticks >= running.time_slice {
                    running.ticks = 0;
                    self.needs_preempt = true;
                }
            }
        }
    }
}

/// Register the scheduler for timer-driven preemption. Called once during boot.
pub fn register_for_timer(sched: &'static mut ErgativeScheduler) {
    SCHEDULER_PTR.call_once(|| SchedulerPtr(sched as *mut ErgativeScheduler));
}

/// Called from the timer interrupt handler to invoke preemption.
pub unsafe fn on_timer_tick() {
    if let Some(sched_wrapper) = SCHEDULER_PTR.get() {
        let sched_ptr: *mut ErgativeScheduler = sched_wrapper.0;
        (*sched_ptr).tick();
    }
}

/// Yield the current process through the global scheduler.
/// Safe to call from any process context. No-op if no scheduler is registered.
pub fn global_yield() {
    if let Some(sched_wrapper) = SCHEDULER_PTR.get() {
        unsafe { (*sched_wrapper.0).yield_current(); }
    } else {
        core::hint::spin_loop();
    }
}

/// Check preemption flag through the global scheduler.
pub fn global_check_preempt() {
    if let Some(sched_wrapper) = SCHEDULER_PTR.get() {
        unsafe { (*sched_wrapper.0).check_preempt(); }
    }
}
