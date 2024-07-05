//! This module contains the DFA implementation.
//! The DFA is used to match a string against a regex pattern.
//! The DFA is generated from the NFA using the subset construction algorithm.

use std::collections::BTreeMap;

use crate::{character_class::CharacterClass, MultiPatternNfa, PatternId, StateId};

/// The DFA implementation.
#[derive(Debug, Default)]
pub struct Dfa {
    // The states of the DFA.
    states: Vec<DfaState>,
    // The patterns for the accepting states.
    patterns: Vec<String>,
    // The accepting states of the DFA as well as the corresponding pattern id.
    accepting_states: BTreeMap<StateId, PatternId>,
    // The character classes used in the DFA.
    char_classes: Vec<CharacterClass>,
    // The transitions of the DFA.
    transitions: BTreeMap<StateId, BTreeMap<CharacterClass, StateId>>,
}

impl Dfa {
    /// Get the states of the DFA.
    pub(crate) fn states(&self) -> &[DfaState] {
        &self.states
    }

    /// Get the patterns for the accepting states.
    pub(crate) fn patterns(&self) -> &[String] {
        &self.patterns
    }

    /// Get the accepting states of the DFA.
    pub(crate) fn accepting_states(&self) -> &BTreeMap<StateId, PatternId> {
        &self.accepting_states
    }

    /// Get the character classes used in the DFA.
    pub(crate) fn char_classes(&self) -> &[CharacterClass] {
        &self.char_classes
    }

    /// Get the transitions of the DFA.
    pub(crate) fn transitions(&self) -> &BTreeMap<StateId, BTreeMap<CharacterClass, StateId>> {
        &self.transitions
    }

    /// Create a DFA from a multi-pattern NFA.
    /// The DFA is created using the subset construction algorithm.
    fn from_nfa(nfa: MultiPatternNfa) -> Self {
        let MultiPatternNfa {
            nfa,
            patterns,
            accepting_states,
            char_classes,
        } = nfa;
        let mut dfa = Dfa {
            states: Vec::new(),
            patterns,
            accepting_states: BTreeMap::new(),
            char_classes,
            transitions: BTreeMap::new(),
        };
        // The initial state of the DFA is the epsilon closure of the start state of the NFA.
        let start_state = nfa.epsilon_closure(StateId::default());
        // The initial state is the start state of the DFA.
        let initial_state = dfa.add_state(start_state, &accepting_states);
        // The work list is used to keep track of the states that need to be processed.
        let mut work_list = vec![initial_state];
        // The marked flag is used to mark a state as visited during the subset construction algorithm.
        dfa.states[initial_state.as_index()].marked = true;

        while let Some(state_id) = work_list.pop() {
            let nfa_states = dfa.states[state_id.as_index()].nfa_states.clone();
            for char_class in dfa.char_classes.clone() {
                let target_states =
                    nfa.epsilon_closure_set(nfa.move_set(&nfa_states, char_class.id()));
                if !target_states.is_empty() {
                    let target_state = dfa.add_state(target_states, &accepting_states);
                    dfa.transitions
                        .entry(state_id)
                        .or_default()
                        .insert(char_class.clone(), target_state);
                    if !dfa.states[target_state.as_index()].marked {
                        dfa.states[target_state.as_index()].marked = true;
                        work_list.push(target_state);
                    }
                }
            }
        }

        dfa
    }

    /// Add a state to the DFA if it does not already exist.
    /// The state is identified by the NFA states that constitute the DFA state.
    /// The accepting states are used to determine if the DFA state is an accepting state.
    fn add_state(
        &mut self,
        mut nfa_states: Vec<StateId>,
        accepting_states: &BTreeMap<StateId, PatternId>,
    ) -> StateId {
        nfa_states.sort_unstable();
        nfa_states.dedup();
        if let Some(state_id) = self
            .states
            .iter()
            .position(|state| state.nfa_states == nfa_states)
        {
            return StateId::new(state_id);
        }

        let state_id = self.states.len();
        let state = DfaState::new(state_id.into(), nfa_states);

        // Check if the constraint holds that only one pattern can match, i.e. the DFA
        // state only contains one accpting NFA state. This should always be the case since
        // the NFA is a multi-pattern NFA.
        debug_assert!(
            state
                .nfa_states
                .iter()
                .filter(|nfa_state_id| accepting_states.contains_key(nfa_state_id))
                .count()
                <= 1
        );

        // Check if the state contains an accepting state.
        for nfa_state_id in &state.nfa_states {
            if let Some(pattern_id) = accepting_states.get(nfa_state_id) {
                // The state is an accepting state.
                self.accepting_states
                    .insert(StateId::new(state_id), *pattern_id);
                break;
            }
        }

        self.states.push(state);
        StateId::new(state_id)
    }
}

impl From<MultiPatternNfa> for Dfa {
    fn from(nfa: MultiPatternNfa) -> Self {
        Dfa::from_nfa(nfa)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct DfaState {
    id: StateId,
    // The ids of the NFA states that constitute this DFA state. The id can only be used as indices
    // into the NFA states.
    nfa_states: Vec<StateId>,
    // The marked flag is used to mark a state as visited during the subset construction algorithm.
    marked: bool,
}

impl DfaState {
    /// Create a new DFA state solely from the NFA states that constitute the DFA state.
    pub(crate) fn new(id: StateId, nfa_states: Vec<StateId>) -> Self {
        DfaState {
            id,
            nfa_states,
            marked: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dot::{dfa_render_to, multi_render_to};
    use std::fs::File;

    // A macro that simplifies the rendering of a dot file for test purposes
    macro_rules! dfa_render_to {
        ($nfa:expr, $label:expr) => {
            let mut f = File::create(concat!($label, ".dot")).unwrap();
            dfa_render_to($nfa, $label, &mut f);
        };
    }

    // A macro that simplifies the rendering of a dot file for test purposes
    macro_rules! multi_render_to {
        ($nfa:expr, $label:expr) => {
            let mut f = File::create(concat!($label, ".dot")).unwrap();
            multi_render_to($nfa, $label, &mut f);
        };
    }

    #[test]
    fn test_dfa_from_nfa() {
        let mut multi_pattern_nfa = MultiPatternNfa::new();
        let result = multi_pattern_nfa.add_pattern("b|a{2,3}");
        assert!(result.is_ok());
        let result = multi_pattern_nfa.add_pattern("(a|b)*abb");
        assert!(result.is_ok());
        multi_render_to!(&multi_pattern_nfa, "input_nfa");

        let dfa = Dfa::from(multi_pattern_nfa);

        dfa_render_to!(&dfa, "dfa_from_nfa");

        assert_eq!(dfa.states().len(), 9);
        assert_eq!(dfa.patterns().len(), 2);
        assert_eq!(dfa.accepting_states().len(), 4);
        assert_eq!(dfa.char_classes().len(), 2);
    }

    #[test]
    fn test_dfa_from_nfa_2() {
        let mut multi_pattern_nfa = MultiPatternNfa::new();
        let result = multi_pattern_nfa.add_pattern("in");
        assert!(result.is_ok());
        let result = multi_pattern_nfa.add_pattern("int");
        assert!(result.is_ok());

        let dfa = Dfa::from(multi_pattern_nfa);

        dfa_render_to!(&dfa, "dfa_from_nfa_2");

        assert_eq!(dfa.states().len(), 4);
        assert_eq!(dfa.patterns().len(), 2);
        assert_eq!(dfa.accepting_states().len(), 2);
        assert_eq!(dfa.char_classes().len(), 3);
    }

    #[test]
    fn test_dfa_from_nfa_3() {
        let mut multi_pattern_nfa = MultiPatternNfa::new();
        let result = multi_pattern_nfa.add_patterns(vec![
            "\\r\\n|\\r|\\n",
            "[\\s--\\r\\n]+",
            "(//.*(\\r\\n|\\r|\\n))",
            "(/\\*.*?\\*/)",
            "%start",
            "%title",
            "%comment",
            "%user_type",
            "=",
            "%grammar_type",
            "%line_comment",
            "%block_comment",
            "%auto_newline_off",
            "%auto_ws_off",
            "%on",
            "%enter",
            "%%",
            "::",
            ":",
            ";",
            "\\|",
            "<",
            ">",
            "\"(\\\\.|[^\\\\])*?\"",
            "'(\\\\'|[^'])*?'",
            "\\u{2F}(\\\\.|[^\\\\])*?\\u{2F}",
            "\\(",
            "\\)",
            "\\[",
            "\\]",
            "\\{",
            "\\}",
            "[a-zA-Z_][a-zA-Z0-9_]*",
            "%scanner",
            ",",
            "%sc",
            "%push",
            "%pop",
            "\\^",
            ".",
        ]);
        match result {
            Ok(_) => {}
            Err(e) => {
                panic!("Error: {}", e);
            }
        }

        let dfa = Dfa::from(multi_pattern_nfa);

        dfa_render_to!(&dfa, "dfa_from_nfa_3");

        assert_eq!(dfa.states().len(), 154);
        assert_eq!(dfa.patterns().len(), 40);
        assert_eq!(dfa.accepting_states().len(), 45);
        assert_eq!(dfa.char_classes().len(), 50);
    }
}
