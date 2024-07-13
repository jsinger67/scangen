use regex_automata::{Match, PatternID};

use crate::{
    compiled_dfa::CompiledDfa, dfa::Dfa, multi_pattern_nfa::MultiPatternNfa, unsupported, Result,
    ScanGenError, ScanGenErrorKind,
};

/// The `MultiPatternDfa` struct represents a multi-pattern DFA.
/// The `MultiPatternDfa` struct can be used to match multiple patterns in parallel.
#[derive(Default)]
pub struct MultiPatternDfa {
    /// The DFAs that are used to match the patterns. Each DFA is used to match a single pattern.
    dfas: Vec<CompiledDfa>,
}

impl MultiPatternDfa {
    /// Creates a new `MultiPatternDfa` object.
    pub fn new() -> Self {
        MultiPatternDfa::default()
    }
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
        if self.dfas.iter().any(|d| d.pattern() == pattern.as_ref()) {
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
        let mut compiled_dfa = CompiledDfa::new();
        compiled_dfa.compile(&minimzed_dfa)?;

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

    /// Executes a leftmost search and returns the first match that is found, if one exists.
    /// It starts the search at the position of the given CharIndices iterator.
    /// During the search, all DFAs are advanced in parallel by one character at a time.
    pub fn find_from(&mut self, char_indices: std::str::CharIndices) -> Option<Match> {
        for dfa in self.dfas.iter_mut() {
            dfa.reset();
        }

        for (i, c) in char_indices {
            for dfa in self.dfas.iter_mut() {
                dfa.advance(i, c);
            }

            if !self.dfas.iter().any(|dfa| dfa.search_on()) {
                // No DFA is still searching, so we can stop the search.
                break;
            }
        }

        self.find_first_longest_match()
    }

    /// Executes a leftmost search and returns the first match that is found, if one exists.
    /// During the search, all DFAs are advanced in parallel by one character at a time.
    pub fn find(&mut self, input: &str) -> Option<Match> {
        for dfa in self.dfas.iter_mut() {
            dfa.reset();
        }

        let chars = input.char_indices();
        for (i, c) in chars {
            for dfa in self.dfas.iter_mut() {
                dfa.advance(i, c);
            }

            if !self.dfas.iter().any(|dfa| dfa.search_on()) {
                // No DFA is still searching, so we can stop the search.
                break;
            }
        }

        self.find_first_longest_match()
    }

    /// Returns an iterator over all non-overlapping matches.
    /// The iterator yields a [`Match`] value until no more matches could be found.
    pub fn find_iter<'r, 'h>(&'r mut self, input: &'h str) -> FindMatches<'r, 'h> {
        FindMatches::new(self, input)
    }
}

impl std::fmt::Debug for MultiPatternDfa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MultiPatternDfa {{ dfas: {:?} }}", self.dfas)
    }
}

/// An iterator over all non-overlapping matches.
///
/// The iterator yields a [`Match`] value until no more matches could be found.
///
/// The lifetime parameters are as follows:
///
/// * `'r` represents the lifetime of the `Regex` that produced this iterator.
/// * `'h` represents the lifetime of the haystack being searched.
///
/// This iterator can be created with the [`MultiPatternDfa::find_iter`] method.
#[derive(Debug)]
pub struct FindMatches<'r, 'h> {
    multi_pattern_dfa: &'r mut MultiPatternDfa,
    char_indices: std::str::CharIndices<'h>,
}

impl<'r, 'h> FindMatches<'r, 'h> {
    /// Creates a new `FindMatches` iterator.
    pub fn new(multi_pattern_dfa: &'r mut MultiPatternDfa, input: &'h str) -> Self {
        FindMatches {
            multi_pattern_dfa,
            char_indices: input.char_indices(),
        }
    }

    /// Returns the next match in the haystack.
    ///
    /// If no match is found, `None` is returned.
    pub fn next(&mut self) -> Option<Match> {
        let result = self.multi_pattern_dfa.find_from(self.char_indices.clone());
        if let Some(matched) = result {
            // Advance the char_indices iterator to the end of the match.
            let end = matched.span().end - 1;
            let mut peekable = self.char_indices.by_ref().peekable();
            while peekable.next_if(|(i, _)| *i < end).is_some() {}
        }
        result
    }
}

impl Iterator for FindMatches<'_, '_> {
    type Item = Match;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

#[cfg(test)]

mod tests {
    use regex_automata::{PatternID, Span};

    use super::*;

    // Pattern taken from parol
    const PATTERNS: &[&str] = &[
        /* 0 */ "\\r\\n|\\r|\\n",
        /* 1 */ "[\\s--\\r\\n]+",
        /* 2 */ "(//.*(\\r\\n|\\r|\\n))",
        /* 3 */ "(/\\*.*?\\*/)",
        /* 4 */ "%start",
        /* 5 */ "%title",
        /* 6 */ "%comment",
        /* 7 */ "%user_type",
        /* 8 */ "=",
        /* 9 */ "%grammar_type",
        /* 10 */ "%line_comment",
        /* 11 */ "%block_comment",
        /* 12 */ "%auto_newline_off",
        /* 13 */ "%auto_ws_off",
        /* 14 */ "%on",
        /* 15 */ "%enter",
        /* 16 */ "%%",
        /* 17 */ "::",
        /* 18 */ ":",
        /* 19 */ ";",
        /* 20 */ "\\|",
        /* 21 */ "<",
        /* 22 */ ">",
        /* 23 */ "\"(\\\\.|[^\\\\])*?\"",
        /* 24 */ "'(\\\\'|[^'])*?'",
        /* 25 */ "\\u{2F}(\\\\.|[^\\\\])*?\\u{2F}",
        /* 26 */ "\\(",
        /* 27 */ "\\)",
        /* 28 */ "\\[",
        /* 29 */ "\\]",
        /* 30 */ "\\{",
        /* 31 */ "\\}",
        /* 32 */ "[a-zA-Z_][a-zA-Z0-9_]*",
        /* 33 */ "%scanner",
        /* 34 */ ",",
        /* 35 */ "%sc",
        /* 36 */ "%push",
        /* 37 */ "%pop",
        /* 38 */ "\\^",
        /* 39 */ ".",
    ];

    // A data type that provides test data for string search tests.
    struct TestDataFind {
        name: &'static str,
        patterns: &'static [&'static str],
        input: &'static str,
        match_result: Option<(PatternID, Span)>,
    }

    // Test data for string search tests.
    const TEST_DATA_FIND: &[TestDataFind] = &[
        TestDataFind {
            name: "in_int_with_input_int",
            patterns: &["in", "int"],
            input: "int",
            match_result: Some((PatternID::new_unchecked(1), Span { start: 0, end: 3 })),
        },
        TestDataFind {
            name: "in_int_with_input_in",
            patterns: &["in", "int"],
            input: "in",
            match_result: Some((PatternID::new_unchecked(0), Span { start: 0, end: 2 })),
        },
        TestDataFind {
            name: "in_int_with_input_in_padded_with_whitespace",
            patterns: &["in", "int"],
            input: "  in  ",
            match_result: Some((PatternID::new_unchecked(0), Span { start: 2, end: 4 })),
        },
        TestDataFind {
            name: "in_int_with_input_int_padded_with_whitespace",
            patterns: &["in", "int"],
            input: "  int  ",
            match_result: Some((PatternID::new_unchecked(1), Span { start: 2, end: 5 })),
        },
        TestDataFind {
            name: "in_int_with_input_int_padded_with_whitespace_and_newline",
            patterns: &["in", "int"],
            input: "  int  \n",
            match_result: Some((PatternID::new_unchecked(1), Span { start: 2, end: 5 })),
        },
        TestDataFind {
            name: "in_int_with_input_int_int",
            patterns: &["in", "int"],
            input: "  int  int ",
            match_result: Some((PatternID::new_unchecked(1), Span { start: 2, end: 5 })),
        },
        TestDataFind {
            name: "parol_with_input_space_percent_sc",
            patterns: PATTERNS,
            input: " %sc %scanner ",
            match_result: Some((PatternID::new_unchecked(1), Span { start: 0, end: 1 })),
        },
        TestDataFind {
            name: "parol_with_input_percent_sc",
            patterns: PATTERNS,
            input: "%sc %scanner ",
            match_result: Some((PatternID::new_unchecked(35), Span { start: 0, end: 3 })),
        },
        TestDataFind {
            name: "parol_with_input_percent_scan",
            patterns: PATTERNS,
            input: "%scan",
            // The pattern %sc is matched first, so the match is %sc.
            match_result: Some((PatternID::new_unchecked(35), Span { start: 0, end: 3 })),
        },
        TestDataFind {
            name: "parol_with_input_percent_scanner",
            patterns: PATTERNS,
            input: "%scanner ",
            match_result: Some((PatternID::new_unchecked(33), Span { start: 0, end: 8 })),
        },
    ];

    // A data type that provides test data for search iterator tests.
    struct TestDataFindIter {
        name: &'static str,
        patterns: &'static [&'static str],
        input: &'static str,
        match_result: &'static [(PatternID, Span)],
    }

    // Test data for search iterator tests.
    const TEST_DATA_FIND_ITER: &[TestDataFindIter] = &[
        TestDataFindIter {
            name: "in_int_with_input_int",
            patterns: &["in", "int"],
            input: "int",
            match_result: &[(PatternID::new_unchecked(1), Span { start: 0, end: 3 })],
        },
        TestDataFindIter {
            name: "in_int_with_input_in",
            patterns: &["in", "int"],
            input: "in",
            match_result: &[(PatternID::new_unchecked(0), Span { start: 0, end: 2 })],
        },
        TestDataFindIter {
            name: "in_int_with_input_in_padded_with_whitespace",
            patterns: &["in", "int"],
            input: "  in  ",
            match_result: &[(PatternID::new_unchecked(0), Span { start: 2, end: 4 })],
        },
        TestDataFindIter {
            name: "in_int_with_input_int_padded_with_whitespace",
            patterns: &["in", "int"],
            input: "  int  ",
            match_result: &[(PatternID::new_unchecked(1), Span { start: 2, end: 5 })],
        },
        TestDataFindIter {
            name: "in_int_with_input_int_padded_with_whitespace_and_newline",
            patterns: &["in", "int"],
            input: "  int  \n",
            match_result: &[(PatternID::new_unchecked(1), Span { start: 2, end: 5 })],
        },
        TestDataFindIter {
            name: "in_int_with_input_int_int",
            patterns: &["in", "int"],
            input: "  int  int ",
            match_result: &[
                (PatternID::new_unchecked(1), Span { start: 2, end: 5 }),
                (PatternID::new_unchecked(1), Span { start: 7, end: 10 }),
            ],
        },
        TestDataFindIter {
            name: "parol_with_input_space_percent_sc",
            patterns: PATTERNS,
            input: " %sc %scanner ",
            match_result: &[
                (PatternID::new_unchecked(1), Span { start: 0, end: 1 }), // whitespace
                (PatternID::new_unchecked(35), Span { start: 1, end: 4 }), // %sc
                (PatternID::new_unchecked(1), Span { start: 4, end: 5 }), // whitespace
                (PatternID::new_unchecked(33), Span { start: 5, end: 13 }), // %scanner
                (PatternID::new_unchecked(1), Span { start: 13, end: 14 }), // whitespace
            ],
        },
        TestDataFindIter {
            name: "parol_with_input_percent_sc_space_percent_scanner",
            patterns: PATTERNS,
            input: "%sc %scanner ",
            match_result: &[
                (PatternID::new_unchecked(35), Span { start: 0, end: 3 }), // %sc
                (PatternID::new_unchecked(1), Span { start: 3, end: 4 }),  // whitespace
                (PatternID::new_unchecked(33), Span { start: 4, end: 12 }), // %scanner
                (PatternID::new_unchecked(1), Span { start: 12, end: 13 }), // whitespace
            ],
        },
        TestDataFindIter {
            name: "parol_with_input_percent_scan",
            patterns: PATTERNS,
            input: "%scan",
            // The pattern %sc is matched first, so the match is %sc.
            // The remaining input is "an", which matches the identifier pattern (32).
            match_result: &[
                (PatternID::new_unchecked(35), Span { start: 0, end: 3 }),
                (PatternID::new_unchecked(32), Span { start: 3, end: 5 }),
            ],
        },
        TestDataFindIter {
            name: "parol_with_input_percent_scanner",
            patterns: PATTERNS,
            input: "%scanner ",
            match_result: &[
                (PatternID::new_unchecked(33), Span { start: 0, end: 8 }),
                (PatternID::new_unchecked(1), Span { start: 8, end: 9 }),
            ],
        },
    ];

    // Initialize the logger for the tests
    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_multi_dfa_find() {
        init();

        for data in TEST_DATA_FIND {
            let mut multi_pattern_dfa = MultiPatternDfa::new();
            multi_pattern_dfa.add_patterns(data.patterns).unwrap();
            let match_result = multi_pattern_dfa
                .find(data.input)
                .map(|ma| (ma.pattern(), ma.span()));
            assert_eq!(match_result, data.match_result, "{}", data.name);
        }
    }

    #[test]
    fn test_multi_dfa_find_iter() {
        init();

        for data in TEST_DATA_FIND_ITER {
            let mut multi_pattern_dfa = MultiPatternDfa::new();
            multi_pattern_dfa.add_patterns(data.patterns).unwrap();
            let find_iter = multi_pattern_dfa.find_iter(data.input);
            let match_result: Vec<(PatternID, Span)> =
                find_iter.map(|ma| (ma.pattern(), ma.span())).collect();
            assert_eq!(match_result.as_slice(), data.match_result, "{}", data.name);
        }
    }
}
