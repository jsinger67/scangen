use std::collections::BTreeMap;

use log::trace;

use crate::{
    character_class::{CharacterClass, ComparableAst},
    nfa::{EpsilonTransition, Nfa},
    parse_regex_syntax, CharClassId, PatternId, Result, StateId,
};

/// A NFA that can match multiple patterns in parallel.
#[derive(Debug, Default)]
pub struct MultiPatternNfa {
    nfa: NfaWithCharClasses,
    patterns: Vec<String>,
    accepting_states: BTreeMap<StateId, PatternId>,
    char_classes: Vec<CharacterClass>,
}

impl MultiPatternNfa {
    /// Create a new multi-pattern NFA.
    pub fn new() -> Self {
        Self {
            nfa: NfaWithCharClasses::new(),
            patterns: Vec::new(),
            accepting_states: BTreeMap::new(),
            char_classes: Vec::new(),
        }
    }

    /// Get the NFA.
    pub fn nfa(&self) -> &NfaWithCharClasses {
        &self.nfa
    }

    /// Get an immutable reference of the patterns.
    pub fn patterns(&self) -> &[String] {
        &self.patterns
    }

    /// Get the accepting states.
    pub fn accepting_states(&self) -> &BTreeMap<StateId, PatternId> {
        &self.accepting_states
    }

    /// Get the character classes.
    pub(crate) fn char_classes(&self) -> &[CharacterClass] {
        &self.char_classes
    }

    /// Add a pattern to the multi-pattern NFA.
    pub fn add_pattern(&mut self, pattern: &str) -> Result<PatternId> {
        if let Some(id) = self.patterns.iter().position(|p| p == pattern) {
            // If the pattern already exists, return the terminal id
            // Not sure if this should rather be an error
            return Ok(PatternId::new(id));
        }

        let pattern_id = PatternId::new(self.patterns.len());
        let mut nfa: Nfa = parse_regex_syntax(pattern)?.try_into()?;
        self.patterns.push(pattern.to_string());

        // Shift the state ids of the given NFA
        nfa.shift_ids(self.nfa.states().len());

        // Add the end state of the given NFA to the accepting states of the own NFA along with the
        // terminal id
        self.accepting_states.insert(nfa.end_state(), pattern_id);

        // Add an epsilon transition from the start state of the own NFA to the start state of the
        // given NFA
        self.nfa
            .add_epsilon_transition(StateId::default(), nfa.start_state());

        // Move the states of the given NFA to the own NFA
        self.nfa.append(&mut self.char_classes, nfa);

        Ok(pattern_id)
    }
}
#[derive(Debug, Clone, Default)]
pub(crate) struct MultiNfaState {
    state: StateId,
    epsilon_transitions: Vec<EpsilonTransition>,
    transitions: Vec<MultiNfaTransition>,
}

impl MultiNfaState {
    pub(crate) fn id(&self) -> StateId {
        self.state
    }

    pub(crate) fn epsilon_transitions(&self) -> &[EpsilonTransition] {
        &self.epsilon_transitions
    }

    pub(crate) fn transitions(&self) -> &[MultiNfaTransition] {
        &self.transitions
    }
}

#[derive(Debug, Clone, Default)]
pub struct NfaWithCharClasses {
    states: Vec<MultiNfaState>,
}
impl NfaWithCharClasses {
    fn new() -> Self {
        Self {
            states: vec![MultiNfaState::default()],
        }
    }

    pub(crate) fn states(&self) -> &[MultiNfaState] {
        &self.states
    }

    pub(crate) fn add_epsilon_transition(&mut self, from: StateId, target_state: StateId) {
        self.states[from.as_index()]
            .epsilon_transitions
            .push(EpsilonTransition { target_state });
    }

    // Move the states of the given NFA to the own NFA and thereby consume the given NFA.
    // Also we convert the character classes of the given NFA to CharacterClassIds.
    fn append(&mut self, char_classes: &mut Vec<CharacterClass>, nfa: Nfa) {
        nfa.states().iter().for_each(|state| {
            let mut new_state = MultiNfaState {
                state: state.id(),
                ..Default::default()
            };
            state
                .epsilon_transitions()
                .iter()
                .for_each(|epsilon_transition| {
                    new_state.epsilon_transitions.push(EpsilonTransition {
                        target_state: epsilon_transition.target_state(),
                    });
                });
            state.transitions().iter().for_each(|transition| {
                if let Some(class_id) = char_classes
                    .iter()
                    .position(|c| c.ast == ComparableAst(transition.chars().clone()))
                {
                    trace!("Found existing char class: {:?}", transition.chars());
                    new_state.transitions.push(MultiNfaTransition {
                        chars: CharClassId::new(class_id),
                        target_state: transition.target_state(),
                    });
                } else {
                    trace!("Adding new char class: {:?}", transition.chars());
                    let chars = CharClassId::new(char_classes.len());
                    char_classes.push(CharacterClass::new(chars, transition.chars().clone()));
                    new_state.transitions.push(MultiNfaTransition {
                        chars,
                        target_state: transition.target_state(),
                    });
                }
            });
            self.states.push(new_state);
        });
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MultiNfaTransition {
    // The characters to match
    chars: CharClassId,
    // The next state to transition to
    target_state: StateId,
}

impl MultiNfaTransition {
    pub(crate) fn target_state(&self) -> StateId {
        self.target_state
    }

    pub(crate) fn chars(&self) -> CharClassId {
        self.chars
    }
}

#[cfg(test)]
mod tests {
    use crate::dot::multi_render_to;
    use std::fs::File;

    use super::*;

    // A macro that simplifies the rendering of a dot file for test purposes
    macro_rules! multi_render_to {
        ($nfa:expr, $label:expr) => {
            let mut f = File::create(concat!($label, ".dot")).unwrap();
            multi_render_to($nfa, $label, &mut f);
        };
    }

    // Initialize the logger for the tests
    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_multi_pattern_nfa() {
        init();

        let mut multi_pattern_nfa = MultiPatternNfa::new();
        let pattern_id = multi_pattern_nfa.add_pattern("a").unwrap();
        assert_eq!(multi_pattern_nfa.patterns(), &["a".to_string()]);
        assert_eq!(
            multi_pattern_nfa.accepting_states(),
            &[(StateId::new(2), pattern_id)].iter().cloned().collect()
        );

        multi_render_to!(&multi_pattern_nfa, "multi_a");

        let pattern_id = multi_pattern_nfa.add_pattern("b").unwrap();
        assert_eq!(
            multi_pattern_nfa.patterns(),
            &["a".to_string(), "b".to_string()]
        );

        multi_render_to!(&multi_pattern_nfa, "multi_a_or_b");

        assert_eq!(
            multi_pattern_nfa.accepting_states(),
            &[
                (StateId::new(2), PatternId::new(0)),
                (StateId::new(4), pattern_id)
            ]
            .iter()
            .cloned()
            .collect()
        );

        let pattern_id = multi_pattern_nfa.add_pattern("a").unwrap();
        // The pattern "a" already exists, so the terminal id should be the same as before
        assert_eq!(pattern_id, PatternId::new(0));

        multi_render_to!(&multi_pattern_nfa, "multi_a_or_b2");

        assert_eq!(
            multi_pattern_nfa.accepting_states(),
            &[
                (StateId::new(2), pattern_id),
                (StateId::new(4), PatternId::new(1))
            ]
            .iter()
            .cloned()
            .collect()
        );
    }

    #[test]
    fn test_multi_pattern_nfa_2() {
        init();

        let mut multi_pattern_nfa = MultiPatternNfa::new();
        let result = multi_pattern_nfa.add_pattern("b|a{2,3}");
        assert!(result.is_ok());
        let result = multi_pattern_nfa.add_pattern("(a|b)*abb");
        assert!(result.is_ok());

        multi_render_to!(&multi_pattern_nfa, "multi_complex");

        assert_eq!(
            multi_pattern_nfa.accepting_states(),
            &[
                (StateId::new(11), PatternId::new(0)),
                (StateId::new(25), PatternId::new(1))
            ]
            .iter()
            .cloned()
            .collect()
        );
    }
}
