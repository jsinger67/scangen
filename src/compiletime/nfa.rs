//! This module contains the NFA (Non-deterministic Finite Automaton) implementation.
//! The NFA is used to represent the regex syntax as a finite automaton.
//! The NFA is later converted to a DFA (Deterministic Finite Automaton) for matching strings.

use std::vec;

use regex_syntax::ast::Ast;

use super::StateID;

#[derive(Debug, Clone, Default)]
pub(crate) struct Nfa {
    pub(crate) pattern: String,
    pub(crate) states: Vec<NfaState>,
    // Used during NFA construction
    pub(crate) start_state: StateID,
    // Used during NFA construction
    pub(crate) end_state: StateID,
}

impl Nfa {
    pub(crate) fn new() -> Self {
        Self {
            pattern: String::new(),
            states: vec![NfaState::default()],
            start_state: StateID::default(),
            end_state: StateID::default(),
        }
    }

    // Returns true if the NFA is empty, i.e. no states and no transitions have been added.
    pub(crate) fn is_empty(&self) -> bool {
        self.start_state == StateID::default()
            && self.end_state == StateID::default()
            && self.states.len() == 1
            && self.states[0].is_empty()
    }

    pub(crate) fn start_state(&self) -> StateID {
        self.start_state
    }

    pub(crate) fn end_state(&self) -> StateID {
        self.end_state
    }

    pub(crate) fn states(&self) -> &[NfaState] {
        &self.states
    }

    #[allow(dead_code)]
    pub(crate) fn pattern(&self) -> &str {
        &self.pattern
    }

    pub(crate) fn set_pattern(&mut self, pattern: &str) {
        self.pattern = pattern.to_string();
    }

    pub(crate) fn add_state(&mut self, state: NfaState) {
        self.states.push(state);
    }

    pub(crate) fn set_start_state(&mut self, state: StateID) {
        self.start_state = state;
    }

    pub(crate) fn set_end_state(&mut self, state: StateID) {
        self.end_state = state;
    }

    pub(crate) fn add_transition(&mut self, from: StateID, chars: Ast, target_state: StateID) {
        self.states[from].transitions.push(NfaTransition {
            chars,
            target_state,
        });
    }

    pub(crate) fn add_epsilon_transition(&mut self, from: StateID, target_state: StateID) {
        self.states[from]
            .epsilon_transitions
            .push(EpsilonTransition { target_state });
    }

    pub(crate) fn new_state(&mut self) -> StateID {
        let state = StateID::new(self.states.len());
        self.add_state(NfaState::new(state));
        state
    }

    /// Apply an offset to every state number.
    pub(crate) fn shift_ids(&mut self, offset: usize) -> (StateID, StateID) {
        for state in self.states.iter_mut() {
            state.offset(offset);
        }
        self.start_state = StateID::new(self.start_state.as_usize() + offset);
        self.end_state = StateID::new(self.end_state.as_usize() + offset);
        (self.start_state, self.end_state)
    }

    /// Concatenates the current NFA with another NFA.
    pub(crate) fn concat(&mut self, mut nfa: Nfa) {
        if self.is_empty() {
            // If the current NFA is empty, set the start and end states of the current NFA to the
            // start and end states of the new NFA
            self.set_start_state(nfa.start_state);
            self.set_end_state(nfa.end_state);
            self.states = nfa.states;
            return;
        }

        // Apply an offset to the state numbers of the given NFA
        let (nfa_start_state, nfa_end_state) = nfa.shift_ids(self.states.len());
        // Move the states of the given NFA to the current NFA
        self.append(nfa);

        // Connect the end state of the current NFA to the start state of the new NFA
        self.add_epsilon_transition(self.end_state, nfa_start_state);

        // Update the end state of the current NFA to the end state of the new NFA
        self.set_end_state(nfa_end_state);
    }

    pub(crate) fn alternation(&mut self, mut nfa: Nfa) {
        if self.is_empty() {
            // If the current NFA is empty, set the start and end states of the current NFA to the
            // start and end states of the new NFA
            self.set_start_state(nfa.start_state);
            self.set_end_state(nfa.end_state);
            self.states = nfa.states;
            return;
        }

        // Apply an offset to the state numbers of the given NFA
        let (nfa_start_state, nfa_end_state) = nfa.shift_ids(self.states.len());

        // Move the states of given the NFA to the current NFA
        self.append(nfa);

        // Create a new start state
        let start_state = self.new_state();
        // Connect the new start state to the start state of the current NFA
        self.add_epsilon_transition(start_state, self.start_state);
        // Connect the new start state to the start state of the new NFA
        self.add_epsilon_transition(start_state, nfa_start_state);

        // Create a new end state
        let end_state = self.new_state();
        // Connect the end state of the current NFA to the new end state
        self.add_epsilon_transition(self.end_state, end_state);
        // Connect the end state of the new NFA to the new end state
        self.add_epsilon_transition(nfa_end_state, end_state);

        // Update the start and end states of the current NFA
        self.set_start_state(start_state);
        self.set_end_state(end_state);
    }

    pub(crate) fn zero_or_one(&mut self) {
        // Create a new start state
        let start_state = self.new_state();
        // Connect the new start state to the start state of the current NFA
        self.add_epsilon_transition(start_state, self.start_state);
        // Connect the new start state to the end state of the current NFA
        self.add_epsilon_transition(start_state, self.end_state);

        // Update the start and end states of the current NFA
        self.set_start_state(start_state);
    }

    pub(crate) fn one_or_more(&mut self) {
        // Create a new start state
        let start_state = self.new_state();
        // Connect the new start state to the start state of the current NFA
        self.add_epsilon_transition(start_state, self.start_state);

        // Create a new end state
        let end_state = self.new_state();
        // Connect the end state of the current NFA to the new end state
        self.add_epsilon_transition(self.end_state, end_state);
        // Connect the end state of the current NFA to the start state of the current NFA
        self.add_epsilon_transition(self.end_state, self.start_state);

        // Update the start and end states of the current NFA
        self.set_start_state(start_state);
        self.set_end_state(end_state);
    }

    pub(crate) fn zero_or_more(&mut self) {
        // Create a new start state
        let start_state = self.new_state();
        // Connect the new start state to the start state of the current NFA
        self.add_epsilon_transition(start_state, self.start_state);
        // Connect the new start state to the end state of the current NFA
        self.add_epsilon_transition(start_state, self.end_state);

        // Create a new end state
        let end_state = self.new_state();
        // Connect the end state of the current NFA to the new end state
        self.add_epsilon_transition(self.end_state, end_state);
        // Connect the end state of the current NFA to the start state of the current NFA
        self.add_epsilon_transition(self.end_state, self.start_state);

        // Update the start and end states of the current NFA
        self.set_start_state(start_state);
        self.set_end_state(end_state);
    }

    /// Move the states of the given NFA to the current NFA and thereby consume the NFA.
    pub(crate) fn append(&mut self, mut nfa: Nfa) {
        self.states.append(nfa.states.as_mut());
        // Check the index constraints
        debug_assert!(self
            .states
            .iter()
            .enumerate()
            .all(|(i, s)| s.id().as_usize() == i));
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct NfaState {
    state: StateID,
    epsilon_transitions: Vec<EpsilonTransition>,
    transitions: Vec<NfaTransition>,
}

impl NfaState {
    pub(crate) fn new(state: StateID) -> Self {
        Self {
            state,
            epsilon_transitions: Vec::new(),
            transitions: Vec::new(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.transitions.is_empty() && self.epsilon_transitions.is_empty()
    }

    pub(crate) fn id(&self) -> StateID {
        self.state
    }

    pub(crate) fn transitions(&self) -> &[NfaTransition] {
        &self.transitions
    }

    pub(crate) fn epsilon_transitions(&self) -> &[EpsilonTransition] {
        &self.epsilon_transitions
    }

    /// Apply an offset to every state number.
    pub(crate) fn offset(&mut self, offset: usize) {
        self.state = StateID::new(self.state.as_usize() + offset);
        for transition in self.transitions.iter_mut() {
            transition.target_state = StateID::new(transition.target_state.as_usize() + offset);
        }
        for epsilon_transition in self.epsilon_transitions.iter_mut() {
            epsilon_transition.target_state =
                StateID::new(epsilon_transition.target_state.as_usize() + offset);
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct NfaTransition {
    // This can be a Literal or a CharacterClass
    // We will later generate a predicate from this that determines if a character matches this transition
    chars: Ast,
    // The next state to transition to
    target_state: StateID,
}

impl NfaTransition {
    pub(crate) fn target_state(&self) -> StateID {
        self.target_state
    }

    pub(crate) fn chars(&self) -> &Ast {
        &self.chars
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct EpsilonTransition {
    pub(crate) target_state: StateID,
}

impl EpsilonTransition {
    pub(crate) fn target_state(&self) -> StateID {
        self.target_state
    }
}

impl From<StateID> for EpsilonTransition {
    fn from(state: StateID) -> Self {
        Self {
            target_state: state,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{compiletime::parse_regex_syntax, nfa_render_to};

    use super::*;

    #[test]
    fn test_nfa_from_ast() {
        // Create an example AST
        let ast = parse_regex_syntax("a").unwrap();

        // Convert the AST to an NFA
        let nfa: Nfa = ast.try_into().unwrap();

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 2);
        assert_eq!(nfa.start_state.as_usize(), 0);
        assert_eq!(nfa.end_state.as_usize(), 1);
    }

    #[test]
    fn test_nfa_from_ast_concat() {
        // Create an example AST and convert the AST to an NFA
        let nfa: Nfa = parse_regex_syntax("ab").unwrap().try_into().unwrap();

        nfa_render_to!(&nfa, "ab");

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 4);
        assert_eq!(nfa.start_state.as_usize(), 0);
        assert_eq!(nfa.end_state.as_usize(), 3);
    }

    #[test]
    fn test_nfa_concat() {
        // Create two example ASTs and convert them to an NFAs
        let mut nfa1: Nfa = parse_regex_syntax("a").unwrap().try_into().unwrap();
        let nfa2: Nfa = parse_regex_syntax("b").unwrap().try_into().unwrap();
        nfa1.concat(nfa2);

        // Add assertions here to validate the NFA
        assert_eq!(nfa1.states.len(), 4);
        assert_eq!(nfa1.start_state.as_usize(), 0);
        assert_eq!(nfa1.end_state.as_usize(), 3);
    }

    #[test]
    fn test_nfa_alternation() {
        // Create two example ASTs and convert them to an NFAs
        let mut nfa1: Nfa = parse_regex_syntax("a").unwrap().try_into().unwrap();
        let nfa2: Nfa = parse_regex_syntax("b").unwrap().try_into().unwrap();
        nfa1.alternation(nfa2);

        nfa_render_to!(&nfa1, "a_or_b");

        // Add assertions here to validate the NFA
        assert_eq!(nfa1.states.len(), 6);
        assert_eq!(nfa1.start_state.as_usize(), 4);
        assert_eq!(nfa1.end_state.as_usize(), 5);
    }

    #[test]
    fn test_nfa_repetition() {
        // Create an example AST and convert the AST to an NFA
        let mut nfa: Nfa = parse_regex_syntax("a").unwrap().try_into().unwrap();
        nfa.zero_or_more();

        nfa_render_to!(&nfa, "a_many");

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 4);
        assert_eq!(nfa.start_state.as_usize(), 2);
        assert_eq!(nfa.end_state.as_usize(), 3);
    }

    #[test]
    fn test_nfa_zero_or_one() {
        // Create an example AST and convert the AST to an NFA
        let mut nfa: Nfa = parse_regex_syntax("a").unwrap().try_into().unwrap();
        nfa.zero_or_one();

        nfa_render_to!(&nfa, "a_zero_or_one");

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 3);
        assert_eq!(nfa.start_state.as_usize(), 2);
        assert_eq!(nfa.end_state.as_usize(), 1);
    }

    #[test]
    fn test_nfa_one_or_more() {
        // Create an example AST and convert the AST to an NFA
        let mut nfa: Nfa = parse_regex_syntax("a").unwrap().try_into().unwrap();
        nfa.one_or_more();

        nfa_render_to!(&nfa, "a_one_or_more");

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 4);
        assert_eq!(nfa.start_state.as_usize(), 2);
        assert_eq!(nfa.end_state.as_usize(), 3);
    }

    #[test]
    fn test_nfa_zero_or_more() {
        // Create an example AST and convert the AST to an NFA
        let mut nfa: Nfa = parse_regex_syntax("a").unwrap().try_into().unwrap();
        nfa.zero_or_more();

        nfa_render_to!(&nfa, "a_zero_or_more");

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 4);
        assert_eq!(nfa.start_state.as_usize(), 2);
        assert_eq!(nfa.end_state.as_usize(), 3);
    }

    #[test]
    fn test_complex_nfa() {
        // Create an example AST and convert the AST to an NFA
        let nfa: Nfa = parse_regex_syntax("(a|b)*abb").unwrap().try_into().unwrap();

        nfa_render_to!(&nfa, "complex");

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 14);
        assert_eq!(nfa.start_state.as_usize(), 6);
        assert_eq!(nfa.end_state.as_usize(), 13);
    }

    #[test]
    fn test_nfa_offset_states() {
        // Create an example AST and convert the AST to an NFA
        let mut nfa: Nfa = parse_regex_syntax("a").unwrap().try_into().unwrap();
        nfa.shift_ids(10);

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 2);
        assert_eq!(nfa.start_state.as_usize(), 10);
        assert_eq!(nfa.end_state.as_usize(), 11);
    }

    #[test]
    fn test_nfa_repetition_at_least() {
        // Create an example AST and convert the AST to an NFA
        let nfa: Nfa = parse_regex_syntax("a{3,}").unwrap().try_into().unwrap();

        nfa_render_to!(&nfa, "a_at_least_3");

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 10);
        assert_eq!(nfa.start_state.as_usize(), 0);
        assert_eq!(nfa.end_state.as_usize(), 9);
    }

    #[test]
    fn test_nfa_repetition_bounded() {
        // Create an example AST and convert the AST to an NFA
        let nfa: Nfa = parse_regex_syntax("a{3,5}").unwrap().try_into().unwrap();

        nfa_render_to!(&nfa, "a_bounded_3_5");

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 12);
        assert_eq!(nfa.start_state.as_usize(), 0);
        assert_eq!(nfa.end_state.as_usize(), 10);
    }

    // Ascii character class are not yet implemented
    #[test]
    fn test_character_class_expression() {
        // Create an example AST and convert the AST to an NFA
        let nfa: Nfa = parse_regex_syntax(r"[[:digit:]]")
            .unwrap()
            .try_into()
            .unwrap();

        nfa_render_to!(&nfa, "digit");

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 2);
        assert_eq!(nfa.start_state.as_usize(), 0);
        assert_eq!(nfa.end_state.as_usize(), 1);
    }
}
