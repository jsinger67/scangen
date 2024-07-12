use log::trace;
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
    /// During the search, all DFAs are advanced in parallel by one character at a time.
    pub fn find(&mut self, input: &str) -> Option<Match> {
        for dfa in self.dfas.iter_mut() {
            dfa.reset();
        }

        let chars = input.char_indices();
        for (i, c) in chars {
            for (pattern_index, dfa) in self.dfas.iter_mut().enumerate() {
                dfa.advance(i, c);
                trace!(
                    "DFA for pattern #{} is in {:?}",
                    pattern_index,
                    dfa.matching_state().inner_state()
                );
            }

            if !self.dfas.iter().any(|dfa| dfa.search_on()) {
                // No DFA is still searching, so we can stop the search.
                break;
            }
        }

        // We evaluate the matches of the DFAs in ascending order to prioritize the matches with the
        // lowest pattern id.
        // We find the pattern with the lowest start position and the longest length.
        let mut current_match: Option<Match> = None;
        for (pattern, dfa) in self.dfas.iter().enumerate() {
            if let Some(span) = dfa.current_match() {
                if current_match.is_none()
                    || span.start < current_match.unwrap().start()
                    || span.start == current_match.unwrap().start()
                        && span.len() > current_match.unwrap().span().len()
                {
                    // We have a match and we continue the look for a longer match.
                    trace!("Matched pattern #{}: {:?}", pattern, span);
                    current_match = Some(Match::new(PatternID::new_unchecked(pattern), span));
                }
            }
        }
        current_match
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
    struct TestData {
        name: &'static str,
        patterns: &'static [&'static str],
        input: &'static str,
        match_result: Option<(PatternID, Span)>,
    }

    // Test data for string search tests.
    const TEST_DATA: &[TestData] = &[
        TestData {
            name: "in_int_with_input_int",
            patterns: &["in", "int"],
            input: "int",
            match_result: Some((PatternID::new_unchecked(1), Span { start: 0, end: 3 })),
        },
        TestData {
            name: "in_int_with_input_in",
            patterns: &["in", "int"],
            input: "in",
            match_result: Some((PatternID::new_unchecked(0), Span { start: 0, end: 2 })),
        },
        TestData {
            name: "in_int_with_input_in_padded_with_whitespace",
            patterns: &["in", "int"],
            input: "  in  ",
            match_result: Some((PatternID::new_unchecked(0), Span { start: 2, end: 4 })),
        },
        TestData {
            name: "in_int_with_input_int_padded_with_whitespace",
            patterns: &["in", "int"],
            input: "  int  ",
            match_result: Some((PatternID::new_unchecked(1), Span { start: 2, end: 5 })),
        },
        TestData {
            name: "in_int_with_input_int_padded_with_whitespace_and_newline",
            patterns: &["in", "int"],
            input: "  int  \n",
            match_result: Some((PatternID::new_unchecked(1), Span { start: 2, end: 5 })),
        },
        TestData {
            name: "in_int_with_input_int_int",
            patterns: &["in", "int"],
            input: "  int  int ",
            match_result: Some((PatternID::new_unchecked(1), Span { start: 2, end: 5 })),
        },
        TestData {
            name: "parol_with_input_space_percent_sc",
            patterns: PATTERNS,
            input: " %sc %scanner ",
            match_result: Some((PatternID::new_unchecked(1), Span { start: 0, end: 1 })),
        },
        TestData {
            name: "parol_with_input_percent_sc",
            patterns: PATTERNS,
            input: "%sc %scanner ",
            match_result: Some((PatternID::new_unchecked(35), Span { start: 0, end: 3 })),
        },
        TestData {
            name: "parol_with_input_percent_scan",
            patterns: PATTERNS,
            input: "%scan",
            // The pattern %sc is matched first, so the match is %sc.
            match_result: Some((PatternID::new_unchecked(35), Span { start: 0, end: 3 })),
        },
        TestData {
            name: "parol_with_input_percent_scanner",
            patterns: PATTERNS,
            input: "%scanner ",
            match_result: Some((PatternID::new_unchecked(33), Span { start: 0, end: 8 })),
        },
    ];

    // Initialize the logger for the tests
    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_multi_dfa_find() {
        init();

        for data in TEST_DATA {
            let mut multi_pattern_nfa = MultiPatternDfa::new();
            multi_pattern_nfa.add_patterns(data.patterns).unwrap();
            let match_result = multi_pattern_nfa
                .find(data.input)
                .map(|ma| (ma.pattern(), ma.span()));
            assert_eq!(match_result, data.match_result, "{}", data.name);
        }
    }
}
