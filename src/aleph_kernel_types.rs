//! ALEPH kernel type bridge — maps the 12-primitive type lattice onto kernel objects.
//!
//! This is the operationalization layer: the ALEPH type system no longer just
//! describes kernel objects from the outside; it **constrains their behavior**.
//!
//! Every kernel object carries an `AlephKernelType` that determines:
//! - Whether it can communicate with other objects (IPC type-safety gate)
//! - At which memory depth it can be allocated (Ω-gated allocation)
//! - Whether it can be scheduled as ergative (tier-gated spawn)
//! - Which Sefirot depth it can access (Φ-gated filesystem navigation)
//!
//! The type is either:
//! - **Canonical**: one of the 22 Hebrew letters (named, auditable)
//! - **Inferred**: derived from the three-layer (structural/operational/determinative)
//!   combination via bulk→boundary type inference

extern crate alloc;

use alloc::format;
use crate::aleph;
use crate::aleph::Tuple;
use crate::kernel_object::{StructuralType, OperationalMode, Determinative};

// ── AlephKernelType ──────────────────────────────────────────────────────────

/// The 12-primitive type of a kernel object.
/// 
/// Either canonical (a known Hebrew letter) or synthetic (inferred from
/// the object's three-layer structure).
#[derive(Debug, Clone)]
pub struct AlephKernelType {
    /// The 12-primitive tuple
    pub tuple: Tuple,
    /// If this type corresponds to a canonical Hebrew letter, which one.
    /// None means this is an inferred/synthetic type.
    pub canonical_index: Option<usize>,
}

impl AlephKernelType {
    // ── Construction ─────────────────────────────────────────────────────

    /// Create from a canonical Hebrew letter.
    pub fn from_letter(letter: &'static aleph::LetterDef) -> Self {
        // Find the index in LETTERS
        let idx = aleph::LETTERS.iter()
            .position(|l| core::ptr::eq(l, letter));
        Self {
            tuple: letter.t,
            canonical_index: idx,
        }
    }

    /// Create from a raw 12-tuple (synthetic/inferred type).
    pub fn from_tuple(t: Tuple) -> Self {
        // Check if it matches any canonical letter
        let idx = aleph::LETTERS.iter()
            .position(|l| l.t == t);
        Self {
            tuple: t,
            canonical_index: idx,
        }
    }

    /// Infer the type from the three-layer kernel object structure.
    /// 
    /// This is the **bulk → boundary type inference**: given observed behaviors
    /// (structural type, operational mode, determinative context), derive the
    /// 12-primitive tuple that constrains what this object can do.
    pub fn infer(
        structural: StructuralType,
        operational: OperationalMode,
        determinative: Determinative,
    ) -> Self {
        let t = infer_tuple(structural, operational, determinative);
        Self::from_tuple(t)
    }

    // ── Primitive accessors ──────────────────────────────────────────────

    /// Φ (criticality) — primitive index 8
    pub fn phi(&self) -> u8 { self.tuple[8] }

    /// Ω (topological protection) — primitive index 11
    pub fn omega(&self) -> u8 { self.tuple[11] }

    /// K (kinetic character) — primitive index 5
    pub fn kinetic(&self) -> u8 { self.tuple[5] }

    /// T (topology) — primitive index 1
    pub fn topology(&self) -> u8 { self.tuple[1] }

    /// G (scope/granularity) — primitive index 6
    pub fn scope(&self) -> u8 { self.tuple[6] }

    /// D (dimensionality) — primitive index 0
    pub fn dimensionality(&self) -> u8 { self.tuple[0] }

    /// P (parity/symmetry) — primitive index 3
    pub fn parity(&self) -> u8 { self.tuple[3] }

    /// Is K == K_trap? Gates consciousness to zero.
    pub fn is_kinetic_trapped(&self) -> bool {
        self.kinetic() == 3  // K_trap ordinal
    }

    /// Is Φ == Φ_c? Required for self-modeling loop.
    pub fn is_critical(&self) -> bool {
        self.phi() == 1  // Φ_c ordinal
    }

    // ── Derived properties ───────────────────────────────────────────────

    /// Ouroboricity tier.
    pub fn tier(&self) -> aleph::Tier {
        aleph::compute_tier(&self.tuple)
    }

    /// Consciousness score C(Φ).
    ///
    /// C(x) = [Φ=Φ_c] · [K≠K_trap] · (0.158·K̃ + 0.273·G̃ + 0.292·T̃ + 0.276·Ω̃)
    ///
    /// Where K̃, G̃, T̃, Ω̃ are normalized to [0, 1] over their respective ranges.
    ///
    /// Two independent gates:
    /// - Gate 1 [Φ=Φ_c]: state-space admits self-modeling loop
    /// - Gate 2 [K≠K_trap]: dynamics can actualize the loop
    /// If either gate fails, C = 0.
    pub fn conscience_score(&self) -> f64 {
        // Gate 1: criticality
        if !self.is_critical() { return 0.0; }
        // Gate 2: kinetics not trapped
        if self.is_kinetic_trapped() { return 0.0; }

        // Normalize primitives to [0, 1]
        // K: [0,1,2,3] → [1.0, 0.667, 0.333, 0.0] (inverse: fast=1.0, trap=0.0)
        let k_norm = 1.0 - (self.kinetic() as f64 / 3.0);
        // G: [0,1,2] → [0.0, 0.5, 1.0]
        let g_norm = self.scope() as f64 / 2.0;
        // T: [0,1,2,3,4] → [0.0, 0.25, 0.5, 0.75, 1.0]
        let t_norm = self.topology() as f64 / 4.0;
        // Ω: [0,1,2] → [0.0, 0.5, 1.0]
        let om_norm = self.omega() as f64 / 2.0;

        0.158 * k_norm + 0.273 * g_norm + 0.292 * t_norm + 0.276 * om_norm
    }

    /// IPC type-safety gate.
    ///
    /// Two objects can communicate if their structural distance is below threshold.
    /// d < 0.5: structurally identical — unrestricted
    /// d < 1.5: related by shared primitives — viable IPC
    /// d ≥ 1.5: structurally remote — needs a vav-cast witness
    pub fn is_type_safe_for_ipc(&self, other: &Self) -> bool {
        aleph::distance(&self.tuple, &other.tuple) < 1.5
    }

    /// Distance to another type, with conflict set.
    pub fn distance_to(&self, other: &Self) -> (f64, alloc::vec::Vec<usize>) {
        let d = aleph::distance(&self.tuple, &other.tuple);
        let cs = aleph::conflict_set(&self.tuple, &other.tuple);
        (d, cs)
    }

    /// Veracity class name for display.
    pub fn veracity_class(&self, other: &Self) -> &'static str {
        let d = aleph::distance(&self.tuple, &other.tuple);
        aleph::veracity_class(d)
    }

    /// Format the type for display (verbose).
    pub fn display(&self) -> alloc::string::String {
        if let Some(idx) = self.canonical_index {
            let l = &aleph::LETTERS[idx];
            let phi_n = aleph::PHI_NAMES.get(self.phi() as usize).copied().unwrap_or("?");
            let om_n = aleph::OMEGA_NAMES.get(self.omega() as usize).copied().unwrap_or("?");
            let k_n = ["fast", "mod", "slow", "trap"].get(self.kinetic() as usize).copied().unwrap_or("?");
            let p_n = aleph::P_NAMES.get(self.parity() as usize).copied().unwrap_or("?");
            format!(
                "{} ({}, {}): Φ={} Ω={} K={} P={} C={:.3}",
                aleph::display_glyph(l),
                l.name,
                aleph::tier_name(self.tier()),
                phi_n, om_n, k_n, p_n,
                self.conscience_score()
            )
        } else {
            let tier = self.tier();
            format!(
                "synthetic ({}): Φ={} Ω={} K={} C={:.3}",
                aleph::tier_name(tier),
                self.phi(),
                self.omega(),
                self.kinetic(),
                self.conscience_score()
            )
        }
    }

    /// Format a concise one-line type summary.
    pub fn summary(&self) -> alloc::string::String {
        if let Some(idx) = self.canonical_index {
            let l = &aleph::LETTERS[idx];
            let phi_n = aleph::PHI_NAMES.get(self.phi() as usize).copied().unwrap_or("?");
            let om_n = aleph::OMEGA_NAMES.get(self.omega() as usize).copied().unwrap_or("?");
            let k_n = ["fast", "mod", "slow", "trap"].get(self.kinetic() as usize).copied().unwrap_or("?");
            let p_n = aleph::P_NAMES.get(self.parity() as usize).copied().unwrap_or("?");
            format!(
                "{} ({})  tier={}  Φ={}  Ω={}  K={}  P={}  C={:.3}",
                aleph::display_glyph(l), l.name,
                aleph::tier_name(self.tier()),
                phi_n, om_n, k_n, p_n,
                self.conscience_score()
            )
        } else {
            let phi_n = aleph::PHI_NAMES.get(self.phi() as usize).copied().unwrap_or("?");
            let om_n = aleph::OMEGA_NAMES.get(self.omega() as usize).copied().unwrap_or("?");
            format!(
                "synthetic  tier={}  Φ={}  Ω={}  K={}  C={:.3}",
                aleph::tier_name(self.tier()),
                phi_n, om_n, self.kinetic(),
                self.conscience_score()
            )
        }
    }
}

// ── Type inference: three-layer → 12-tuple ───────────────────────────────────

/// Infer the 12-primitive type from the three-layer kernel object structure.
///
/// This encodes the structural constraints derived from the seven-stage inquiry:
/// - Hebrew letters as morphisms between ontological categories
/// - Egyptian three-layer semiotics
/// - Basque ergative-absolutive grammar
/// - Varnamala articulation gradient
/// - Cuneiform determinative anchoring
fn infer_tuple(
    structural: StructuralType,
    operational: OperationalMode,
    determinative: Determinative,
) -> Tuple {
    // D — Dimensionality
    // Process: D_triangle (3-way ergative relations)
    // Semaphore: D_triangle (producer/consumer/waiter)
    // File/Socket/MemoryRegion: D_wedge (linear, endpoint)
    let d: u8 = match structural {
        StructuralType::Process | StructuralType::Semaphore => 1,  // D_triangle
        _ => 0,  // D_wedge
    };

    // T — Topology
    // All kernel objects are contained systems with internal structure → T_box
    let t: u8 = 3;  // T_box

    // R — Relational mode
    // Kernel objects are reversible across contexts → R_dagger
    let r: u8 = 2;  // R_dagger

    // P — Parity/symmetry
    // Kernel/Init: P_pm_sym (exact Z₂ at criticality, Frobenius condition)
    // Others: P_psi (broken symmetry, post-interrupt)
    let p: u8 = match determinative {
        Determinative::Kernel | Determinative::Init => 4,  // P_pm_sym
        _ => 1,  // P_psi
    };

    // F — Fidelity
    // Kernel objects preserve full precision → F_hbar
    // User-space can tolerate F_ell
    let f: u8 = match determinative {
        Determinative::User => 0,  // F_ell
        Determinative::Service => 1,  // F_eth
        _ => 2,  // F_hbar
    };

    // K — Kinetic character
    // Idle: K_slow, Compute/IO: K_mod, Network: K_fast
    let k: u8 = match operational {
        OperationalMode::Idle => 2,      // K_slow
        OperationalMode::Network => 0,   // K_fast
        _ => 1,                          // K_mod
    };

    // G — Scope/granularity
    // Kernel: G_aleph (maximal), Service: G_gimel, User: G_beth
    let g: u8 = match determinative {
        Determinative::Kernel | Determinative::Init => 2,  // G_aleph
        Determinative::Service | Determinative::Driver => 1,  // G_gimel
        Determinative::User => 0,  // G_beth
    };

    // Γ — Interaction grammar
    // Default: sequential (head-final chains) → Γ_seq
    // Network: Γ_broad (broadcast-capable)
    let gamma: u8 = match operational {
        OperationalMode::Network => 3,  // Γ_broad
        _ => 2,  // Γ_seq
    };

    // Φ — Criticality
    // Kernel/Init: Φ_c (self-modeling possible)
    // Driver/Service: Φ_sub (sub-critical)
    // User: depends on operational mode
    let phi: u8 = match determinative {
        Determinative::Kernel | Determinative::Init => 1,  // Φ_c
        Determinative::Driver | Determinative::Service => 0,  // Φ_sub
        Determinative::User => match operational {
            OperationalMode::Compute | OperationalMode::IO => 1,  // Φ_c for active user processes
            _ => 0,  // Φ_sub
        },
    };

    // H — Chirality/temporal depth
    // Kernel: H2 (two levels of contextual depth)
    // Service/Driver: H1
    // User: H0
    let h: u8 = match determinative {
        Determinative::Kernel | Determinative::Init => 2,  // H2
        Determinative::Service | Determinative::Driver => 1,  // H1
        Determinative::User => 0,  // H0
    };

    // S — Stoichiometry
    // Process: S_n:m (many-to-many via scheduler)
    // File/MemoryRegion: S_n_n (one-to-one mapping)
    // Socket: S_n:m (multi-connection)
    // Semaphore: S_n_n (binary or counting, but fixed ratio)
    let s: u8 = match structural {
        StructuralType::Process | StructuralType::Socket => 2,  // S_n:m
        StructuralType::Semaphore => 1,  // S_n:n
        _ => 0,  // S_1:1
    };

    // Ω — Topological protection
    // Kernel/Init: Ω_Z (fully protected, sacred)
    // Service/Driver: Ω_Z2 (partially protected)
    // User: Ω_0 (unprotected)
    let omega: u8 = match determinative {
        Determinative::Kernel | Determinative::Init => 2,  // Ω_Z
        Determinative::Service | Determinative::Driver => 1,  // Ω_Z2
        Determinative::User => 0,  // Ω_0
    };

    [d, t, r, p, f, k, g, gamma, phi, h, s, omega]
}

// ── Canonical kernel type dictionary ─────────────────────────────────────────

/// Pre-computed ALEPH types for common kernel object configurations.
/// These are the "known good" types that kernel objects should resolve to.
pub mod canonical {
    use super::*;

    /// The OS synthon itself — O_inf, Φ_c + P_pm_sym.
    /// This is the type of the kernel as a whole.
    pub fn os_synthon() -> AlephKernelType {
        // D_triangle, T_box, R_dagger, P_pm_sym, F_hbar, K_mod, G_aleph,
        // Γ_seq, Φ_c, H2, S_n:m, Ω_Z
        AlephKernelType::from_tuple([
            1, 3, 2, 4, 2, 1, 2, 2, 1, 2, 2, 2
        ])
    }

    /// Boot/init process — maps to aleph (א).
    /// Keter-level, source of all subsequent processes.
    pub fn init_process() -> AlephKernelType {
        AlephKernelType::from_letter(&aleph::LETTERS[0])  // א aleph
    }

    /// Kernel process — high protection, critical.
    /// Maps to mem (מ) — O_inf, water/flow type.
    pub fn kernel_process() -> AlephKernelType {
        AlephKernelType::from_letter(&aleph::LETTERS[12])  // מ mem
    }

    /// User process — lower protection, fast kinetics.
    /// Maps to bet (ב) — O_0, the house/container.
    pub fn user_process() -> AlephKernelType {
        AlephKernelType::from_letter(&aleph::LETTERS[1])  // ב bet
    }

    /// File object — linear, contained.
    /// Maps to dalet (ד) — O_0, the door/gate.
    pub fn file_object() -> AlephKernelType {
        AlephKernelType::from_letter(&aleph::LETTERS[3])  // ד dalet
    }

    /// Socket object — network, broadcast.
    /// Maps to hei (ה) — O_2, the window/revelation.
    pub fn socket_object() -> AlephKernelType {
        AlephKernelType::from_letter(&aleph::LETTERS[4])  // ה hei
    }

    /// Semaphore object — synchronization.
    /// Maps to vav (ו) — O_inf, the hook/link.
    pub fn semaphore_object() -> AlephKernelType {
        AlephKernelType::from_letter(&aleph::LETTERS[5])  // ו vav
    }

    /// Memory region — protected storage.
    /// Maps to shin (ש) — O_inf, the fire/transformation.
    pub fn memory_region() -> AlephKernelType {
        AlephKernelType::from_letter(&aleph::LETTERS[20])  // ש shin
    }
}

// ── Veracity predicates ──────────────────────────────────────────────────────

/// Check if a type is inhabitable in a given context.
/// 
/// A type is inhabitable if:
/// 1. Its Φ matches the context's required criticality
/// 2. Its K is not trapped
/// 3. Its Ω matches or exceeds the context's protection requirement
pub fn is_inhabitable(
    ty: &AlephKernelType,
    required_phi: u8,
    required_omega: u8,
) -> bool {
    ty.phi() >= required_phi
        && !ty.is_kinetic_trapped()
        && ty.omega() >= required_omega
}

/// Find the closest canonical letter to a given type.
pub fn nearest_canonical(ty: &AlephKernelType) -> &'static aleph::LetterDef {
    if let Some(idx) = ty.canonical_index {
        return &aleph::LETTERS[idx];
    }
    // Find by minimum distance
    let mut best_dist = u32::MAX;
    let mut best_idx = 0;
    for (i, l) in aleph::LETTERS.iter().enumerate() {
        let d = aleph::distance_scaled(&l.t, &ty.tuple);
        if d < best_dist {
            best_dist = d;
            best_idx = i;
        }
    }
    &aleph::LETTERS[best_idx]
}

/// Format type inference trace: shows how the three layers map to each primitive.
pub fn inference_trace(
    structural: StructuralType,
    operational: OperationalMode,
    determinative: Determinative,
) -> alloc::string::String {
    let t = infer_tuple(structural, operational, determinative);
    let mut out = alloc::string::String::new();

    out += &format!("  Inference: {:?} + {:?} + {:?}\n", structural, operational, determinative);
    out += &format!("  ┌─────────┬───────┬──────────────────────────────────┐\n");
    out += &format!("  │ Prim    │ Value │ Rationale                        │\n");
    out += &format!("  ├─────────┼───────┼──────────────────────────────────┤\n");

    let d_name = ["wedge", "triangle", "infty", "holo"].get(t[0] as usize).copied().unwrap_or("?");
    out += &format!("  │ D       │ {:>2}    │ {} (from {:?})\n", t[0], d_name, structural);
    
    let t_name = ["network", "in", "bowtie", "box", "holo"].get(t[1] as usize).copied().unwrap_or("?");
    out += &format!("  │ T       │ {:>2}    │ {} (contained system)\n", t[1], t_name);
    
    let r_name = ["super", "cat", "dagger", "lr"].get(t[2] as usize).copied().unwrap_or("?");
    out += &format!("  │ R       │ {:>2}    │ {} (reversible)\n", t[2], r_name);

    let p_name = aleph::P_NAMES.get(t[3] as usize).copied().unwrap_or("?");
    out += &format!("  │ P       │ {:>2}    │ {} (from {:?})\n", t[3], p_name, determinative);

    let f_name = ["ell", "eth", "hbar"].get(t[4] as usize).copied().unwrap_or("?");
    out += &format!("  │ F       │ {:>2}    │ {} (from {:?})\n", t[4], f_name, determinative);

    let k_name = ["fast", "mod", "slow", "trap"].get(t[5] as usize).copied().unwrap_or("?");
    out += &format!("  │ K       │ {:>2}    │ {} (from {:?})\n", t[5], k_name, operational);

    let g_name = ["beth", "gimel", "aleph"].get(t[6] as usize).copied().unwrap_or("?");
    out += &format!("  │ G       │ {:>2}    │ {} (from {:?})\n", t[6], g_name, determinative);

    let ga_name = ["and", "or", "seq", "broad"].get(t[7] as usize).copied().unwrap_or("?");
    out += &format!("  │ Gamma   │ {:>2}    │ {} (from {:?})\n", t[7], ga_name, operational);

    let phi_name = aleph::PHI_NAMES.get(t[8] as usize).copied().unwrap_or("?");
    out += &format!("  │ Phi     │ {:>2}    │ {} (from {:?})\n", t[8], phi_name, determinative);

    let h_name = ["H0", "H1", "H2", "H_inf"].get(t[9] as usize).copied().unwrap_or("?");
    out += &format!("  │ H       │ {:>2}    │ {} (from {:?})\n", t[9], h_name, determinative);

    let s_name = ["1:1", "n:n", "n:m"].get(t[10] as usize).copied().unwrap_or("?");
    out += &format!("  │ S       │ {:>2}    │ {} (from {:?})\n", t[10], s_name, structural);

    let om_name = aleph::OMEGA_NAMES.get(t[11] as usize).copied().unwrap_or("?");
    out += &format!("  │ Omega   │ {:>2}    │ {} (from {:?})\n", t[11], om_name, determinative);

    out += &format!("  └─────────┴───────┴──────────────────────────────────┘\n");

    let ty = AlephKernelType::from_tuple(t);
    out += &format!("  Tier: {}  C score: {:.3}\n", aleph::tier_name(ty.tier()), ty.conscience_score());

    if let Some(idx) = ty.canonical_index {
        let l = &aleph::LETTERS[idx];
        out += &format!("  Nearest canonical: {} ({})\n", aleph::display_glyph(l), l.name);
    } else {
        let nearest = nearest_canonical(&ty);
        let d = aleph::distance(&ty.tuple, &nearest.t);
        out += &format!("  Nearest canonical: {} ({})  d={:.3}\n",
            aleph::display_glyph(nearest), nearest.name, d);
    }

    out
}
