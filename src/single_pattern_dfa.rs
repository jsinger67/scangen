use std::collections::{BTreeMap, BTreeSet};

use log::trace;
use regex_automata::util::primitives::StateID;

use crate::{
    character_class::CharacterClass, dfa::DfaState, errors::DfaError, MultiPatternNfa, Result,
    ScanGenError, ScanGenErrorKind,
};

/// A DFA type that can be used to match a single pattern.
#[derive(Debug, Clone, Default)]
pub struct SinglePatternDfa {
    // The states of the DFA. The start state is always the first state in the vector, i.e. state 0.
    states: Vec<DfaState>,
    // The pattern this DFA can match.
    pattern: String,
    // The accepting states of the DFA.
    accepting_states: BTreeSet<StateID>,
    // The character classes used in the DFA.
    char_classes: Vec<CharacterClass>,
    // The transitions of the DFA.
    transitions: BTreeMap<StateID, BTreeMap<CharacterClass, StateID>>,
}

impl SinglePatternDfa {
    /// Create a new DFA with the given pattern.
    pub fn new(pattern: String) -> Self {
        Self {
            states: Vec::new(),
            pattern,
            accepting_states: BTreeSet::new(),
            char_classes: Vec::new(),
            transitions: BTreeMap::new(),
        }
    }

    /// Add a state to the DFA.
    pub fn add_state(&mut self, state: DfaState) {
        self.states.push(state);
    }

    /// Add an accepting state to the DFA.
    pub fn add_accepting_state(&mut self, state_id: StateID) {
        self.accepting_states.insert(state_id);
    }

    /// Add a character class to the DFA.
    pub fn add_char_class(&mut self, char_class: CharacterClass) {
        self.char_classes.push(char_class);
    }

    /// Add a transition to the DFA.
    pub fn add_transition(&mut self, from: StateID, on: CharacterClass, to: StateID) {
        self.transitions.entry(from).or_default().insert(on, to);
    }

    /// Get the pattern this DFA can match.
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Get the states of the DFA.
    pub fn states(&self) -> &[DfaState] {
        &self.states
    }

    /// Get the accepting states of the DFA.
    pub fn accepting_states(&self) -> &BTreeSet<StateID> {
        &self.accepting_states
    }

    /// Get the character classes used in the DFA.
    pub fn char_classes(&self) -> &[CharacterClass] {
        &self.char_classes
    }

    /// Get the transitions of the DFA.
    pub fn transitions(&self) -> &BTreeMap<StateID, BTreeMap<CharacterClass, StateID>> {
        &self.transitions
    }

    /// Add a state to the DFA if it does not already exist.
    /// The state is identified by the NFA states that constitute the DFA state.
    /// The accepting states are used to determine if the DFA state is an accepting state.
    fn add_state_if_new<I>(
        &mut self,
        nfa_states: I,
        accepting_states: &[StateID],
    ) -> Result<StateID>
    where
        I: IntoIterator<Item = StateID>,
    {
        let mut nfa_states: Vec<StateID> = nfa_states.into_iter().collect();
        nfa_states.sort_unstable();
        nfa_states.dedup();
        if let Some(state_id) = self
            .states
            .iter()
            .position(|state| state.nfa_states() == nfa_states)
        {
            return Ok(StateID::new(state_id)?);
        }

        let state_id = StateID::new(self.states.len())?;
        let state = DfaState::new(state_id, nfa_states);

        // Check if the constraint holds that only one pattern can match, i.e. the DFA
        // state only contains one accpting NFA state. This should always be the case since
        // the NFA is a multi-pattern NFA.
        debug_assert!(
            state
                .nfa_states()
                .iter()
                .filter(|nfa_state_id| accepting_states.contains(nfa_state_id))
                .count()
                <= 1
        );

        // Check if the state contains an accepting state.
        for nfa_state_id in state.nfa_states() {
            if accepting_states.contains(nfa_state_id) {
                // The state is an accepting state.
                self.add_accepting_state(state_id);
                break;
            }
        }

        trace!(
            "Add state: {}: {:?}",
            state.id().as_usize(),
            state.nfa_states()
        );

        self.states.push(state);
        Ok(state_id)
    }

    /// Create a DFA from a multi-pattern NFA.
    /// The DFA is created using the subset construction algorithm.
    /// The multi-pattern NFA must only contain a single pattern.
    fn try_from_nfa(nfa: MultiPatternNfa) -> Result<Self> {
        let MultiPatternNfa {
            nfa,
            patterns,
            accepting_states,
            char_classes,
        } = nfa;
        if patterns.len() != 1 {
            return Err(ScanGenError::new(ScanGenErrorKind::DfaError(
                DfaError::SinglePatternDfaError(
                    "Only single pattern NFAs are supported".to_string(),
                ),
            )));
        }
        let accepting_states: Vec<StateID> = accepting_states.into_iter().map(|a| a.0).collect();
        let mut dfa = SinglePatternDfa::new(patterns[0].clone());
        dfa.char_classes = char_classes;
        // The initial state of the DFA is the epsilon closure of the start state of the NFA.
        let start_state = nfa.epsilon_closure(StateID::default());
        // The initial state is the start state of the DFA.
        let initial_state = dfa.add_state_if_new(start_state, &accepting_states)?;
        // The work list is used to keep track of the states that need to be processed.
        let mut work_list = vec![initial_state];
        // The marked flag is used to mark a state as visited during the subset construction algorithm.
        dfa.states[initial_state].set_marked(true);

        while let Some(state_id) = work_list.pop() {
            let nfa_states: Vec<StateID> = dfa.states[state_id].nfa_states().to_vec();
            for char_class in dfa.char_classes.clone() {
                let target_states =
                    nfa.epsilon_closure_set(nfa.move_set(&nfa_states, char_class.id()));
                if !target_states.is_empty() {
                    let target_state = dfa.add_state_if_new(target_states, &accepting_states)?;
                    dfa.transitions
                        .entry(state_id)
                        .or_default()
                        .insert(char_class.clone(), target_state);
                    if !dfa.states[target_state].marked() {
                        dfa.states[target_state].set_marked(true);
                        work_list.push(target_state);
                    }
                }
            }
        }

        Ok(dfa)
    }
}
