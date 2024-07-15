use regex_automata::{Match, PatternID};

use crate::DfaData;

use super::{Dfa, FindMatches};

/// A regular expression.
#[derive(Debug)]
pub struct Regex {
    /// The DFAs that are used to search for matches.
    pub dfas: Vec<Dfa>,
}

impl Regex {
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
        for dfa in self.dfas.iter_mut() {
            dfa.reset();
        }

        for (i, c) in char_indices {
            for dfa in self.dfas.iter_mut() {
                dfa.advance(i, c, matches_char_class);
            }

            if !self.dfas.iter().any(|dfa| dfa.search_on()) {
                // No DFA is still searching, so we can stop the search.
                break;
            }
        }

        self.find_first_longest_match()
    }

    /// We evaluate the matches of the DFAs in ascending order to prioritize the matches with the
    /// lowest pattern id.
    /// We find the pattern with the lowest start position and the longest length.
    fn find_first_longest_match(&mut self) -> Option<Match> {
        let mut current_match: Option<Match> = None;
        for (pattern, dfa) in self.dfas.iter().enumerate() {
            if let Some(span) = dfa.current_match() {
                if current_match.is_none()
                    || span.start < current_match.unwrap().start()
                    || span.start == current_match.unwrap().start()
                        && span.len() > current_match.unwrap().span().len()
                {
                    // We have a match and we continue the look for a longer match.
                    current_match = Some(Match::new(PatternID::new_unchecked(pattern), span));
                }
            }
        }
        current_match
    }
}

impl From<&[DfaData]> for Regex {
    fn from(dfas: &[DfaData]) -> Self {
        Regex {
            dfas: dfas.iter().map(Dfa::from).collect(),
        }
    }
}
