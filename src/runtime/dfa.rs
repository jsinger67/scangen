use regex_automata::Span;

use crate::compiled_dfa::MatchingState;

/// The data of a DFA generated as Rust code.
pub type DfaData = (
    &'static str,
    &'static [usize],
    &'static [(usize, usize)],
    &'static [(usize, (usize, usize))],
);

/// Runtime version of a DFA.
#[derive(Debug, Clone)]
pub struct Dfa {
    /// The pattern that this DFA recognizes.
    pub pattern: &'static str,
    /// The states that are accepting states.
    pub accepting_states: &'static [usize],
    /// The ranges of transitions for each state.
    pub state_ranges: &'static [(usize, usize)],
    /// The transitions for each state.
    pub transitions: &'static [(usize, (usize, usize))],
    /// The current state of the DFA.
    pub current_state: usize,
    /// The current matching state of the DFA.
    pub matching_state: MatchingState,
}

impl Dfa {
    /// Advances the DFA by one character.
    #[inline]
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
            self.current_state = next_state;
        } else {
            self.matching_state.no_transition();
        }
    }

    /// Finds the next state of the DFA.
    #[inline]
    fn find_transition(
        &self,
        c: char,
        matches_char_class: fn(char, usize) -> bool,
    ) -> Option<usize> {
        let (start, end) = self.state_ranges[self.current_state];
        let transitions = &self.transitions[start..end];
        for (state, (char_class, target_state)) in transitions {
            debug_assert_eq!(state, &self.current_state);
            if (matches_char_class)(c, *char_class) {
                return Some(*target_state);
            }
        }
        None
    }

    #[inline]
    pub(crate) fn reset(&mut self) {
        self.current_state = 0;
        self.matching_state = MatchingState::new();
    }

    /// Returns true if the search should continue on the next character.
    #[inline]
    pub(crate) fn search_on(&self) -> bool {
        !self.matching_state.is_longest_match()
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
            pattern: data.0,
            accepting_states: data.1,
            state_ranges: data.2,
            transitions: data.3,
            current_state: 0,
            matching_state: MatchingState::new(),
        }
    }
}
