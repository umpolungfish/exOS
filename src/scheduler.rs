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

#[derive(Debug, Clone)]
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
}

impl ProcessControlBlock {
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

    /// Legacy constructor for backward compatibility with boot tests.
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
        }
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
    pub fn yield_current(&mut self) {
        if let Some(mut current) = self.running.take() {
            current.state = ProcessState::Ready;
            current.ticks = 0;
            self.ready_queue.push_back(current);
        }
        self.run_next();
    }

    /// Schedule the next process — ergative + tier-aware priority
    pub fn schedule_next(&mut self) -> Option<&ProcessControlBlock> {
        if !self.symmetry_broken {
            return None;
        }
        self.run_next();
        self.running.as_ref()
    }

    fn run_next(&mut self) {
        let mut tasks: Vec<_> = self.ready_queue.drain(..).collect();
        tasks.sort_by_key(|pcb| core::cmp::Reverse(pcb.effective_priority()));
        for task in tasks {
            self.ready_queue.push_back(task);
        }

        if let Some(mut next) = self.ready_queue.pop_front() {
            next.state = ProcessState::Running;
            next.ticks = 0;
            self.running = Some(next);
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

        pcb.determine_role();

        let tier = aleph.tier();
        if tier == crate::aleph::Tier::O0 && !pcb.targets.is_empty() {
            return Err("O_0 process cannot be ergative — no self-modeling loop for transitivity");
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
        self.run_next();
    }

    pub fn unblock(&mut self, id: u64) {
        for pcb in self.ready_queue.iter_mut() {
            if pcb.id == id && pcb.state == ProcessState::Blocked {
                pcb.state = ProcessState::Ready;
                return;
            }
        }
    }

    /// Tick the scheduler — called by timer interrupt for round-robin preemption.
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
        if self.needs_preempt {
            self.yield_current();
            self.needs_preempt = false;
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
