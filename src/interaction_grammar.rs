//! Interaction Grammar (Gamma) for IPC.
//!
//! Gamma governs the composition logic of messages.
//! Gamma_seq (2): sequential, ordered packet delivery.
//! Gamma_broad (3): multicast/broadcast support.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InteractionGrammar {
    /// Sequential, ordered steps
    Sequential = 2,
    /// One-to-all broadcast
    Broadcast = 3,
}

impl InteractionGrammar {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Sequential => "Gamma_seq",
            Self::Broadcast => "Gamma_broad",
        }
    }
}
