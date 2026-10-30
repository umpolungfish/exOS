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
use crate::aleph_kernel_types::AlephKernelType;

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
///
/// With the ALEPH type bridge, every object also carries its 12-primitive type
/// signature, which constrains what it can do (IPC, memory, scheduling, filesystem).
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
    /// The 12-primitive ALEPH type that constrains this object's behavior.
    /// Inferred from the three-layer structure if not explicitly provided.
    pub aleph_type: AlephKernelType,
}

impl KernelObject {
    /// Create a kernel object with an auto-inferred ALEPH type.
    ///
    /// The type is inferred from the structural/operational/determinative
    /// combination (bulk → boundary type inference).
    pub fn new(
        structural: StructuralType,
        operational: OperationalMode,
        determinative: Determinative,
        id: u64,
    ) -> Self {
        let aleph_type = AlephKernelType::infer(structural, operational, determinative);
        Self {
            structural,
            operational,
            determinative,
            id,
            aleph_type,
        }
    }

    /// Create a kernel object with an explicit ALEPH type.
    ///
    /// Use this when you want to override the inferred type with a canonical
    /// Hebrew letter or a specifically crafted type.
    pub fn with_type(
        structural: StructuralType,
        operational: OperationalMode,
        determinative: Determinative,
        id: u64,
        aleph_type: AlephKernelType,
    ) -> Self {
        Self {
            structural,
            operational,
            determinative,
            id,
            aleph_type,
        }
    }

    /// A message/object without a determinative is syntactically malformed.
    /// This prevents type confusion at the protocol level.
    ///
    /// With ALEPH types, we also validate that the determinative is consistent
    /// with the object's Ω (topological protection) level:
    ///   - Kernel/Init: requires Ω ≥ 2 (Ω_Z — topologically protected)
    ///   - Driver/Service: requires Ω ≥ 1 (Ω_Z2 — partially protected)
    ///   - User: requires Ω = 0 (Ω_0 — unprotected, fast)
    pub fn is_well_formed(&self) -> bool {
        let omega = self.aleph_type.omega();
        let omega_consistent = match self.determinative {
            Determinative::Kernel | Determinative::Init => omega >= 2,
            Determinative::Driver | Determinative::Service => omega >= 1,
            Determinative::User => omega == 0,
        };
        omega_consistent
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
