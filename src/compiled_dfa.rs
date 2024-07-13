#![allow(dead_code)]

use regex_automata::{util::primitives::StateID, Span};

use crate::{character_class::CharClassID, dfa::Dfa, match_function::MatchFunction, Result};

/// A compiled DFA that can be used to match a string.
///
/// The DFA is compiled from a DFA by creating match functions for all character classes.
/// The match functions are used to decide if a character is in a character class.
/// Furthermore, the compile creates optimized data structures for the DFA to speed up matching.
///
/// MatchFunctions are not Clone nor Copy, so we aggregate them into a new struct CompiledDfa
/// which is Clone and Copy neither.
#[derive(Default)]
pub struct CompiledDfa {
    /// The pattern matched by the DFA.
    pattern: String,
    /// The accepting states of the DFA as well as the corresponding pattern id.
    accepting_states: Vec<StateID>,
    /// Each entry in the vector represents a state in the DFA. The entry is a tuple of first and
    /// last index into the transitions vector.
    state_ranges: Vec<(usize, usize)>,
    /// The transitions of the DFA.
    transitions: Vec<(StateID, (CharClassID, StateID))>,
    /// The match functions for the DFA
    match_functions: Vec<MatchFunction>,
    /// The current state of the DFA during matching
    current_state: StateID,
    /// The state of matching
    matching_state: MatchingState,
}

impl CompiledDfa {
    pub fn new() -> Self {
        CompiledDfa::default()
    }

    pub(crate) fn pattern(&self) -> &str {
        &self.pattern
    }

    pub(crate) fn match_functions(&self) -> &[MatchFunction] {
        &self.match_functions
    }

    pub(crate) fn compile(&mut self, dfa: &Dfa) -> Result<()> {
        // Set the pattern
        debug_assert_eq!(dfa.patterns().len(), 1);
        self.pattern = dfa.patterns()[0].to_string();
        // Create the match functions for all character classes
        self.match_functions.clear();
        dfa.char_classes()
            .iter()
            .try_for_each(|char_class| -> Result<()> {
                let match_function = char_class.ast.0.clone().try_into()?;
                self.match_functions.push(match_function);
                Ok(())
            })?;
        // Create the transitions vector as well as the state_ranges vector
        self.transitions.clear();
        self.state_ranges.clear();
        for _ in 0..dfa.states().len() {
            self.state_ranges.push((0, 0));
        }
        for (state, state_transitions) in dfa.transitions() {
            let start = self.transitions.len();
            self.state_ranges[*state] = (start, start + state_transitions.len());
            let mut transitions_for_state = state_transitions
                .iter()
                .map(|(char_class, target_state)| (*state, (char_class.id(), *target_state)))
                .collect::<Vec<_>>();
            self.transitions.append(&mut transitions_for_state);
        }
        // Create the accepting states vector
        self.accepting_states = dfa.accepting_states().keys().cloned().collect();
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
        if let Some(next_state) = self.find_transition(c) {
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

    #[inline]
    fn find_transition(&self, c: char) -> Option<StateID> {
        let (start, end) = self.state_ranges[self.current_state];
        let transitions = &self.transitions[start..end];
        for (_, (char_class, target_state)) in transitions {
            if self.match_functions[char_class.as_usize()].call(c) {
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

impl std::fmt::Debug for CompiledDfa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompiledDfa")
            .field("accepting_states", &self.accepting_states)
            .field("state_ranges", &self.state_ranges)
            .field("transitions", &self.transitions)
            .field("match_functions", &self.match_functions)
            .field("current_state", &self.current_state)
            .field("matching_state", &self.matching_state)
            .finish()
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

    /// No transition was found.
    /// See matching_state.dot for the state diagram
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

    /// Transition to a non-accepting state.
    /// See matching_state.dot for the state diagram
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

    /// Transition to an accepting state.
    /// See matching_state.dot for the state diagram
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
/// See matching_state.dot for the state diagram
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
