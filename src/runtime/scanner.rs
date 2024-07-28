use crate::common::Match;

use crate::DfaData;

use super::{Dfa, FindMatches, ScannerMode};

/// A Scanner.
/// It consists of multiple DFAs that are used to search for matches.
///
/// Each DFA corresponds to a terminal symbol (token type) the lexer/scanner can recognize.
/// The DFAs are advanced in parallel to search for matches.
/// It further constists of at least one scanner mode. Scanners generated by `scangen` support
/// multiple scanner modes. This feature is known from Flex as *Start conditions* and provides more
/// flexibility by defining several scanners for several parts of your grammar.
/// See <https://www.cs.princeton.edu/~appel/modern/c/software/flex/flex.html#SEC11>
/// for more information.
///
/// To create a scanner, you can use the `ScannerBuilder` to add DFAs and scanner mode data.
/// If no scanner mode data is added, a default mode is created.
/// The default mode contains all DFAs and assigns incrementing token type numbers to them.
/// The default mode is named `INITIAL`.
///
#[derive(Debug)]
pub struct Scanner {
    /// The DFAs that are used to search for matches.
    pub(crate) dfas: Vec<Dfa>,
    /// The scanner modes that are used to search for matches.
    pub(crate) scanner_modes: Vec<ScannerMode>,
    /// The current scanner mode.
    pub(crate) current_mode: usize,
}

impl Scanner {
    /// Returns an iterator over all non-overlapping matches.
    /// The iterator yields a [`Match`] value until no more matches could be found.
    pub fn find_iter<'r, 'h>(
        &'r mut self,
        input: &'h str,
        matches_char_class: fn(char, usize) -> bool,
    ) -> FindMatches<'r, 'h> {
        FindMatches::new(self, input, matches_char_class)
    }

    /// Executes a leftmost search and returns the first match that is found, if one exists.
    /// It starts the search at the position of the given CharIndices iterator.
    /// During the search, all DFAs are advanced in parallel by one character at a time.
    pub fn find_from(
        &mut self,
        char_indices: std::str::CharIndices,
        matches_char_class: fn(char, usize) -> bool,
    ) -> Option<Match> {
        let current_mode = &mut self.scanner_modes[self.current_mode];
        for dfa in current_mode.dfas.iter_mut() {
            dfa.reset();
        }

        // All indices of the DFAs that are still active.
        let mut active_dfas = (0..current_mode.dfas.len()).collect::<Vec<_>>();

        for (i, c) in char_indices {
            for dfa_index in &active_dfas {
                current_mode.dfas[*dfa_index].advance(i, c, matches_char_class);
            }

            if i == 0 {
                // We remove all DFAs that did not find a match at the start position.
                for (index, dfa) in current_mode.dfas.iter().enumerate() {
                    if dfa.matching_state().is_no_match() {
                        active_dfas.retain(|&dfa_index| dfa_index != index);
                    }
                }
            }

            // We remove all DFAs from `active_dfas` that finished.
            active_dfas.retain(|&dfa_index| current_mode.dfas[dfa_index].search_for_longer_match());

            // If all DFAs have finished, we can stop the search.
            if active_dfas.is_empty() {
                break;
            }
        }

        let current_match = self.find_first_longest_match();
        self.execute_possible_mode_switch(current_match);
        current_match
    }

    /// This function is used by [super::find_matches::FindMatches::peek_n].
    ///
    /// Executes a leftmost search and returns the first match that is found, if one exists.
    /// It starts the search at the position of the given CharIndices iterator.
    /// In contrast to `find_from`, this method does not execute a mode switch if a transition is
    /// defined for the token type found.
    pub(crate) fn peek_from(
        &mut self,
        char_indices: &mut std::str::CharIndices,
        matches_char_class: fn(char, usize) -> bool,
    ) -> Option<Match> {
        let current_mode = &mut self.scanner_modes[self.current_mode];
        for dfa in current_mode.dfas.iter_mut() {
            dfa.reset();
        }

        // All indices of the DFAs that are still active.
        let mut active_dfas = (0..current_mode.dfas.len()).collect::<Vec<_>>();

        for (i, c) in char_indices {
            for dfa_index in &active_dfas {
                current_mode.dfas[*dfa_index].advance(i, c, matches_char_class);
            }

            if i == 0 {
                // We remove all DFAs that did not find a match at the start position.
                for (index, dfa) in current_mode.dfas.iter().enumerate() {
                    if dfa.matching_state().is_no_match() {
                        active_dfas.retain(|&dfa_index| dfa_index != index);
                    }
                }
            }

            // We remove all DFAs from `active_dfas` that finished.
            active_dfas.retain(|&dfa_index| current_mode.dfas[dfa_index].search_for_longer_match());

            // If all DFAs have finished, we can stop the search.
            if active_dfas.is_empty() {
                break;
            }
        }

        self.find_first_longest_match()
    }

    /// We evaluate the matches of the DFAs in ascending order to prioritize the matches with the
    /// lowest index.
    /// We find the pattern with the lowest start position and the longest length.
    fn find_first_longest_match(&mut self) -> Option<Match> {
        let mut current_match: Option<Match> = None;
        {
            let current_mode = &self.scanner_modes[self.current_mode];
            for dfa in current_mode.dfas.iter() {
                if let Some(dfa_match) = dfa.current_match() {
                    if current_match.is_none()
                        || dfa_match.start() < current_match.unwrap().start()
                        || dfa_match.start() == current_match.unwrap().start()
                            && dfa_match.len() > current_match.unwrap().span().len()
                    {
                        // We have a match and we continue the look for a longer match.
                        current_match = Some(dfa_match);
                    }
                }
            }
        }
        current_match
    }

    /// Executes a possible mode switch if a transition is defined for the token type found.
    #[inline]
    fn execute_possible_mode_switch(&mut self, current_match: Option<Match>) {
        let current_mode = &self.scanner_modes[self.current_mode];
        if let Some(current_match) = current_match.as_ref() {
            // We perform a scanner mode switch if a transition is defined for the token type found.
            if let Some(next_mode) = current_mode.has_transition(current_match.token_type()) {
                self.current_mode = next_mode;
            }
        }
    }

    /// Returns the number of the next scanner mode if a transition is defined for the token type.
    /// If no transition is defined, None returned.
    pub fn has_transition(&self, token_type: usize) -> Option<usize> {
        self.scanner_modes[self.current_mode].has_transition(token_type)
    }

    /// Sets the current scanner mode.
    ///
    /// A parser can explicitly set the scanner mode to switch to a different set of DFAs.
    /// Usually, the scanner mode is changed by the scanner itself based on the transitions defined
    /// in the scanner mode.
    pub fn set_mode(&mut self, mode: usize) {
        self.current_mode = mode;
    }

    /// Returns the current scanner mode.
    pub fn current_mode(&self) -> usize {
        self.current_mode
    }
}

impl From<&[DfaData]> for Scanner {
    fn from(dfas: &[DfaData]) -> Self {
        Scanner {
            dfas: dfas.iter().map(Dfa::from).collect(),
            scanner_modes: Vec::new(),
            current_mode: 0,
        }
    }
}
