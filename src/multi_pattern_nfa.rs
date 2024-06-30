use std::collections::BTreeMap;

use crate::{nfa::Nfa, parse_regex_syntax, Result, StateId, TerminalId};

/// A NFA that can match multiple patterns in parallel.
#[derive(Debug, Default)]
pub struct MultiPatternNfa {
    nfa: Nfa,
    patterns: Vec<String>,
    accepting_states: BTreeMap<StateId, TerminalId>,
}

impl MultiPatternNfa {
    /// Create a new multi-pattern NFA.
    pub fn new() -> Self {
        Self {
            nfa: Nfa::new(),
            patterns: Vec::new(),
            accepting_states: BTreeMap::new(),
        }
    }

    /// Get the NFA.
    pub fn nfa(&self) -> &Nfa {
        &self.nfa
    }

    /// Get the pattern.
    pub fn pattern(&self) -> &[String] {
        &self.patterns
    }

    /// Get the accepting states.
    pub fn accepting_states(&self) -> &BTreeMap<StateId, TerminalId> {
        &self.accepting_states
    }

    /// Add a pattern to the multi-pattern NFA.
    pub fn add_pattern(&mut self, pattern: &str) -> Result<TerminalId> {
        if let Some(id) = self.patterns.iter().position(|p| p == pattern) {
            // If the pattern already exists, return the terminal id
            // Not sure if this should rather be an error
            return Ok(TerminalId::new(id));
        }

        let terminal_id = TerminalId::new(self.patterns.len());
        let mut nfa: Nfa = parse_regex_syntax(pattern)?.try_into()?;
        self.patterns.push(pattern.to_string());

        // Shift the state ids of the given NFA
        nfa.shift_ids(self.nfa.states().len());

        // Add the end state of the given NFA to the accepting states of the own NFA along with the
        // terminal id
        self.accepting_states.insert(nfa.end_state(), terminal_id);

        // Add an epsilon transition from the start state of the own NFA to the start state of the
        // given NFA
        self.nfa
            .add_epsilon_transition(self.nfa.start_state(), nfa.start_state());

        // Move the states of the given NFA to the own NFA
        self.nfa.append(nfa);

        Ok(terminal_id)
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

    #[test]
    fn test_multi_pattern_nfa() {
        let mut multi_pattern_nfa = MultiPatternNfa::new();
        let terminal_id = multi_pattern_nfa.add_pattern("a").unwrap();
        assert_eq!(multi_pattern_nfa.pattern(), &["a".to_string()]);
        assert_eq!(
            multi_pattern_nfa.accepting_states(),
            &[(StateId::new(2), terminal_id)].iter().cloned().collect()
        );

        multi_render_to!(&multi_pattern_nfa, "multi_a");

        let terminal_id = multi_pattern_nfa.add_pattern("b").unwrap();
        assert_eq!(
            multi_pattern_nfa.pattern(),
            &["a".to_string(), "b".to_string()]
        );

        multi_render_to!(&multi_pattern_nfa, "multi_a_or_b");

        assert_eq!(
            multi_pattern_nfa.accepting_states(),
            &[
                (StateId::new(2), TerminalId::new(0)),
                (StateId::new(4), terminal_id)
            ]
            .iter()
            .cloned()
            .collect()
        );

        let terminal_id = multi_pattern_nfa.add_pattern("a").unwrap();
        // The pattern "a" already exists, so the terminal id should be the same as before
        assert_eq!(terminal_id, TerminalId::new(0));

        multi_render_to!(&multi_pattern_nfa, "multi_a_or_b2");

        assert_eq!(
            multi_pattern_nfa.accepting_states(),
            &[
                (StateId::new(2), terminal_id),
                (StateId::new(4), TerminalId::new(1))
            ]
            .iter()
            .cloned()
            .collect()
        );
    }

    #[test]
    fn test_multi_pattern_nfa_2() {
        let mut multi_pattern_nfa = MultiPatternNfa::new();
        let result = multi_pattern_nfa.add_pattern("b|a{2,3}");
        assert!(result.is_ok());
        let result = multi_pattern_nfa.add_pattern("(a|b)*abb");
        assert!(result.is_ok());

        multi_render_to!(&multi_pattern_nfa, "multi_complex");

        assert_eq!(
            multi_pattern_nfa.accepting_states(),
            &[
                (StateId::new(11), TerminalId::new(0)),
                (StateId::new(25), TerminalId::new(1))
            ]
            .iter()
            .cloned()
            .collect()
        );
    }
}
