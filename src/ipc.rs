//! Inter-Process Communication (from Egyptian hieroglyphs).
//!
//! IPC messages carry THREE layers simultaneously — exactly as hieroglyphic
//! signs encode logogram + phonogram + determinative:
//!
//! | Layer           | Hieroglyph analog  | Purpose                               |
//! |-----------------|---------------------|---------------------------------------|
//! | Structural sig  | Logogram            | What TYPE of object is being passed   |
//! | Payload         | Phonogram           | The actual data (the "pronounced" part) |
//! | Determinative   | Determinative       | Runtime context that disambiguates the payload's meaning |
//!
//! A message WITHOUT a determinative layer is **syntactically malformed**.
//! This prevents type confusion attacks at the protocol level.
//! The same payload means different things under different determinatives
//! (exactly as a hieroglyphic sign means different things with different
//! unpronounced semantic classifiers).
//!
//! With the ALEPH type bridge, messages also carry the 12-primitive types
//! of source and target objects, enabling **type-gated IPC**:
//! - d < 0.5: structurally identical — unrestricted
//! - d < 1.5: related by shared primitives — viable IPC
//! - d ≥ 1.5: structurally remote — needs a vav-cast witness to mediate

use crate::kernel_object::{Determinative, StructuralType};
use crate::aleph_kernel_types::AlephKernelType;

/// Structural signature — what type of object is being passed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructuralSignature {
    pub source_type: StructuralType,
    pub target_type: StructuralType,
}

/// The determinative context — disambiguates the payload
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageDeterminative {
    /// Source determinative: the semantic context of the sender
    pub source_ctx: Determinative,
    /// Target determinative: the expected semantic context of the receiver
    pub target_ctx: Determinative,
}

/// ALEPH type gate result — why a message was accepted or rejected.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeGateResult {
    /// Accepted — types are within safe distance
    Accepted { distance: f64, class: &'static str },
    /// Rejected — types are too far apart
    Rejected { distance: f64, class: &'static str, reason: &'static str },
    /// No type information present — fall through to structural check only
    NoTypeInfo,
}

impl TypeGateResult {
    pub fn is_accepted(&self) -> bool {
        matches!(self, TypeGateResult::Accepted { .. })
    }
}

/// Vav-cast witness — a mediating ALEPH type that enables communication
/// between structurally remote objects (d ≥ 1.5).
///
/// The witness must:
/// 1. Be within safe distance of both source and target (d < 1.5 each)
/// 2. Have ouroboricity tier ≥ O_1 (can sustain a self-modeling loop)
///
/// This is the Hebrew letter Vav (ו) principle: the hook that links
/// heaven and earth, enabling communication across the gap.
#[derive(Debug, Clone)]
pub struct IpcWitness {
    /// The mediating ALEPH type
    pub witness_type: AlephKernelType,
}

impl IpcWitness {
    pub fn new(witness_type: AlephKernelType) -> Self {
        Self { witness_type }
    }

    /// Validate that this witness can mediate between source and target.
    /// Returns true if the witness is within safe distance of both ends
    /// and has sufficient tier.
    pub fn is_valid(
        &self,
        source_type: &AlephKernelType,
        target_type: &AlephKernelType,
    ) -> bool {
        use crate::aleph::Tier;
        let tier = self.witness_type.tier();
        // Witness must have at least O_1 (critical loop possible, even if unprotected)
        if matches!(tier, Tier::O0) {
            return false;
        }
        // Witness must be within safe distance of both source and target
        let d_source = crate::aleph::distance(&self.witness_type.tuple, &source_type.tuple);
        let d_target = crate::aleph::distance(&self.witness_type.tuple, &target_type.tuple);
        d_source < 1.5 && d_target < 1.5
    }
}

/// Three-layer IPC message with optional ALEPH type gating
#[derive(Debug, Clone)]
pub struct IpcMessage {
    /// Layer 1: Structural signature (logogram — what type)
    pub structural: StructuralSignature,
    /// Layer 2: Payload (phonogram — the actual data)
    pub payload: &'static [u8],
    /// Layer 3: Determinative (semantic context — unpronounced but load-bearing)
    pub determinative: MessageDeterminative,
    /// ALEPH source type — the 12-primitive type of the sending object
    pub source_aleph_type: Option<AlephKernelType>,
    /// ALEPH target type — the 12-primitive type of the receiving object
    pub target_aleph_type: Option<AlephKernelType>,
    /// Optional vav-cast witness for structurally remote communication
    pub witness: Option<IpcWitness>,
}

impl IpcMessage {
    /// Create a message without ALEPH type information.
    /// Falls back to structural/determinative validation only.
    pub fn new(
        structural: StructuralSignature,
        payload: &'static [u8],
        determinative: MessageDeterminative,
    ) -> Self {
        Self {
            structural,
            payload,
            determinative,
            source_aleph_type: None,
            target_aleph_type: None,
            witness: None,
        }
    }

    /// Create a message with ALEPH type information.
    /// Enables type-gated IPC validation.
    pub fn with_types(
        structural: StructuralSignature,
        payload: &'static [u8],
        determinative: MessageDeterminative,
        source_aleph_type: AlephKernelType,
        target_aleph_type: AlephKernelType,
    ) -> Self {
        Self {
            structural,
            payload,
            determinative,
            source_aleph_type: Some(source_aleph_type),
            target_aleph_type: Some(target_aleph_type),
            witness: None,
        }
    }

    /// Create a message with a vav-cast witness for structurally remote communication.
    pub fn with_witness(
        structural: StructuralSignature,
        payload: &'static [u8],
        determinative: MessageDeterminative,
        source_aleph_type: AlephKernelType,
        target_aleph_type: AlephKernelType,
        witness: IpcWitness,
    ) -> Self {
        Self {
            structural,
            payload,
            determinative,
            source_aleph_type: Some(source_aleph_type),
            target_aleph_type: Some(target_aleph_type),
            witness: Some(witness),
        }
    }

    /// Validate: a message without a determinative is syntactically malformed.
    /// Additionally, the determinative must be consistent with the structural types.
    pub fn is_well_formed(&self) -> bool {
        // The determinative must be consistent with the source structural type
        let structural_valid = match (self.structural.source_type, self.determinative.source_ctx) {
            (StructuralType::Process, Determinative::Kernel) => true,
            (StructuralType::Process, Determinative::User) => true,
            (StructuralType::Process, Determinative::Service) => true,
            (StructuralType::Process, Determinative::Init) => true,
            (StructuralType::File, _) => true,
            (StructuralType::Socket, _) => true,
            (StructuralType::Semaphore, _) => true,
            (StructuralType::MemoryRegion, _) => true,
            _ => false,
        };
        structural_valid
    }

    /// Type-gate validation — the ALEPH type system constrains IPC.
    ///
    /// If ALEPH types are present, validates structural distance:
    /// - d < 0.5: "transparent" — structurally identical, unrestricted
    /// - d < 1.5: "near-grounded" — related by shared primitives, viable IPC
    /// - d ≥ 1.5: "aspirational" — structurally remote, requires vav-cast witness
    ///
    /// If types are not present, returns NoTypeInfo (fall through to structural check).
    pub fn is_type_valid(&self) -> TypeGateResult {
        let (source, target) = match (&self.source_aleph_type, &self.target_aleph_type) {
            (Some(s), Some(t)) => (s, t),
            _ => return TypeGateResult::NoTypeInfo,
        };

        let d = crate::aleph::distance(&source.tuple, &target.tuple);
        let vc = crate::aleph::veracity_class(d);

        if d < 1.5 {
            TypeGateResult::Accepted { distance: d, class: vc }
        } else if d >= 1.5 {
            // Check if a vav-cast witness is present and valid
            if let Some(ref witness) = self.witness {
                if witness.is_valid(source, target) {
                    return TypeGateResult::Accepted { distance: d, class: "witnessed" };
                }
            }
            TypeGateResult::Rejected {
                distance: d,
                class: vc,
                reason: "structurally remote, no valid vav-cast witness",
            }
        } else {
            TypeGateResult::Accepted { distance: d, class: vc }
        }
    }

    /// The raw payload length
    pub fn len(&self) -> usize {
        self.payload.len()
    }

    pub fn is_empty(&self) -> bool {
        self.payload.is_empty()
    }
}
