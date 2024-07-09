#![allow(dead_code)]
use log::trace;
use regex_automata::{util::primitives::StateID, Match, Span};

use crate::{character_class::CharacterClass, dfa::Dfa, match_function::MatchFunction, Result};

/// A compiled DFA that can be used to match a string.
///
/// The DFA is compiled from a DFA by creating match functions for all character classes.
/// The match functions are used to decide if a character is in a character class.
///
/// MatchFunctions are not Clone nor Copy, so we aggregate them into a new struct CompiledDfa
/// which is Clone and Copy neither.
pub(crate) struct CompiledDfa {
    // The base DFA
    dfa: Dfa,
    // The match functions for the DFA
    match_functions: Vec<MatchFunction>,
    // The current state of the DFA during matching
    current_state: StateID,
    // The current position in the input string
    current_position: usize,
    // The start position of the current match
    start_position: Option<usize>,
    // The end position of the current match
    end_position: Option<usize>,
    // The last accepting state of the current match
    last_accepting_state: Option<StateID>,
    // The position of the last accepting state
    last_accepting_position: Option<usize>,
}

impl CompiledDfa {
    pub(crate) fn new(dfa: Dfa) -> Self {
        CompiledDfa {
            dfa,
            match_functions: Vec::new(),
            current_state: StateID::new_unchecked(0),
            current_position: 0,
            start_position: None,
            end_position: None,
            last_accepting_state: None,
            last_accepting_position: None,
        }
    }

    pub(crate) fn dfa(&self) -> &Dfa {
        &self.dfa
    }

    pub(crate) fn match_functions(&self) -> &[MatchFunction] {
        &self.match_functions
    }

    pub(crate) fn compile(&mut self) -> Result<()> {
        // Create the match functions for all character classes
        self.dfa
            .char_classes()
            .iter()
            .try_for_each(|char_class| -> Result<()> {
                let match_function = char_class.ast.0.clone().try_into()?;
                self.match_functions.push(match_function);
                Ok(())
            })?;
        Ok(())
    }

    pub(crate) fn reset(&mut self) {
        self.current_state = StateID::new_unchecked(0);
        self.current_position = 0;
        self.start_position = None;
        self.end_position = None;
        self.last_accepting_state = None;
        self.last_accepting_position = None;
    }

    pub(crate) fn reset_match(&mut self) {
        self.start_position = None;
        self.end_position = None;
    }

    pub(crate) fn current_state(&self) -> StateID {
        self.current_state
    }

    pub(crate) fn current_position(&self) -> usize {
        self.current_position
    }

    pub(crate) fn start_position(&self) -> Option<usize> {
        self.start_position
    }

    pub(crate) fn end_position(&self) -> Option<usize> {
        self.end_position
    }

    pub(crate) fn last_accepting_state(&self) -> Option<StateID> {
        self.last_accepting_state
    }

    pub(crate) fn last_accepting_position(&self) -> Option<usize> {
        self.last_accepting_position
    }

    pub(crate) fn set_current_state(&mut self, state: StateID) {
        self.current_state = state;
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
    pub(crate) fn find(&mut self, input: &str) -> Option<Match> {
        self.reset();
        let chars = input.char_indices();
        let mut current_match: Option<Match> = None;
        for (i, c) in chars {
            // Get the transitions for the current state
            if let Some(transitions) = self.dfa.transitions().get(&self.current_state) {
                if self.start_position.is_none() {
                    // Start of a new match
                    self.start_position = Some(i);
                }
                if let Some(next_state) =
                    Self::find_transition(transitions, &self.match_functions, c)
                {
                    self.current_state = next_state;
                    self.current_position = i + c.len_utf8();
                    if self.dfa.pattern_id(self.current_state).is_some() {
                        self.last_accepting_state = Some(self.current_state);
                        self.last_accepting_position = Some(self.current_position);
                        self.end_position = Some(i + c.len_utf8());
                        // Continue search on the next character for a longer match
                    }
                } else if self.last_accepting_state.is_some() {
                    // We had a match, return it
                    break;
                } else {
                    // We didn't have a match, continue search
                    self.reset_match();
                }
            } else {
                // Start search on the next character
                continue;
            }
        }
        if let Some(state_id) = self.last_accepting_state {
            current_match = Some(Match::new(
                self.dfa.pattern_id(state_id).unwrap(),
                Span {
                    start: self.start_position.unwrap(),
                    end: self.end_position.unwrap(),
                },
            ));
        }
        current_match
    }

    #[inline]
    fn find_transition(
        transitions: &std::collections::BTreeMap<CharacterClass, StateID>,
        match_functions: &[MatchFunction],
        c: char,
    ) -> Option<StateID> {
        for (char_class, target_state) in transitions {
            if match_functions[char_class.id()].call(c) {
                trace!(
                    "Transition: {} {} {:?} -> {:?}",
                    c,
                    char_class.id,
                    char_class.ast.0.to_string(),
                    target_state.as_usize()
                );
                return Some(*target_state);
            }
        }
        None
    }
}

#[cfg(test)]

mod tests {
    use regex_automata::PatternID;

    use crate::{dfa_render_to, MultiPatternNfa};

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
        // TestData {
        //     name: "parol_with_input_percent_sc",
        //     patterns: PATTERNS,
        //     input: "%sc %scanner ",
        //     match_result: Some((PatternID::new_unchecked(35), Span { start: 2, end: 5 })),
        // },
    ];

    // Initialize the logger for the tests
    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_find() {
        init();

        for data in TEST_DATA {
            let mut multi_pattern_nfa = MultiPatternNfa::new();
            multi_pattern_nfa.add_patterns(data.patterns).unwrap();
            let dfa = Dfa::try_from(multi_pattern_nfa).unwrap();
            let minimized_dfa = dfa.minimize().unwrap();
            dfa_render_to!(&minimized_dfa, &format!("{}_min_dfa", data.name));
            let mut compiled_dfa = CompiledDfa::new(minimized_dfa);
            compiled_dfa.compile().unwrap();
            let match_result = compiled_dfa
                .find(data.input)
                .map(|ma| (ma.pattern(), ma.span()));
            assert_eq!(match_result, data.match_result, "{}", data.name);
        }
    }
}
