use crate::{character_class::CharacterClass, dfa::Dfa};

/// The `MultiPatternDfa` struct represents a multi-pattern DFA.
/// The `MultiPatternDfa` struct can be used to match multiple patterns in parallel.
pub struct MultiPatternDfa {
    /// The DFAs that are used to match the patterns. Each DFA is used to match a single pattern.
    dfas: Vec<Dfa>,
    /// The character classes that are used to match the patterns. These character classes are shared
    /// between the DFAs.
    char_classes: Vec<CharacterClass>,
}
