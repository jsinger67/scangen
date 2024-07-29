use regex_syntax::ast::Ast;

#[cfg(all(feature = "runtime", not(feature = "generate")))]
use crate::common::Span;
use crate::{
    common::MatchingState,
    compiletime::{
        character_class::ComparableAst, dfa::Dfa, match_function::MatchFunction, Result,
        ScanGenError,
    },
};

use super::{CharClassID, StateID};

/// A compiled DFA that can be used to match a string.
///
/// The DFA is compiled from a DFA by creating match functions for all character classes.
/// The match functions are used to decide if a character is in a character class.
/// Furthermore, the compile creates optimized data structures for the DFA to speed up matching.
///
/// MatchFunctions are not Clone nor Copy, so we aggregate them into a new struct CompiledDfa
/// which is Clone and Copy neither.
#[derive(Default)]
pub(crate) struct CompiledDfa {
    /// The pattern matched by the DFA.
    pattern: String,
    /// The accepting states of the DFA as well as the corresponding pattern id.
    accepting_states: Vec<StateID>,
    /// Each entry in the vector represents a state in the DFA. The entry is a tuple of first and
    /// last index into the transitions vector.
    state_ranges: Vec<(usize, usize)>,
    /// The transitions of the DFA. The indices that are relevant for a state are stored in the
    /// state_ranges vector.
    transitions: Vec<(CharClassID, StateID)>,
    /// The state of matching
    matching_state: MatchingState<StateID>,
}

impl CompiledDfa {
    pub fn new() -> Self {
        CompiledDfa::default()
    }

    pub(crate) fn pattern(&self) -> &str {
        &self.pattern
    }

    pub(crate) fn compile(
        &mut self,
        dfa: &Dfa,
        match_functions: &mut Vec<(Ast, MatchFunction)>,
    ) -> Result<()> {
        // Set the pattern
        debug_assert_eq!(dfa.pattern().len(), 1);
        self.pattern = dfa.pattern()[0].to_string();
        // Create the transitions vector as well as the state_ranges vector
        self.transitions.clear();
        self.state_ranges.clear();
        for _ in 0..dfa.states().len() {
            self.state_ranges.push((0, 0));
        }
        for (state, state_transitions) in dfa.transitions() {
            let start = self.transitions.len();
            self.state_ranges[*state] = (start, start + state_transitions.len());
            let mut transitions_for_state = state_transitions.iter().try_fold(
                Vec::new(),
                |mut acc, (char_class, target_state)| {
                    // Create the match function for the character class if it does not exist
                    if let Some(pos) = match_functions
                        .iter()
                        .position(|(ast, _)| ComparableAst(ast.clone()) == char_class.ast)
                    {
                        acc.push((pos.into(), *target_state));
                        Ok::<Vec<(CharClassID, StateID)>, ScanGenError>(acc)
                    } else {
                        let match_function: MatchFunction = char_class.ast().clone().try_into()?;
                        let new_char_class_id = CharClassID::new(match_functions.len());
                        match_functions.push((char_class.ast().clone(), match_function));
                        acc.push((new_char_class_id, *target_state));
                        Ok(acc)
                    }
                },
            )?;
            self.transitions.append(&mut transitions_for_state);
        }
        // Create the accepting states vector
        self.accepting_states = dfa.accepting_states().keys().cloned().collect();
        Ok(())
    }

    #[cfg(all(feature = "runtime", not(feature = "generate")))]
    pub(crate) fn matching_state(&self) -> &MatchingState<StateID> {
        &self.matching_state
    }

    #[cfg(all(feature = "runtime", not(feature = "generate")))]
    pub(crate) fn reset(&mut self) {
        self.matching_state = MatchingState::new();
    }

    #[cfg(all(feature = "runtime", not(feature = "generate")))]
    pub(crate) fn current_state(&self) -> StateID {
        self.matching_state.current_state()
    }

    #[cfg(all(feature = "runtime", not(feature = "generate")))]
    pub(crate) fn current_match(&self) -> Option<Span> {
        self.matching_state.last_match()
    }

    #[cfg(all(feature = "runtime", not(feature = "generate")))]
    pub(crate) fn advance(
        &mut self,
        c_pos: usize,
        c: char,
        match_functions: &[(Ast, MatchFunction)],
    ) {
        // If we already have the longest match, we can stop
        if self.matching_state.is_longest_match() {
            return;
        }
        // Get the transitions for the current state
        if let Some(next_state) = self.find_transition(c, match_functions) {
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

    #[cfg(all(feature = "runtime", not(feature = "generate")))]
    #[inline]
    fn find_transition(
        &self,
        c: char,
        match_functions: &[(Ast, MatchFunction)],
    ) -> Option<StateID> {
        let (start, end) = self.state_ranges[self.matching_state.current_state().as_usize()];
        let transitions = &self.transitions[start..end];
        for (_, (char_class, target_state)) in transitions {
            if match_functions[char_class.as_usize()].1.call(c) {
                return Some(*target_state);
            }
        }
        None
    }

    /// Returns true if the search should continue on the next character.
    #[cfg(all(feature = "runtime", not(feature = "generate")))]
    pub(crate) fn search_on(&self) -> bool {
        !self.matching_state.is_longest_match()
    }

    pub(crate) fn generate_code(&self, output: &mut dyn std::io::Write) -> Result<()> {
        write!(output, "    (\"{}\", &[", self.pattern.escape_default())?;
        for state in &self.accepting_states {
            write!(output, "{}, ", state.as_usize())?;
        }
        write!(output, "], &[")?;

        for (start, end) in &self.state_ranges {
            write!(output, "({}, {}), ", start, end)?;
        }
        write!(output, "], &[")?;
        for (char_class, target_state) in &self.transitions {
            write!(output, "({}, {}), ", char_class, target_state.as_usize())?;
        }
        writeln!(output, "]),")?;

        Ok(())
    }
}

impl std::fmt::Debug for CompiledDfa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompiledDfa")
            .field("accepting_states", &self.accepting_states)
            .field("state_ranges", &self.state_ranges)
            .field("transitions", &self.transitions)
            .field("current_state", &self.matching_state.current_state())
            .field("matching_state", &self.matching_state)
            .finish()
    }
}
