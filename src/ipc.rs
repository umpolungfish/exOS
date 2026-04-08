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

use crate::kernel_object::{Determinative, StructuralType};

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

/// Three-layer IPC message
#[derive(Debug, Clone)]
pub struct IpcMessage {
    /// Layer 1: Structural signature (logogram — what type)
    pub structural: StructuralSignature,
    /// Layer 2: Payload (phonogram — the actual data)
    pub payload: &'static [u8],
    /// Layer 3: Determinative (semantic context — unpronounced but load-bearing)
    pub determinative: MessageDeterminative,
}

impl IpcMessage {
    pub fn new(
        structural: StructuralSignature,
        payload: &'static [u8],
        determinative: MessageDeterminative,
    ) -> Self {
        Self {
            structural,
            payload,
            determinative,
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

    /// The raw payload length
    pub fn len(&self) -> usize {
        self.payload.len()
    }

    pub fn is_empty(&self) -> bool {
        self.payload.is_empty()
    }
}
