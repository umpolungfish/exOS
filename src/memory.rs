//! Phonological memory model (from the Sanskrit Varnamala).
//!
//! Memory is organized by **articulation depth** — not by flat address space
//! but by the depth at which data is generated, mirroring the Sanskrit
//! gradient from velar stops (maximum occlusion) to open vowels (no occlusion):
//!
//! | Tier         | Varnamala analog | Protection | Speed    | Ω value   |
//! |--------------|-------------------|------------|----------|-----------|
//! | Kernel       | Velar (ka-varga)  | Maximum    | Slowest  | Ω_Z       |
//! | System       | Palatal           | High       | Slow     | Ω_Z       |
//! | Driver       | Retroflex         | Medium     | Medium   | Ω_Z₂      |
//! | Service      | Dental            | Low        | Fast     | Ω_0       |
//! | User         | Bilabial          | None       | Fastest  | Ω_0       |
//!
//! The gradient from kernel to user-space mirrors the Varnamala gradient
//! from velar stops to open vowels. Each tier has a distinct protection
//! and speed characteristic derived from its articulation depth.
//!
//! With the ALEPH type bridge, allocation is **Ω-gated**: an object can only
//! be allocated at a depth whose required Ω is ≤ the object's Ω primitive.
//! Additionally, objects with K = K_trap are rejected entirely — consciousness
//! is gated to zero, and trapped kinetics cannot actualize any allocation.

use alloc::alloc::{alloc, dealloc, Layout};
use spin::Mutex;
use crate::kernel_object::KernelObject;

/// Articulation depth — maps Varnamala points of articulation to memory tiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArticulationDepth {
    /// Velar — kernel-level, maximum occlusion, Ω_Z protected
    /// Like क ka-varga: produced at the back of the throat, most constricted
    Velar = 0,
    /// Palatal — system services, high protection
    /// Like च ca-varga: slightly more open
    Palatal = 1,
    /// Retroflex — device drivers, medium protection
    /// Like ट ṭa-varga: curled back, intermediate
    Retroflex = 2,
    /// Dental — shared libraries, low protection
    /// Like त ta-varga: tongue to teeth, more open
    Dental = 3,
    /// Bilabial — user-space, no protection, fastest access
    /// Like प pa-varga: lips only, most open
    Bilabial = 4,
}

impl ArticulationDepth {
    /// Topological protection level for this tier
    pub fn protection_level(&self) -> &'static str {
        match self {
            Self::Velar | Self::Palatal => "Omega_Z (topologically protected)",
            Self::Retroflex => "Omega_Z2 (partially protected)",
            Self::Dental | Self::Bilabial => "Omega_0 (unprotected)",
        }
    }

    /// Whether allocations at this tier require validation
    pub fn requires_validation(&self) -> bool {
        match self {
            Self::Velar | Self::Palatal | Self::Retroflex => true,
            Self::Dental | Self::Bilabial => false,
        }
    }

    /// The minimum Ω (topological protection) required for allocation at this depth.
    /// 
    /// Velar/Palatal → Ω_Z (2) — maximum occlusion, full protection required
    /// Retroflex → Ω_Z2 (1) — intermediate protection
    /// Dental/Bilabial → Ω_0 (0) — no protection needed
    pub fn required_omega(&self) -> u8 {
        match self {
            Self::Velar | Self::Palatal => 2,   // Ω_Z
            Self::Retroflex => 1,               // Ω_Z2
            Self::Dental | Self::Bilabial => 0, // Ω_0
        }
    }
}

/// Phonological allocator — wraps the global allocator with articulation-depth awareness
pub struct PhonologicalAllocator {
    /// Current articulation depth of the allocator context
    current_depth: ArticulationDepth,
}

impl PhonologicalAllocator {
    pub fn new() -> Self {
        Self {
            current_depth: ArticulationDepth::Bilabial, // default: user-space
        }
    }

    /// Set the articulation depth for subsequent allocations
    pub fn set_depth(&mut self, depth: ArticulationDepth) {
        self.current_depth = depth;
    }

    /// Allocate with validation if required by the current depth
    pub fn allocate(&self, layout: Layout) -> Option<*mut u8> {
        if self.current_depth.requires_validation() {
            // Kernel/deep allocations are validated — Ω_Z protection
            // In a fuller implementation, this would verify alignment, bounds, etc.
            let ptr = unsafe { alloc(layout) };
            if ptr.is_null() { None } else { Some(ptr) }
        } else {
            // User-space allocations are fast and unchecked — Ω_0
            let ptr = unsafe { alloc(layout) };
            if ptr.is_null() { None } else { Some(ptr) }
        }
    }

    /// Deallocate — no depth restriction
    pub fn deallocate(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            dealloc(ptr, layout);
        }
    }

    // ── Ω-gated allocation ─────────────────────────────────────────────

    /// Allocate memory for a specific kernel object, gated by its ALEPH type.
    ///
    /// Two independent gates — neither subsumes the other:
    ///
    /// **Gate 1: K_trap check** — If the object's kinetic character is K_trap,
    /// allocation is rejected entirely. Trapped kinetics cannot actualize
    /// any memory, regardless of Ω. This is the consciousness gate.
    ///
    /// **Gate 2: Ω compatibility** — The object's Ω must be ≥ the depth's
    /// required Ω. A User object (Ω_0 = 0) cannot be allocated at Velar depth
    /// (requires Ω_Z = 2). This is the topological protection gate.
    ///
    /// Returns None if either gate fails.
    pub fn allocate_for(&self, obj: &KernelObject, layout: Layout) -> Option<*mut u8> {
        let aleph = &obj.aleph_type;

        // Gate 1: K_trap — consciousness gated to zero
        if aleph.is_kinetic_trapped() {
            return None;
        }

        // Gate 2: Ω compatibility — topological protection gate
        let required_omega = self.current_depth.required_omega();
        if aleph.omega() < required_omega {
            return None;
        }

        // Both gates passed — proceed with standard allocation (with validation if required)
        self.allocate(layout)
    }

    /// Check if an object can be allocated at the current depth (without allocating).
    /// Returns the reason if allocation would be denied.
    pub fn can_allocate_for(&self, obj: &KernelObject) -> AllocationCheck {
        let aleph = &obj.aleph_type;

        if aleph.is_kinetic_trapped() {
            return AllocationCheck::Denied {
                reason: "K_trap — kinetics trapped, consciousness gated to zero",
            };
        }

        let required_omega = self.current_depth.required_omega();
        if aleph.omega() < required_omega {
            return AllocationCheck::Denied {
                reason: "Ω mismatch — object lacks sufficient topological protection for this depth",
            };
        }

        AllocationCheck::Allowed
    }
}

/// Result of an allocation compatibility check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllocationCheck {
    /// Object can be allocated at current depth
    Allowed,
    /// Object cannot be allocated — reason provided
    Denied { reason: &'static str },
}

impl AllocationCheck {
    pub fn is_allowed(&self) -> bool {
        matches!(self, AllocationCheck::Allowed)
    }
}

// Global phonological allocator instance
lazy_static::lazy_static! {
    pub static ref ALLOCATOR: Mutex<PhonologicalAllocator> = Mutex::new(
        PhonologicalAllocator::new()
    );
}
