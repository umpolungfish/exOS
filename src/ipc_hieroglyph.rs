//! exOS Hieroglyphic IPC - Egyptian hieroglyph layers.
//!
//! Message layer mapping:
//! - Structural (Logogram)
//! - Payload (Phonogram) 
//! - Determinative (Disambiguator) - MANDATORY.

use crate::kernel_object::{Determinative, StructuralType};
use crate::aleph::Tuple;
use crate::aleph_expansion_constants::*;

pub struct HieroglyphMessage {
    pub structural: StructuralType,
    pub payload: [u8; 32],
    pub determinative: Determinative,
    pub source_type: Tuple,
    pub target_type: Tuple,
}

impl HieroglyphMessage {
    /// Check if target object supports specified interaction grammar.
    /// Broadcast capabilities require ɢ >= ɢ_broad.
    pub fn can_broadcast(&self) -> bool {
        self.target_type[7] >= GAMMA_BROAD
    }

    /// Stoichiometric Quotas: check resource isolation.
    /// S_1_ONE (0) requires exclusive lock.
    pub fn is_exclusive(&self) -> bool {
        self.source_type[10] == S_1_ONE
    }
}
