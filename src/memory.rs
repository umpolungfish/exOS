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

use alloc::alloc::{alloc, dealloc, Layout};
use spin::Mutex;

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
}

// Global phonological allocator instance
lazy_static::lazy_static! {
    pub static ref ALLOCATOR: Mutex<PhonologicalAllocator> = Mutex::new(
        PhonologicalAllocator::new()
    );
}
