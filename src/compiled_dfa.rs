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
pub struct CompiledDfa {
    // The base DFA
    dfa: Dfa,
    // The match functions for the DFA
    match_functions: Vec<MatchFunction>,
    // The current state of the DFA during matching
    current_state: StateID,
    // The state of matching
    matching_state: MatchingState,
}

impl CompiledDfa {
    pub fn new(dfa: Dfa) -> Self {
        CompiledDfa {
            dfa,
            match_functions: Vec::new(),
            current_state: StateID::new_unchecked(0),
            matching_state: MatchingState::new(),
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

    pub(crate) fn matching_state(&self) -> &MatchingState {
        &self.matching_state
    }

    pub(crate) fn reset(&mut self) {
        self.current_state = StateID::new_unchecked(0);
        self.matching_state = MatchingState::new();
    }

    pub(crate) fn current_state(&self) -> StateID {
        self.current_state
    }

    pub(crate) fn current_match(&self) -> Option<Span> {
        self.matching_state.last_match()
    }

    pub(crate) fn advance(&mut self, c_pos: usize, c: char) {
        // Get the transitions for the current state
        if let Some(transitions) = self.dfa.transitions().get(&self.current_state) {
            if let Some(next_state) = Self::find_transition(transitions, &self.match_functions, c) {
                if self.dfa.accepting_states().contains_key(&next_state) {
                    self.matching_state.transition_to_accepting(c_pos, c);
                } else {
                    self.matching_state.transition_to_non_accepting(c_pos);
                }
                self.current_state = next_state;
            } else {
                self.matching_state.no_transition();
            }
        } else {
            // Start search on the next character
            self.matching_state.no_transition();
        }
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
        for (c_pos, c) in chars {
            // Get the transitions for the current state
            if let Some(transitions) = self.dfa.transitions().get(&self.current_state) {
                if let Some(next_state) =
                    Self::find_transition(transitions, &self.match_functions, c)
                {
                    if self.dfa.accepting_states().contains_key(&next_state) {
                        self.matching_state.transition_to_accepting(c_pos, c);
                    } else {
                        self.matching_state.transition_to_non_accepting(c_pos);
                    }
                    self.current_state = next_state;
                } else {
                    self.matching_state.no_transition();
                }
            } else {
                // Start search on the next character
                self.matching_state.no_transition();
                if self.matching_state.is_longest_match() {
                    break;
                }
                continue;
            }
        }
        if let Some(span) = self.matching_state.last_match() {
            let pattern_id = self.dfa.accepting_states()[&self.current_state];
            Some(Match::new(pattern_id, span))
        } else {
            None
        }
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
                    "Transition: '{}' Id{} {:?} -> {:?}",
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

    /// Returns true if the search should continue on the next character.
    pub(crate) fn search_on(&self) -> bool {
        !self.matching_state.is_longest_match()
    }
}

/// The state of the DFA during matching.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct MatchingState {
    // The current state of the DFA during matching
    state: InnerMatchingState,
    // The start position of the current match
    start_position: Option<usize>,
    // The end position of the current match
    end_position: Option<usize>,
}

impl MatchingState {
    pub(crate) fn new() -> Self {
        MatchingState::default()
    }

    // See matching_state.dot for the state diagram
    pub(crate) fn no_transition(&mut self) {
        match self.state {
            InnerMatchingState::None => {
                // We had no match, continue search
            }
            InnerMatchingState::Start => *self = MatchingState::default(),
            InnerMatchingState::Accepting => {
                // We had a recorded match, return to it
                *self = MatchingState {
                    state: InnerMatchingState::Longest,
                    ..self.clone()
                }
            }
            InnerMatchingState::Longest => {
                // We had the longest match, keep it
            }
        };
    }

    // See matching_state.dot for the state diagram
    pub(crate) fn transition_to_non_accepting(&mut self, i: usize) {
        match self.state {
            InnerMatchingState::None => {
                *self = MatchingState {
                    state: InnerMatchingState::Start,
                    start_position: Some(i),
                    ..self.clone()
                }
            }
            InnerMatchingState::Start => {
                // Continue search for an accepting state
            }
            InnerMatchingState::Accepting => {
                // We had a match, keep it and continue search for a longer match
            }
            InnerMatchingState::Longest => {
                // We had the longest match, keep it
            }
        }
    }

    // See matching_state.dot for the state diagram
    pub(crate) fn transition_to_accepting(&mut self, i: usize, c: char) {
        match self.state {
            InnerMatchingState::None => {
                *self = MatchingState {
                    state: InnerMatchingState::Accepting,
                    start_position: Some(i),
                    end_position: Some(i + c.len_utf8()),
                }
            }
            InnerMatchingState::Start => {
                *self = MatchingState {
                    state: InnerMatchingState::Accepting,
                    end_position: Some(i + c.len_utf8()),
                    ..self.clone()
                }
            }
            InnerMatchingState::Accepting => {
                *self = MatchingState {
                    end_position: Some(i + c.len_utf8()),
                    ..self.clone()
                }
            }
            InnerMatchingState::Longest => {
                // We had the longest match, keep it
            }
        }
    }

    pub(crate) fn is_no_match(&self) -> bool {
        matches!(self.state, InnerMatchingState::None)
    }

    pub(crate) fn is_start_match(&self) -> bool {
        matches!(self.state, InnerMatchingState::Start)
    }

    pub(crate) fn is_accepting_match(&self) -> bool {
        matches!(self.state, InnerMatchingState::Accepting)
    }

    pub(crate) fn is_longest_match(&self) -> bool {
        matches!(self.state, InnerMatchingState::Longest)
    }

    pub(crate) fn last_match(&self) -> Option<Span> {
        if let (Some(start), Some(end)) = (self.start_position, self.end_position) {
            Some(Span { start, end })
        } else {
            None
        }
    }

    pub(crate) fn inner_state(&self) -> InnerMatchingState {
        self.state
    }
}

/// The state enumeration of the DFA during matching.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InnerMatchingState {
    /// No match recorded so far.
    /// Continue search on the next character.
    ///
    /// Current state is not an accepting state.
    ///
    /// If a transition to a non-accepting state is found, record the start of the match and switch
    /// to StartMatch.
    /// If a transition to an accepting state is found, record the match and switch to AcceptingMatch.
    /// If no transition is found stay in NoMatch.
    #[default]
    None,

    /// Start of a match has been recorded.
    /// Continue search for an accepting state.
    ///
    /// Current state is not an accepting state.
    ///
    /// If a transition is found, record the match and switch to AcceptingMatch.
    /// If no transition is found, reset the match and switch to NoMatch.
    Start,

    /// Match has been recorded before, continue search for a longer match.
    ///
    /// State is an accepting state.
    ///
    /// If no transition is found, switch to LongestMatch.
    /// If a transition to a non-accepting state is found stay in AcceptingMatch.
    /// If a transition to an accepting state is found, record the match and stay in AcceptingMatch.
    Accepting,

    /// Match has been recorded before.
    /// The match is the longest match found, no longer match is possible.
    ///
    /// State is an accepting state.
    ///
    /// This state can't be left.
    Longest,
}

#[cfg(test)]

mod tests {
    use regex_automata::PatternID;

    use crate::{dfa_render_to, MultiPatternNfa};

    use super::*;

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
