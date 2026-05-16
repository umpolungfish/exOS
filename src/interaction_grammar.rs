//! Interaction Grammar (ɢ) for IPC — primitive index 7.
//!
//! ɢ governs the composition logic of messages.
//! ɢ_seq (2): sequential, ordered packet delivery.
//! ɢ_broad (3): multicast/broadcast support.

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
            Self::Sequential => "\u{0262}_seq",
            Self::Broadcast => "\u{0262}_broad",
        }
    }
}
