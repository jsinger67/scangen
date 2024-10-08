use std::collections::BTreeMap;

use crate::{Result, ScanGenError, ScanGenErrorKind};

use super::{
    character_class::ComparableAst,
    nfa::{EpsilonTransition, Nfa},
    parse_regex_syntax, CharClassID, CharacterClass, PatternID, StateID,
};

macro_rules! unsupported {
    ($feature:expr) => {
        ScanGenError::new($crate::ScanGenErrorKind::UnsupportedFeature(
            $feature.to_string(),
        ))
    };
}

/// A NFA that can match multiple pattern in parallel.
#[derive(Debug, Default)]
pub(crate) struct MultiPatternNfa {
    pub(crate) nfa: NfaWithCharClasses,
    pub(crate) pattern: Vec<String>,
    pub(crate) accepting_states: BTreeMap<StateID, PatternID>,
    pub(crate) char_classes: Vec<CharacterClass>,
}

impl MultiPatternNfa {
    /// Create a new multi-pattern NFA.
    pub fn new() -> Self {
        Self {
            nfa: NfaWithCharClasses::new(),
            pattern: Vec::new(),
            accepting_states: BTreeMap::new(),
            char_classes: Vec::new(),
        }
    }

    /// Get the NFA.
    pub fn nfa(&self) -> &NfaWithCharClasses {
        &self.nfa
    }

    /// Get an immutable reference of the pattern.
    pub fn pattern(&self) -> &[String] {
        &self.pattern
    }

    /// Get the accepting states.
    pub fn accepting_states(&self) -> &BTreeMap<StateID, PatternID> {
        &self.accepting_states
    }

    /// Get the character classes.
    pub(crate) fn char_classes(&self) -> &[CharacterClass] {
        &self.char_classes
    }

    /// Add a pattern to the multi-pattern NFA.
    pub fn add_pattern(&mut self, pattern: &str) -> Result<PatternID> {
        if let Some(id) = self.pattern.iter().position(|p| p == pattern) {
            // If the pattern already exists, return the terminal id
            // Not sure if this should rather be an error
            return Ok(PatternID::new(id));
        }

        let pattern_id = PatternID::new(self.pattern.len());
        let mut nfa: Nfa = parse_regex_syntax(pattern)?.try_into()?;
        nfa.set_pattern(pattern);
        self.pattern.push(pattern.to_string());

        // Shift the state ids of the given NFA
        nfa.shift_ids(self.nfa.states().len());

        // Add the end state of the given NFA to the accepting states of the own NFA along with the
        // pattern id
        self.accepting_states.insert(nfa.end_state(), pattern_id);

        // Add an epsilon transition from the start state of the own NFA to the start state of the
        // given NFA
        self.nfa
            .add_epsilon_transition(StateID::default(), nfa.start_state());

        // Move the states of the given NFA to the own NFA
        self.nfa.append(&mut self.char_classes, nfa);

        Ok(pattern_id)
    }

    /// Add multiple pattern to the multi-pattern NFA.
    #[allow(dead_code)]
    pub fn add_patterns<I, S>(&mut self, pattern: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for (index, pattern) in pattern.into_iter().enumerate() {
            let result = self.add_pattern(pattern.as_ref()).map(|_| ());
            if let Err(ScanGenError { source }) = &result {
                match &**source {
                    ScanGenErrorKind::RegexSyntaxError(_) => result?,
                    ScanGenErrorKind::UnsupportedFeature(s) => Err(unsupported!(format!(
                        "Error in pattern #{} '{}': {}",
                        index,
                        pattern.as_ref(),
                        s
                    )))?,
                    _ => result?,
                }
            } else {
                result?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct MultiNfaState {
    state: StateID,
    epsilon_transitions: Vec<EpsilonTransition>,
    transitions: Vec<MultiNfaTransition>,
}

impl MultiNfaState {
    pub(crate) fn id(&self) -> StateID {
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
    #[allow(dead_code)]
    pattern: String,
    states: Vec<MultiNfaState>,
}
impl NfaWithCharClasses {
    pub fn new() -> Self {
        Self {
            pattern: String::new(),
            states: vec![MultiNfaState::default()],
        }
    }

    pub(crate) fn states(&self) -> &[MultiNfaState] {
        &self.states
    }

    #[allow(dead_code)]
    pub(crate) fn pattern(&self) -> &str {
        &self.pattern
    }

    #[allow(dead_code)]
    pub(crate) fn set_pattern(&mut self, pattern: &str) {
        self.pattern = pattern.to_string();
    }

    pub(crate) fn add_epsilon_transition(&mut self, from: StateID, target_state: StateID) {
        self.states[from]
            .epsilon_transitions
            .push(EpsilonTransition { target_state });
    }

    // Move the states of the given NFA to the own NFA and thereby consume the given NFA.
    // Also we convert the character classes of the given NFA to CharacterClassIds.
    pub(crate) fn append(&mut self, char_classes: &mut Vec<CharacterClass>, nfa: Nfa) {
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
                    new_state.transitions.push(MultiNfaTransition {
                        chars: CharClassID::new(class_id),
                        target_state: transition.target_state(),
                    });
                } else {
                    let chars = CharClassID::new(char_classes.len());
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

    /// Calculate the epsilon closure of a state.
    pub(crate) fn epsilon_closure(&self, state: StateID) -> Vec<StateID> {
        // The state itself is always part of the ε-closure
        let mut closure = vec![state];
        let mut i = 0;
        while i < closure.len() {
            let current_state = closure[i];
            for epsilon_transition in self.states[current_state].epsilon_transitions() {
                if !closure.contains(&epsilon_transition.target_state()) {
                    closure.push(epsilon_transition.target_state());
                }
            }
            i += 1;
        }
        closure
    }

    /// Calculate the epsilon closure of a set of states and return the unique states.
    pub(crate) fn epsilon_closure_set<I>(&self, states: I) -> Vec<StateID>
    where
        I: IntoIterator<Item = StateID>,
    {
        let mut closure: Vec<StateID> = states.into_iter().collect();
        let mut i = 0;
        while i < closure.len() {
            let current_state = closure[i];
            for epsilon_transition in self.states[current_state].epsilon_transitions() {
                if !closure.contains(&epsilon_transition.target_state()) {
                    closure.push(epsilon_transition.target_state());
                }
            }
            i += 1;
        }
        closure.sort_unstable();
        closure.dedup();
        closure
    }

    /// Calculate move(T, a) for a set of states T and a character class a.
    /// This is the set of states that can be reached from T by matching a.
    pub(crate) fn move_set(&self, states: &[StateID], char_class: CharClassID) -> Vec<StateID> {
        let mut move_set = Vec::new();
        for state in states {
            for transition in self.states()[*state].transitions() {
                if transition.chars() == char_class {
                    move_set.push(transition.target_state());
                }
            }
        }
        move_set
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct MultiNfaTransition {
    // The characters to match
    chars: CharClassID,
    // The next state to transition to
    target_state: StateID,
}

impl MultiNfaTransition {
    pub(crate) fn target_state(&self) -> StateID {
        self.target_state
    }

    pub(crate) fn chars(&self) -> CharClassID {
        self.chars
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A macro that simplifies the rendering of a dot file for a multi-pattern NFA.
    macro_rules! multi_nfa_render_to {
        ($nfa:expr, $label:expr) => {
            let mut f = std::fs::File::create(format!("data/{}.dot", $label)).unwrap();
            $crate::compiletime::dot::multi_nfa_render($nfa, $label, &mut f);
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
        assert_eq!(multi_pattern_nfa.pattern(), &["a".to_string()]);
        assert_eq!(
            multi_pattern_nfa.accepting_states(),
            &[(StateID::new(2), pattern_id)].iter().cloned().collect()
        );

        multi_nfa_render_to!(&multi_pattern_nfa, "multi_a");

        let pattern_id = multi_pattern_nfa.add_pattern("b").unwrap();
        assert_eq!(
            multi_pattern_nfa.pattern(),
            &["a".to_string(), "b".to_string()]
        );

        multi_nfa_render_to!(&multi_pattern_nfa, "multi_a_or_b");

        assert_eq!(
            multi_pattern_nfa.accepting_states(),
            &[
                (StateID::new(2), PatternID::new(0)),
                (StateID::new(4), pattern_id)
            ]
            .iter()
            .cloned()
            .collect()
        );

        let pattern_id = multi_pattern_nfa.add_pattern("a").unwrap();
        // The pattern "a" already exists, so the terminal id should be the same as before
        assert_eq!(pattern_id, PatternID::new(0));

        multi_nfa_render_to!(&multi_pattern_nfa, "multi_a_or_b2");

        assert_eq!(
            multi_pattern_nfa.accepting_states(),
            &[
                (StateID::new(2), pattern_id),
                (StateID::new(4), PatternID::new(1))
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

        multi_nfa_render_to!(&multi_pattern_nfa, "multi_complex");

        assert_eq!(
            multi_pattern_nfa.accepting_states(),
            &[
                (StateID::new(11), PatternID::new(0)),
                (StateID::new(25), PatternID::new(1))
            ]
            .iter()
            .cloned()
            .collect()
        );
    }

    #[test]
    fn test_add_multiple_patterns() {
        init();

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

        multi_nfa_render_to!(&multi_pattern_nfa, "multiple_patterns");
    }
}
