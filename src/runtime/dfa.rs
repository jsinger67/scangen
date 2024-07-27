use crate::{
    common::{MatchingState, Span},
    DfaData, Match,
};

/// Runtime version of a DFA.
#[derive(Debug, Clone)]
pub struct Dfa {
    /// The pattern that this DFA recognizes.
    pub pattern: String,
    /// The states that are accepting states.
    pub accepting_states: Vec<usize>,
    /// The ranges of transitions for each state.
    pub state_ranges: Vec<(usize, usize)>,
    /// The transitions for each state.
    pub transitions: Vec<(usize, (usize, usize))>,
    /// The current matching state of the DFA.
    pub(crate) matching_state: MatchingState<usize>,
}

impl Dfa {
    /// Advances the DFA by one character.
    pub fn advance(&mut self, c_pos: usize, c: char, matches_char_class: fn(char, usize) -> bool) {
        // If we already have the longest match, we can stop
        if self.matching_state.is_longest_match() {
            return;
        }
        // Get the transitions for the current state
        if let Some(next_state) = self.find_transition(c, matches_char_class) {
            if self.accepting_states.contains(&next_state) {
                self.matching_state.transition_to_accepting(c_pos, c);
            } else {
                self.matching_state.transition_to_non_accepting(c_pos);
            }
            self.matching_state.set_current_state(next_state);
        } else {
            self.matching_state.no_transition();
        }
    }

    /// Finds the next state of the DFA.
    fn find_transition(
        &self,
        c: char,
        matches_char_class: fn(char, usize) -> bool,
    ) -> Option<usize> {
        let current_state = self.matching_state.current_state();
        let (start, end) = self.state_ranges[current_state];
        let transitions = &self.transitions[start..end];
        transitions
            .iter()
            .find_map(|(state, (char_class, target_state))| {
                debug_assert_eq!(state, &current_state);
                if (matches_char_class)(c, *char_class) {
                    Some(*target_state)
                } else {
                    None
                }
            })
    }

    #[inline]
    pub(crate) fn reset(&mut self) {
        self.matching_state = MatchingState::new();
    }

    /// Returns true if the search should continue on the next character if the automaton has ever
    /// been in the matching state Start.
    #[inline]
    pub(crate) fn search_for_longer_match(&self) -> bool {
        !self.matching_state.is_longest_match() && !self.matching_state.is_no_match()
    }

    /// Returns the current match.
    #[inline]
    pub(crate) fn current_match(&self) -> Option<Span> {
        self.matching_state.last_match()
    }
}

impl From<&DfaData> for Dfa {
    fn from(data: &DfaData) -> Self {
        Dfa {
            pattern: data.0.to_owned(),
            accepting_states: data.1.to_vec(),
            state_ranges: data.2.to_vec(),
            transitions: data.3.to_vec(),
            matching_state: MatchingState::new(),
        }
    }
}

/// A DFA bundled with its associated token type number.
/// This struct is used to allow different token type number for the same pattern, i.e. Dfas, in
/// different scanner modes.
///
/// You could imagine to have differnt patterns for, e.g. a Comment in different scanner modes, but
/// you want to have the same token type number for all of them.
#[derive(Debug)]
pub(crate) struct DfaWithTokenType {
    dfa: Dfa,
    token_type: usize,
}

impl DfaWithTokenType {
    /// Creates a new DFA with its associated token type number.
    pub(crate) fn new(dfa: Dfa, token_type: usize) -> Self {
        Self { dfa, token_type }
    }

    /// Returns the current match.
    #[inline]
    pub(crate) fn current_match(&self) -> Option<Match> {
        self.dfa
            .current_match()
            .map(|span| Match::new(self.token_type, span))
    }

    /// Resets the DFA.
    #[inline]
    pub(crate) fn reset(&mut self) {
        self.dfa.reset();
    }

    /// Advances the DFA by one character.
    #[inline]
    pub(crate) fn advance(
        &mut self,
        c_pos: usize,
        c: char,
        matches_char_class: fn(char, usize) -> bool,
    ) {
        self.dfa.advance(c_pos, c, matches_char_class);
    }

    /// Returns the matching state of the DFA.
    #[inline]
    pub(crate) fn matching_state(&self) -> &MatchingState<usize> {
        &self.dfa.matching_state
    }

    /// Returns true if the search should continue on the next character if the automaton has ever
    /// been in the matching state Start.
    /// This is used to determine if the search should continue after the automaton has found a
    /// match.
    #[inline]
    pub(crate) fn search_for_longer_match(&self) -> bool {
        self.dfa.search_for_longer_match()
    }
}
