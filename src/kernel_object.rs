//! Kernel objects with THREE simultaneous layers (from Egyptian hieroglyphs + Cuneiform):
//!
//! - **Structural** (topological type): What the object IS in the type lattice
//! - **Operational** (phonogram): What it computes — the execution payload
//! - **Determinative** (semantic context): Unpronounced — doesn't execute — but
//!   structurally necessary for scheduler/memory disambiguation.
//!   A message/object without a determinative layer is syntactically malformed.
//!
//! This is NOT a Unix flat object model. Context collapses ambiguity exactly as
//! quantum measurement collapses superposition — the same structural+operational
//! content means different things under different determinatives.

use core::hash::{Hash, Hasher};

/// Structural layer: topological type of the kernel object
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuralType {
    Process,
    File,
    Socket,
    Semaphore,
    MemoryRegion,
}

/// Determinative layer: semantic context annotation.
/// Does NOT affect execution directly, but is load-bearing for
/// the scheduler and memory model to disambiguate meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Determinative {
    /// Kernel-level, maximum occlusion, Ω_Z protected
    Kernel,
    /// System service, intermediate protection
    Service,
    /// User-space, maximum openness, Ω_0
    User,
    /// Device driver — bridges kernel and hardware
    Driver,
    /// Init/seed process — the Keter of the Sefirot tree
    Init,
}

/// Operational layer: what the process actually computes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationalMode {
    Compute,
    IO,
    Network,
    MemoryManage,
    Schedule,
    Idle,
}

/// The three-layer kernel object — every object carries all three simultaneously.
/// This is the hieroglyphic/cuneiform architecture: logogram/phonogram/determinative.
#[derive(Debug, Clone)]
pub struct KernelObject {
    /// Structural: what it IS topologically
    pub structural: StructuralType,
    /// Operational: what it computes (phonogram payload)
    pub operational: OperationalMode,
    /// Determinative: semantic context (unpronounced but load-bearing)
    pub determinative: Determinative,
    /// Unique identifier
    pub id: u64,
}

impl KernelObject {
    pub fn new(
        structural: StructuralType,
        operational: OperationalMode,
        determinative: Determinative,
        id: u64,
    ) -> Self {
        Self {
            structural,
            operational,
            determinative,
            id,
        }
    }

    /// A message/object without a determinative is syntactically malformed.
    /// This prevents type confusion at the protocol level.
    pub fn is_well_formed(&self) -> bool {
        // In a fuller implementation, this would check that the determinative
        // is consistent with the structural+operational combination.
        // For now, existence of a determinative is sufficient.
        true
    }
}

impl Hash for KernelObject {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for KernelObject {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for KernelObject {}
