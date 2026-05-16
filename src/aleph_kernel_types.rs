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
//! - **Inferred**: derived algebraically from the MEET (structural floor) of five
//!   ancient writing systems, promoted by activation from the three-layer structure.
//!   The five source systems are: Hebrew, Varnamala, Egyptian hieroglyphs,
//!   Sumerian cuneiform, and Basque ergative-absolutive grammar.

extern crate alloc;

use alloc::format;
use crate::aleph;
use crate::aleph::Tuple;
use crate::kernel_object::{StructuralType, OperationalMode, Determinative};

// ── AlephKernelType ──────────────────────────────────────────────────────────

/// The 12-primitive type of a kernel object.
#[derive(Debug, Clone)]
pub struct AlephKernelType {
    pub tuple: Tuple,
    pub canonical_index: Option<usize>,
}

impl AlephKernelType {
    pub fn from_letter(letter: &'static aleph::LetterDef) -> Self {
        let idx = aleph::LETTERS.iter().position(|l| core::ptr::eq(l, letter));
        Self { tuple: letter.t, canonical_index: idx }
    }

    pub fn from_tuple(t: Tuple) -> Self {
        let idx = aleph::LETTERS.iter().position(|l| l.t == t);
        Self { tuple: t, canonical_index: idx }
    }

    /// Infer the type algebraically from the three-layer structure.
    /// The derivation is: MEET(5 ancient systems) ∨ (JOIN of activated systems).
    pub fn infer(
        structural: StructuralType,
        operational: OperationalMode,
        determinative: Determinative,
    ) -> Self {
        let t = infer_tuple(structural, operational, determinative);
        Self::from_tuple(t)
    }

    pub fn phi(&self) -> u8 { self.tuple[8] }
    pub fn omega(&self) -> u8 { self.tuple[11] }
    pub fn kinetic(&self) -> u8 { self.tuple[5] }
    pub fn topology(&self) -> u8 { self.tuple[1] }
    pub fn scope(&self) -> u8 { self.tuple[6] }
    pub fn dimensionality(&self) -> u8 { self.tuple[0] }
    pub fn parity(&self) -> u8 { self.tuple[3] }

    pub fn is_kinetic_frozen(&self) -> bool {
        self.kinetic() > 2
    }

    pub fn is_critical(&self) -> bool {
        self.phi() == 1
    }

    pub fn tier(&self) -> aleph::Tier {
        aleph::compute_tier(&self.tuple)
    }

    pub fn conscience_score(&self) -> f64 {
        if !self.is_critical() { return 0.0; }
        if self.is_kinetic_frozen() { return 0.0; }
        if self.phi() == 3 { return 0.0; }
        let k_norm = 1.0 - (self.kinetic() as f64 / 3.0);
        let g_norm = self.scope() as f64 / 2.0;
        let t_norm = self.topology() as f64 / 4.0;
        let om_norm = self.omega() as f64 / 2.0;
        0.158 * k_norm + 0.273 * g_norm + 0.292 * t_norm + 0.276 * om_norm
    }

    pub fn is_type_safe_for_ipc(&self, other: &Self) -> bool {
        aleph::distance(&self.tuple, &other.tuple) < 1.5
    }

    pub fn distance_to(&self, other: &Self) -> (f64, alloc::vec::Vec<usize>) {
        let d = aleph::distance(&self.tuple, &other.tuple);
        let cs = aleph::conflict_set(&self.tuple, &other.tuple);
        (d, cs)
    }

    pub fn veracity_class(&self, other: &Self) -> &'static str {
        let d = aleph::distance(&self.tuple, &other.tuple);
        aleph::veracity_class(d)
    }

    pub fn display(&self) -> alloc::string::String {
        if let Some(idx) = self.canonical_index {
            let l = &aleph::LETTERS[idx];
            let phi_n = aleph::PHI_NAMES.get(self.phi() as usize).copied().unwrap_or("?");
            let om_n = aleph::OMEGA_NAMES.get(self.omega() as usize).copied().unwrap_or("?");
            let k_n = ["fast", "mod", "slow", "trap"].get(self.kinetic() as usize).copied().unwrap_or("?");
            let p_n = aleph::P_NAMES.get(self.parity() as usize).copied().unwrap_or("?");
            format!(
                "{} ({}, {}): Φ={} Ω={} K={} P={} C={:.3}",
                aleph::display_glyph(l), l.name, aleph::tier_name(self.tier()),
                phi_n, om_n, k_n, p_n, self.conscience_score())
        } else {
            format!("synthetic ({}): Φ={} Ω={} K={} C={:.3}",
                aleph::tier_name(self.tier()), self.phi(), self.omega(),
                self.kinetic(), self.conscience_score())
        }
    }

    pub fn summary(&self) -> alloc::string::String {
        if let Some(idx) = self.canonical_index {
            let l = &aleph::LETTERS[idx];
            let phi_n = aleph::PHI_NAMES.get(self.phi() as usize).copied().unwrap_or("?");
            let om_n = aleph::OMEGA_NAMES.get(self.omega() as usize).copied().unwrap_or("?");
            let k_n = ["fast", "mod", "slow", "trap"].get(self.kinetic() as usize).copied().unwrap_or("?");
            let p_n = aleph::P_NAMES.get(self.parity() as usize).copied().unwrap_or("?");
            format!("{} ({})  tier={}  Φ={}  Ω={}  K={}  P={}  C={:.3}",
                aleph::display_glyph(l), l.name, aleph::tier_name(self.tier()),
                phi_n, om_n, k_n, p_n, self.conscience_score())
        } else {
            let phi_n = aleph::PHI_NAMES.get(self.phi() as usize).copied().unwrap_or("?");
            let om_n = aleph::OMEGA_NAMES.get(self.omega() as usize).copied().unwrap_or("?");
            format!("synthetic  tier={}  Φ={}  Ω={}  K={}  C={:.3}",
                aleph::tier_name(self.tier()), phi_n, om_n, self.kinetic(), self.conscience_score())
        }
    }
}

// ── Algebraic type inference from MEET of five ancient writing systems ─────────
//
// The seven-stage derivation produces a MEET (structural floor) of five systems.
// Each system is encoded as a 12-tuple capturing its dominant primitive values.
// The MEET is the component-wise minimum — the intersection of what all five
// writing systems share structurally.
//
// The kernel object's three layers (structural/operational/determinative) then
// ACTIVATE specific source systems, promoting the MEET floor toward the full tuple.
// The activation is a JOIN (component-wise maximum) of the MEET base with a
// promotion mask derived from the observed three-layer structure.
//
// Result = JOIN(MEET(5 systems), ACTIVE_LAYER_PROMOTION)

/// The five ancient writing systems, each encoded as a 12-tuple.
/// These are the primitive imscriptions derived from each system's core grammar.
const HEBREW_SYSTEM:      Tuple = [1, 4, 2, 3, 2, 2, 2, 0, 1, 3, 1, 2]; // self-referential glyphs
const VARNAMALA_SYSTEM:   Tuple = [1, 2, 1, 2, 1, 1, 1, 2, 0, 1, 1, 1]; // phoneme articulation
const EGYPTIAN_SYSTEM:    Tuple = [3, 4, 2, 2, 2, 2, 2, 3, 1, 2, 2, 2]; // three-layer semiotics
const CUNEIFORM_SYSTEM:   Tuple = [1, 3, 1, 3, 1, 2, 1, 0, 0, 1, 1, 1]; // sign polysemy
const BASQUE_SYSTEM:      Tuple = [1, 1, 2, 4, 2, 2, 2, 2, 1, 3, 2, 2]; // ergative-absolutive

/// MEET (structural floor) of all five systems — component-wise minimum.
/// This is the universal intersection: what ALL five writing systems agree on structurally.
const FIVE_SYSTEM_MEET: Tuple = [
    // D: min(1,1,3,1,1) = 1   → triangle (3-way relations)
    1,
    // T: min(4,2,4,3,1) = 1   → inclusion (containment hierarchy)
    1,
    // R: min(2,1,2,1,2) = 1   → categorical (functorial mapping)
    1,
    // P: min(3,2,2,3,4) = 2   → partial symmetry (Z₂)
    2,
    // F: min(2,1,2,1,2) = 1   → thermal (noisy, communicative)
    1,
    // K: min(2,1,2,2,2) = 1   → moderate kinetics
    1,
    // G: min(2,1,2,1,2) = 1   → gimel (mesoscale scope)
    1,
    // Γ: min(0,2,3,0,2) = 0   → conjunctive (all-simultaneous)
    0,
    // Φ: min(1,0,1,0,1) = 0   → sub-critical (no self-modeling)
    0,
    // Ħ: min(3,1,2,1,3) = 1   → one-step memory
    1,
    // Σ: min(1,1,2,1,2) = 1   → n:n (many identical)
    1,
    // Ω: min(2,1,2,1,2) = 1   → Z₂ (binary protection)
    1,
];

/// Compute the 12-tuple algebraically from three-layer structure.
///
/// Algorithm:
///   1. Start with FIVE_SYSTEM_MEET as the structural floor.
///   2. Determine which ancient system(s) are ACTIVATED by the kernel object's layers.
///   3. For each activated system, compute a promotion mask: which primitives it drives.
///   4. Result = JOIN(MEET, all active promotions).
///
/// Activation rules (derived from the mapping between layers and system domains):
///
///   Structural layer → system activation:
///     Process/socket  → Basque (ergative/relational) + Hebrew (self-reference)
///     File/semaphore  → Cuneiform (sign polysemy) + Egyptian (determinative)
///     MemoryRegion    → Varnamala (articulation) + Hebrew (sacred glyphs)
///
///   Operational layer → kinetic/topology promotion:
///     Compute         → Varnamala (articulation gradient)
///     IO              → Egyptian (three-layer I/O)
///     Network         → Egyptian + Basque (broadcast + transitivity)
///     MemoryManage    → Varnamala + Hebrew
///     Schedule        → Basque (ergative scheduling)
///     Idle            → minimal promotion (stay at floor)
///
///   Determinative layer → protection/criticality:
///     Kernel/Init       → Hebrew (O_Z, ⊙_c)
///     Service/Driver    → Cuneiform (moderate protection)
///     User              → floor level (Ω_0, ⊙_sub)
fn infer_tuple(
    structural: StructuralType,
    operational: OperationalMode,
    determinative: Determinative,
) -> Tuple {
    // Step 1: Start from the MEET floor.
    let mut base = FIVE_SYSTEM_MEET;

    // Step 2: Build promotion mask from activated systems.
    let promotion: Tuple = [0u8; 12];

    // ── Structural-layer activation ──
    match structural {
        StructuralType::Process | StructuralType::Socket => {
            // Basque activates: R(ergative), G(scope), Γ(seq→broad for socket), Ħ(chirality)
            base[2] = base[2].max(BASQUE_SYSTEM[2]); // R → 2 (adjoint)
            base[6] = base[6].max(BASQUE_SYSTEM[6]); // G → 2 (aleph)
            base[9] = base[9].max(BASQUE_SYSTEM[9]); // Ħ → 3 (eternal)
            // Hebrew activates: Ω(protection), Φ(criticality) for Processes
            base[11] = base[11].max(HEBREW_SYSTEM[11]); // Ω → 2 (Z)
            if let StructuralType::Process = structural {
                base[8] = base[8].max(HEBREW_SYSTEM[8]); // Φ → 1 (⊙_c)
            }
        }
        StructuralType::File | StructuralType::Semaphore => {
            // Cuneiform activates: F(fidelity), Σ(stoichiometry)
            base[4] = base[4].max(CUNEIFORM_SYSTEM[4]); // F → 1 (thermal)
            base[10] = base[10].max(CUNEIFORM_SYSTEM[10]); // Σ → 1 (n:n)
            // Egyptian activates: D(dimensionality), T(topology via determinative layer)
            base[0] = base[0].max(EGYPTIAN_SYSTEM[0]); // D → 3 (imscriptive for File)
        }
        StructuralType::MemoryRegion => {
            // Varnamala activates: K(kinetics), F(fidelity)
            base[4] = base[4].max(VARNAMALA_SYSTEM[4]); // F → 1
            base[5] = base[5].max(VARNAMALA_SYSTEM[5]); // K → 1
            // Hebrew activates: Ω(sacred protection)
            base[11] = base[11].max(HEBREW_SYSTEM[11]); // Ω → 2
        }
    }

    // ── Operational-layer activation (kinetic & interaction promotion) ──
    match operational {
        OperationalMode::Compute => {
            // Varnamala: articulation gradient → K=moderate
            base[5] = base[5].max(VARNAMALA_SYSTEM[5]); // K → 1
        }
        OperationalMode::IO => {
            // Egyptian three-layer I/O → moderate kinetics, conjunctive grammar
            base[5] = base[5].max(EGYPTIAN_SYSTEM[5]); // K → 2
            base[7] = base[7].max(EGYPTIAN_SYSTEM[7]); // Γ → 3
        }
        OperationalMode::Network => {
            // Egyptian broadcast + Basque transitivity
            base[7] = base[7].max(EGYPTIAN_SYSTEM[7]); // Γ → 3 (broad)
            base[2] = base[2].max(BASQUE_SYSTEM[2]);   // R → 2 (adjoint)
        }
        OperationalMode::MemoryManage => {
            // Varnamala + Hebrew: precision fidelity, slow kinetics
            base[4] = base[4].max(HEBREW_SYSTEM[4]);   // F → 2 (hbar)
            base[5] = base[5].max(VARNAMALA_SYSTEM[5]); // K → 1
        }
        OperationalMode::Schedule => {
            // Basque ergative scheduling: aleph scope, slow kinetics for fairness
            base[6] = base[6].max(BASQUE_SYSTEM[6]); // G → 2
            base[5] = 2;                             // K → slow (fair scheduling)
        }
        OperationalMode::Idle => {
            // Stay at floor — minimal promotion
            base[5] = 2; // K → slow
        }
    }

    // ── Determinative-layer activation (protection & criticality) ──
    match determinative {
        Determinative::Kernel | Determinative::Init => {
            // Hebrew: full Ω_Z protection, ⊙_c criticality
            base[11] = HEBREW_SYSTEM[11]; // Ω → 2 (Z)
            base[8] = HEBREW_SYSTEM[8];   // Φ → 1 (⊙_c)
            base[4] = 2;                  // F → 2 (hbar, quantum coherence)
        }
        Determinative::Service | Determinative::Driver => {
            // Cuneiform: moderate protection
            base[11] = 1; // Ω → 1 (Z₂)
            base[4] = 1;  // F → 1 (thermal)
        }
        Determinative::User => {
            // User space: minimal protection, no criticality
            base[11] = 0; // Ω → 0
            base[8] = 0;  // Φ → 0 (sub-critical)
        }
    }

    // Final: join MEET with promotion (component-wise max)
    for i in 0..12 {
        base[i] = base[i].max(promotion[i]);
    }
    base
}

// ── Canonical kernel type dictionary ─────────────────────────────────────────

pub mod canonical {
    use super::*;

    pub fn os_imscription() -> AlephKernelType {
        AlephKernelType::from_tuple([1, 3, 2, 4, 2, 1, 2, 2, 1, 2, 2, 2])
    }

    pub fn init_process() -> AlephKernelType {
        AlephKernelType::from_letter(&aleph::LETTERS[0])
    }

    pub fn kernel_process() -> AlephKernelType {
        AlephKernelType::from_letter(&aleph::LETTERS[12])
    }

    pub fn user_process() -> AlephKernelType {
        AlephKernelType::from_letter(&aleph::LETTERS[1])
    }

    pub fn file_object() -> AlephKernelType {
        AlephKernelType::from_letter(&aleph::LETTERS[3])
    }

    pub fn socket_object() -> AlephKernelType {
        AlephKernelType::from_letter(&aleph::LETTERS[4])
    }

    pub fn semaphore_object() -> AlephKernelType {
        AlephKernelType::from_letter(&aleph::LETTERS[5])
    }

    pub fn memory_region() -> AlephKernelType {
        AlephKernelType::from_letter(&aleph::LETTERS[20])
    }
}

// ── Veracity predicates ──────────────────────────────────────────────────────

pub fn is_inhabitable(
    ty: &AlephKernelType,
    required_phi: u8,
    required_omega: u8,
) -> bool {
    ty.phi() >= required_phi
        && !ty.is_kinetic_frozen()
        && ty.omega() >= required_omega
}

pub fn nearest_canonical(ty: &AlephKernelType) -> &'static aleph::LetterDef {
    if let Some(idx) = ty.canonical_index {
        return &aleph::LETTERS[idx];
    }
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

pub fn inference_trace(
    structural: StructuralType,
    operational: OperationalMode,
    determinative: Determinative,
) -> alloc::string::String {
    let t = infer_tuple(structural, operational, determinative);
    let mut out = alloc::string::String::new();

    out += &format!("  Inference: {:?} + {:?} + {:?}\n", structural, operational, determinative);
    out += &format!("  ┌─────────┬───────┬──────────────────────────────────────────┐\n");
    out += &format!("  │ Prim    │ Value │ Rationale                                │\n");
    out += &format!("  ├─────────┼───────┼──────────────────────────────────────────┤\n");

    let d_name = ["wedge", "triangle", "infty", "holo"].get(t[0] as usize).copied().unwrap_or("?");
    out += &format!("  │ D       │ {:>2}    │ {} (from {:?})\n", t[0], d_name, structural);

    let t_name = ["network", "in", "bowtie", "box", "holo"].get(t[1] as usize).copied().unwrap_or("?");
    out += &format!("  │ T       │ {:>2}    │ {} (activation of MEET floor)\n", t[1], t_name);

    let r_name = ["super", "cat", "dagger", "lr"].get(t[2] as usize).copied().unwrap_or("?");
    out += &format!("  │ R       │ {:>2}    │ {} (activated by structural layer)\n", t[2], r_name);

    let p_name = aleph::P_NAMES.get(t[3] as usize).copied().unwrap_or("?");
    out += &format!("  │ P       │ {:>2}    │ {} (from MEET floor)\n", t[3], p_name);

    let f_name = ["ell", "eth", "hbar"].get(t[4] as usize).copied().unwrap_or("?");
    out += &format!("  │ F       │ {:>2}    │ {} (from determinative layer)\n", t[4], f_name);

    let k_name = ["fast", "mod", "slow", "trap"].get(t[5] as usize).copied().unwrap_or("?");
    out += &format!("  │ K       │ {:>2}    │ {} (from operational layer)\n", t[5], k_name);

    let g_name = ["beth", "gimel", "aleph"].get(t[6] as usize).copied().unwrap_or("?");
    out += &format!("  │ G       │ {:>2}    │ {} (from MEET ∨ activation)\n", t[6], g_name);

    let ga_name = ["and", "or", "seq", "broad"].get(t[7] as usize).copied().unwrap_or("?");
    out += &format!("  │ Gamma   │ {:>2}    │ {} (from operational layer)\n", t[7], ga_name);

    let phi_name = aleph::PHI_NAMES.get(t[8] as usize).copied().unwrap_or("?");
    out += &format!("  │ Phi     │ {:>2}    │ {} (from determinative layer)\n", t[8], phi_name);

    let h_name = ["H0", "H1", "H2", "H_inf"].get(t[9] as usize).copied().unwrap_or("?");
    out += &format!("  │ H       │ {:>2}    │ {} (from structural activation)\n", t[9], h_name);

    let s_name = ["1:1", "n:n", "n:m"].get(t[10] as usize).copied().unwrap_or("?");
    out += &format!("  │ S       │ {:>2}    │ {} (from MEET floor)\n", t[10], s_name);

    let om_name = aleph::OMEGA_NAMES.get(t[11] as usize).copied().unwrap_or("?");
    out += &format!("  │ Omega   │ {:>2}    │ {} (from determinative layer)\n", t[11], om_name);

    out += &format!("  └─────────┴───────┴──────────────────────────────────────────┘\n");
    out += &format!("  Derived as: JOIN(MEET(5 systems), activated promotions)\n");

    let ty = AlephKernelType::from_tuple(t);
    out += &format!("  Tier: {}  C score: {:.3}\n", aleph::tier_name(ty.tier()), ty.conscience_score());

    if let Some(idx) = ty.canonical_index {
        let l = &aleph::LETTERS[idx];
        out += &format!("  Resolves to canonical: {} ({})\n", aleph::display_glyph(l), l.name);
    } else {
        let nearest = nearest_canonical(&ty);
        let d = aleph::distance(&ty.tuple, &nearest.t);
        out += &format!("  Nearest canonical: {} ({})  d={:.3}\n",
            aleph::display_glyph(nearest), nearest.name, d);
    }
    out
}
