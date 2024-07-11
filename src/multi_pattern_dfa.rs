use crate::{character_class::CharacterClass, SinglePatternDfa};

/// The `MultiPatternDfa` struct represents a multi-pattern DFA.
/// The `MultiPatternDfa` struct can be used to match multiple patterns in parallel.
pub struct MultiPatternDfa {
    /// The DFAs that are used to match the patterns. Each DFA is used to match a single pattern.
    dfas: Vec<SinglePatternDfa>,
    /// The character classes that are used to match the patterns. These character classes are shared
    /// between the DFAs.
    char_classes: Vec<CharacterClass>,
}

impl MultiPatternDfa {
    /// Constructs a new `MultiPatternDfa` with the given DFAs and character classes.
    pub fn new(dfas: Vec<SinglePatternDfa>, char_classes: Vec<CharacterClass>) -> Self {
        Self { dfas, char_classes }
    }

    /// Returns the slice of SinglePatternDfa objects that are used to match the patterns.
    pub fn dfas(&self) -> &[SinglePatternDfa] {
        &self.dfas
    }

    /// Returns the slice of CharacterClass objects that are used to match the patterns.
    pub fn char_classes(&self) -> &[CharacterClass] {
        &self.char_classes
    }

    /// Returns the number of patterns that are matched by the `MultiPatternDfa`.
    pub fn num_patterns(&self) -> usize {
        self.dfas.len()
    }

    /// Returns the pattern that is matched by the `MultiPatternDfa` at the given index.
    pub fn pattern(&self, index: usize) -> &str {
        self.dfas[index].pattern()
    }
}
