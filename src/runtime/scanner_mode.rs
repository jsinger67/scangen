use crate::ScannerModeData;

use super::{Dfa, DfaWithTokenType};

/// A ScannerMode is a set of active DFAs with their associated token type numbers.
///
/// The DFAs are clones from the Scanner's `dfas` field for the sake of performance.
/// The token type numbers are of type `usize` bundled with the DFAs.
#[derive(Debug, Clone)]
pub struct ScannerMode {
    /// The name of the mode.
    pub name: String,
    /// The DFAs and their associated token type numbers.
    pub(crate) dfas: Vec<DfaWithTokenType>,
    /// The transitions between the scanner modes triggered by a token type number.
    /// The entries are tuples of the token type numbers and the new scanner mode index and are
    /// sorted by token type number.
    pub(crate) transitions: Vec<(usize, usize)>,
}

impl ScannerMode {
    /// Creates a new scanner mode from the Scanner's DFAs and the ScannerModeData.
    pub fn new(dfas: &[Dfa], scanner_mode_data: &ScannerModeData) -> Self {
        let name = scanner_mode_data.0.to_string();
        let dfas = scanner_mode_data
            .1
            .iter()
            .map(|(dfa_index, token_type)| {
                DfaWithTokenType::new(dfas[*dfa_index].clone(), *token_type)
            })
            .collect();
        let mut transitions = scanner_mode_data.2.to_vec();
        transitions.sort_by_key(|(term, _)| *term);
        Self {
            name,
            dfas,
            transitions,
        }
    }

    /// Check if the scanner configuration has a transition on the given terminal index
    pub fn has_transition(&self, token_type: usize) -> Option<usize> {
        for (term, scanner) in &self.transitions {
            match token_type.cmp(term) {
                std::cmp::Ordering::Less => return None,
                std::cmp::Ordering::Equal => return Some(*scanner),
                std::cmp::Ordering::Greater => continue,
            }
        }
        None
    }

    /// Returns the name of the scanner mode.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]

mod tests {
    use crate::common::MatchingState;

    use super::*;

    const SCANNER_MODE: ScannerModeData = (
        "test",
        &[(0usize, 0usize)],
        &[(0usize, 0usize), (1usize, 1usize), (3usize, 2usize)],
    );

    #[test]
    fn test_scanner_mode() {
        let dfa = Dfa {
            pattern: "test".to_string(),
            accepting_states: vec![0],
            state_ranges: vec![(0, 0), (1, 1), (2, 2), (3, 3)],
            transitions: vec![],
            matching_state: MatchingState::default(),
        };
        let dfas = vec![dfa];
        let scanner_mode = ScannerMode::new(&dfas, &SCANNER_MODE);
        assert_eq!(scanner_mode.name, "test");
        assert_eq!(scanner_mode.dfas.len(), 1);
        assert_eq!(scanner_mode.transitions.len(), 3);
        assert_eq!(scanner_mode.has_transition(0), Some(0));
        assert_eq!(scanner_mode.has_transition(1), Some(1));
        assert_eq!(scanner_mode.has_transition(2), None);
        assert_eq!(scanner_mode.has_transition(3), Some(2));
        assert_eq!(scanner_mode.has_transition(8), None);
    }
}
