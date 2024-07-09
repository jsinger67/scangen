#![allow(dead_code)]
use crate::{dfa::Dfa, match_function::MatchFunction, Result};

/// A compiled DFA that can be used to match a string.
/// The DFA is compiled from a DFA by creating match functions for all character classes.
/// The match functions are used to decide if a character is in a character class.
///
/// MatchFunctions are not Clone nor Copy, so we aggregate them into a new struct CompiledDfa
/// which is Clone and Copy neither.
pub(crate) struct CompiledDfa {
    // The base DFA
    dfa: Dfa,
    // The match functions for the DFA
    match_functions: Vec<MatchFunction>,
}

impl CompiledDfa {
    pub(crate) fn new(dfa: Dfa) -> Self {
        CompiledDfa {
            dfa,
            match_functions: Vec::new(),
        }
    }

    pub(crate) fn dfa(&self) -> &Dfa {
        &self.dfa
    }

    pub(crate) fn match_functions(&self) -> &[MatchFunction] {
        &self.match_functions
    }

    pub(crate) fn compile(&mut self) -> Result<()> {
        // Create the match functions for all character classes
        self.dfa
            .char_classes()
            .iter()
            .try_for_each(|char_class| -> Result<()> {
                let match_function = char_class.ast.0.clone().try_into()?;
                self.match_functions.push(match_function);
                Ok(())
            })?;
        Ok(())
    }
}

#[cfg(test)]

mod tests {}
