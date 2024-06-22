//! This module contains the NFA (Non-deterministic Finite Automaton) implementation.
//! The NFA is used to represent the regex syntax as a finite automaton.
//! The NFA is later converted to a DFA (Deterministic Finite Automaton) for matching strings.

use std::vec;

use regex_syntax::ast::Ast;

pub(crate) type StateId = usize;

#[derive(Debug, Clone)]
pub(crate) struct Nfa {
    states: Vec<NfaState>,
    // Used during NFA construction
    start_state: StateId,
    // Used during NFA construction
    end_state: StateId,
}

impl Nfa {
    pub(crate) fn new() -> Self {
        Self {
            states: vec![NfaState::new(0)],
            start_state: 0,
            end_state: 0,
        }
    }

    // Returns true if the NFA is empty, i.e. no states and no transitions have been added.
    pub(crate) fn is_empty(&self) -> bool {
        self.start_state == 0
            && self.end_state == 0
            && self.states.len() == 1
            && self.states[0].is_empty()
    }

    pub(crate) fn start_state(&self) -> usize {
        self.start_state
    }

    pub(crate) fn end_state(&self) -> usize {
        self.end_state
    }

    pub(crate) fn states(&self) -> &[NfaState] {
        &self.states
    }

    pub(crate) fn add_state(&mut self, state: NfaState) {
        self.states.push(state);
    }

    pub(crate) fn set_start_state(&mut self, state: usize) {
        self.start_state = state;
    }

    pub(crate) fn set_end_state(&mut self, state: usize) {
        self.end_state = state;
    }

    pub(crate) fn add_transition(&mut self, from: usize, chars: Ast, to: usize) {
        self.states[from].transitions.push(NfaTransition {
            chars,
            target_state: to,
        });
    }

    pub(crate) fn add_epsilon_transition(&mut self, from: usize, to: usize) {
        self.states[from]
            .epsilon_transitions
            .push(EpsilonTransition { target_state: to });
    }

    pub(crate) fn new_state(&mut self) -> usize {
        let state = self.states.len();
        self.add_state(NfaState::new(state));
        state
    }

    pub(crate) fn offset_states(&mut self, offset: usize) {
        for state in self.states.iter_mut() {
            state.offset(offset);
        }
        self.start_state += offset;
        self.end_state += offset;
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

        nfa.offset_states(self.states.len());

        // Move the states of the NFA being concatenated to the current NFA
        self.states.append(nfa.states.as_mut());

        // Connect the end state of the current NFA to the start state of the new NFA
        self.add_epsilon_transition(self.end_state, nfa.start_state);

        // Update the end state of the current NFA to the end state of the new NFA
        self.set_end_state(nfa.end_state);
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

        nfa.offset_states(self.states.len());

        // Move the states of the NFA being concatenated to the current NFA
        self.states.append(nfa.states.as_mut());

        // Create a new start state
        let start_state = self.new_state();
        // Connect the new start state to the start state of the current NFA
        self.add_epsilon_transition(start_state, self.start_state);
        // Connect the new start state to the start state of the new NFA
        self.add_epsilon_transition(start_state, nfa.start_state);

        // Create a new end state
        let end_state = self.new_state();
        // Connect the end state of the current NFA to the new end state
        self.add_epsilon_transition(self.end_state, end_state);
        // Connect the end state of the new NFA to the new end state
        self.add_epsilon_transition(nfa.end_state, end_state);

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
}

#[derive(Debug, Clone)]
pub(crate) struct NfaState {
    state: StateId,
    epsilon_transitions: Vec<EpsilonTransition>,
    transitions: Vec<NfaTransition>,
}

impl NfaState {
    pub(crate) fn new(state: usize) -> Self {
        Self {
            state,
            epsilon_transitions: Vec::new(),
            transitions: Vec::new(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.transitions.is_empty() && self.epsilon_transitions.is_empty()
    }

    pub(crate) fn state(&self) -> usize {
        self.state
    }

    pub(crate) fn transitions(&self) -> &[NfaTransition] {
        &self.transitions
    }

    pub(crate) fn epsilon_transitions(&self) -> &[EpsilonTransition] {
        &self.epsilon_transitions
    }

    /// Apply an offset to the state any number.
    pub(crate) fn offset(&mut self, offset: usize) {
        self.state += offset;
        for transition in self.transitions.iter_mut() {
            transition.target_state += offset;
        }
        for epsilon_transition in self.epsilon_transitions.iter_mut() {
            epsilon_transition.target_state += offset;
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct NfaTransition {
    // This can be a Literal or a CharacterClass
    // We will later generate a predicate from this that determines if a character matches this transition
    chars: Ast,
    // The next state to transition to
    target_state: StateId,
}

impl NfaTransition {
    pub(crate) fn target_state(&self) -> StateId {
        self.target_state
    }

    pub(crate) fn chars(&self) -> &Ast {
        &self.chars
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct EpsilonTransition {
    target_state: StateId,
}

impl EpsilonTransition {
    pub(crate) fn target_state(&self) -> StateId {
        self.target_state
    }
}

impl From<StateId> for EpsilonTransition {
    fn from(state: StateId) -> Self {
        Self {
            target_state: state,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{dot::render_to, parser::parse_regex_syntax};
    use std::fs::File;

    use super::*;

    #[test]
    fn test_nfa_from_ast() {
        // Create an example AST
        let ast = parse_regex_syntax("a").unwrap();

        // Convert the AST to an NFA
        let nfa: Nfa = ast.try_into().unwrap();

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 2);
        assert_eq!(nfa.start_state, 0);
        assert_eq!(nfa.end_state, 1);
    }

    #[test]
    fn test_nfa_from_ast_concat() {
        // Create an example AST and convert the AST to an NFA
        let nfa: Nfa = parse_regex_syntax("ab").unwrap().try_into().unwrap();

        let mut f = File::create("ab.dot").unwrap();
        render_to(&nfa, "ab", &mut f);

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 4);
        assert_eq!(nfa.start_state, 0);
        assert_eq!(nfa.end_state, 3);
    }

    #[test]
    fn test_nfa_concat() {
        // Create two example ASTs and convert them to an NFAs
        let mut nfa1: Nfa = parse_regex_syntax("a").unwrap().try_into().unwrap();
        let nfa2: Nfa = parse_regex_syntax("b").unwrap().try_into().unwrap();
        nfa1.concat(nfa2);

        // Add assertions here to validate the NFA
        assert_eq!(nfa1.states.len(), 4);
        assert_eq!(nfa1.start_state, 0);
        assert_eq!(nfa1.end_state, 3);
    }

    #[test]
    fn test_nfa_alternation() {
        // Create two example ASTs and convert them to an NFAs
        let mut nfa1: Nfa = parse_regex_syntax("a").unwrap().try_into().unwrap();
        let nfa2: Nfa = parse_regex_syntax("b").unwrap().try_into().unwrap();
        nfa1.alternation(nfa2);

        let mut f = File::create("a_or_b.dot").unwrap();
        render_to(&nfa1, "a|b", &mut f);

        // Add assertions here to validate the NFA
        assert_eq!(nfa1.states.len(), 6);
        assert_eq!(nfa1.start_state, 4);
        assert_eq!(nfa1.end_state, 5);
    }

    #[test]
    fn test_nfa_repetition() {
        // Create an example AST and convert the AST to an NFA
        let mut nfa: Nfa = parse_regex_syntax("a").unwrap().try_into().unwrap();
        nfa.zero_or_more();

        let mut f = File::create("a_many.dot").unwrap();
        render_to(&nfa, "a", &mut f);

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 4);
        assert_eq!(nfa.start_state, 2);
        assert_eq!(nfa.end_state, 3);
    }

    #[test]
    fn test_nfa_zero_or_one() {
        // Create an example AST and convert the AST to an NFA
        let mut nfa: Nfa = parse_regex_syntax("a").unwrap().try_into().unwrap();
        nfa.zero_or_one();

        let mut f = File::create("a_zero_or_one.dot").unwrap();
        render_to(&nfa, "a", &mut f);

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 3);
        assert_eq!(nfa.start_state, 2);
        assert_eq!(nfa.end_state, 1);
    }

    #[test]
    fn test_nfa_one_or_more() {
        // Create an example AST and convert the AST to an NFA
        let mut nfa: Nfa = parse_regex_syntax("a").unwrap().try_into().unwrap();
        nfa.one_or_more();

        let mut f = File::create("a_one_or_more.dot").unwrap();
        render_to(&nfa, "a", &mut f);

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 4);
        assert_eq!(nfa.start_state, 2);
        assert_eq!(nfa.end_state, 3);
    }

    #[test]
    fn test_nfa_zero_or_more() {
        // Create an example AST and convert the AST to an NFA
        let mut nfa: Nfa = parse_regex_syntax("a").unwrap().try_into().unwrap();
        nfa.zero_or_more();

        let mut f = File::create("a_zero_or_more.dot").unwrap();
        render_to(&nfa, "a", &mut f);

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 4);
        assert_eq!(nfa.start_state, 2);
        assert_eq!(nfa.end_state, 3);
    }

    #[test]
    fn test_complex_nfa() {
        // Create an example AST and convert the AST to an NFA
        let nfa: Nfa = parse_regex_syntax("(a|b)*abb").unwrap().try_into().unwrap();

        let mut f = File::create("complex.dot").unwrap();
        render_to(&nfa, "(a|b)*abb", &mut f);

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 14);
        assert_eq!(nfa.start_state, 6);
        assert_eq!(nfa.end_state, 13);
    }

    #[test]
    fn test_nfa_offset_states() {
        // Create an example AST and convert the AST to an NFA
        let mut nfa: Nfa = parse_regex_syntax("a").unwrap().try_into().unwrap();
        nfa.offset_states(10);

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 2);
        assert_eq!(nfa.start_state, 10);
        assert_eq!(nfa.end_state, 11);
    }
}
