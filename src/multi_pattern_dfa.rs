use regex_automata::{Match, PatternID};

use crate::{
    compiled_dfa::CompiledDfa, dfa::Dfa, multi_pattern_nfa::MultiPatternNfa, unsupported, Result,
    ScanGenError, ScanGenErrorKind,
};

/// The `MultiPatternDfa` struct represents a multi-pattern DFA.
/// The `MultiPatternDfa` struct can be used to match multiple patterns in parallel.
pub struct MultiPatternDfa {
    /// The DFAs that are used to match the patterns. Each DFA is used to match a single pattern.
    dfas: Vec<CompiledDfa>,
}

impl MultiPatternDfa {
    /// Returns the slice of Dfa objects that are used to match the patterns.
    pub fn dfas(&self) -> &[CompiledDfa] {
        &self.dfas
    }

    /// Returns the number of patterns that are matched by the `MultiPatternDfa`.
    pub fn num_patterns(&self) -> usize {
        self.dfas.len()
    }

    /// Add a pattern to the multi-pattern DFA.
    pub fn add_pattern<S>(&mut self, pattern: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        if self
            .dfas
            .iter()
            .any(|d| d.dfa().patterns()[0] == pattern.as_ref())
        {
            // If the pattern already exists, do nothing.
            // Not sure if this should rather be an error.
            return Ok(());
        }

        let mut multi_pattern_nfa = MultiPatternNfa::new();
        multi_pattern_nfa.add_pattern(pattern.as_ref())?;

        // Convert the multi-pattern NFA to a DFA and minimize it.
        let dfa: Dfa = multi_pattern_nfa.try_into()?;
        let minimzed_dfa = dfa.minimize()?;

        // Compile the minimized DFA.
        let mut compiled_dfa = CompiledDfa::new(minimzed_dfa);
        compiled_dfa.compile()?;

        // Add the compiled DFA to the list of DFAs.
        self.dfas.push(compiled_dfa);

        Ok(())
    }

    /// Add multiple patterns to the multi-pattern DFA.
    pub fn add_patterns<I, S>(&mut self, patterns: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for (index, pattern) in patterns.into_iter().enumerate() {
            let result = self.add_pattern(pattern.as_ref()).map(|_| ());
            if let Err(ScanGenError { source }) = &result {
                match &**source {
                    ScanGenErrorKind::RegexSyntaxError(_) => result?,
                    ScanGenErrorKind::UnsupportedFeature(s) => Err(unsupported!(format!(
                        "Error in pattern #{} '{}': {}",
                        index,
                        pattern.as_ref(),
                        s
                    )))?,
                    _ => result?,
                }
            } else {
                result?;
            }
        }
        Ok(())
    }

    /// Executes a leftmost search and returns the first match that is found, if one exists.
    /// During the search, the current state and position are updated.
    /// If a match is found, the start and end positions of the match are stored.
    /// During search we can have several conditions:
    /// 1. We have a match and we continue the search for a longer match.
    /// 2. We have a start of a match but we can't match the next character, so we re-start the
    /// search on the next character. Therefore we need to reset the start_position and end_position
    /// to None
    /// 3. We don't have a match and we continue the search. If we reach the end of the input string
    /// we return None.
    pub fn find(&mut self, input: &str) -> Option<Match> {
        for dfa in self.dfas.iter_mut() {
            dfa.reset();
        }

        let chars = input.char_indices();
        let mut current_match: Option<Match> = None;
        for (i, c) in chars {
            for dfa in self.dfas.iter_mut() {
                dfa.advance(i, c);
            }

            // We evaluate the matches of the DFAs in reverse order to prioritize the matches with the
            // highest pattern id.
            // for (pattern, dfa) in self.dfas.iter().enumerate().rev() {
            //     if let Some(match_) = dfa.current_match() {
            //         current_match =
            //             Some(Match::new(PatternID::new_unchecked(pattern), match_.span()));
            //         break;
            //     }
            // }
        }

        // We evaluate the matches of the DFAs in reverse order to prioritize the matches with the
        // highest pattern id.
        for (pattern, dfa) in self.dfas.iter().enumerate().rev() {
            if let Some(match_) = dfa.current_match() {
                current_match = Some(Match::new(PatternID::new_unchecked(pattern), match_));
                break;
            }
        }
        current_match
    }
}
