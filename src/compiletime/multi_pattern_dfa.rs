use regex_syntax::ast::Ast;

use crate::{Result, ScanGenError, ScanGenErrorKind};

use super::{compiled_dfa::CompiledDfa, dfa::Dfa, MatchFunction, MultiPatternNfa};

macro_rules! unsupported {
    ($feature:expr) => {
        ScanGenError::new($crate::ScanGenErrorKind::UnsupportedFeature(
            $feature.to_string(),
        ))
    };
}

/// The `MultiPatternDfa` struct represents a multi-pattern DFA.
/// The `MultiPatternDfa` struct can be used to match multiple pattern in parallel.
#[derive(Default)]
pub(crate) struct MultiPatternDfa {
    /// The DFAs that are used to match the pattern. Each DFA is used to match a single pattern.
    dfas: Vec<CompiledDfa>,
    /// The match functions shared by all DFAs.
    match_functions: Vec<(Ast, MatchFunction)>,
}

impl MultiPatternDfa {
    /// Creates a new `MultiPatternDfa` object.
    pub fn new() -> Self {
        MultiPatternDfa::default()
    }
    /// Returns the slice of Dfa objects that are used to match the pattern.
    #[allow(dead_code)]
    pub fn dfas(&self) -> &[CompiledDfa] {
        &self.dfas
    }

    /// Returns the number of pattern that are matched by the `MultiPatternDfa`.
    #[allow(dead_code)]
    pub fn num_patterns(&self) -> usize {
        self.dfas.len()
    }

    /// Add a pattern to the multi-pattern DFA.
    pub fn add_pattern<S>(&mut self, pattern: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        if self.dfas.iter().any(|d| d.pattern() == pattern.as_ref()) {
            // If the pattern already exists, do nothing.
            // Not sure if this should rather be an error.
            return Ok(());
        }

        let mut multi_pattern_nfa = MultiPatternNfa::new();
        multi_pattern_nfa.add_pattern(pattern.as_ref())?;

        // Convert the multi-pattern NFA to a DFA and minimize it.
        let dfa: Dfa = multi_pattern_nfa.try_into()?;
        let minimzed_dfa = dfa.minimize()?;

        // Compile the minimized DFA.
        let mut compiled_dfa = CompiledDfa::new();
        compiled_dfa.compile(&minimzed_dfa, &mut self.match_functions)?;

        // Add the compiled DFA to the list of DFAs.
        self.dfas.push(compiled_dfa);

        Ok(())
    }

    /// Add multiple pattern to the multi-pattern DFA.
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

    pub(crate) fn generate_code(&self, output: &mut dyn std::io::Write) -> Result<()> {
        writeln!(
            output,
            r"#![allow(clippy::manual_is_ascii_check)]

 use scangen::{{Dfa, DfaData, FindMatches, Scanner}};
 
 "
        )?;
        writeln!(output, "const DFAS: &[DfaData; {}] = &[", self.dfas.len())?;
        for (index, dfa) in self.dfas.iter().enumerate() {
            writeln!(output, "    /* {} */ ", index)?;
            dfa.generate_code(output)?;
        }
        writeln!(output, "];")?;
        writeln!(output)?;

        writeln!(
            output,
            "fn matches_char_class(c: char, char_class: usize) -> bool {{"
        )?;
        writeln!(output, "    match char_class {{")?;
        self.match_functions
            .iter()
            .enumerate()
            .try_for_each(|(i, (ast, _))| -> Result<()> {
                MatchFunction::generate_code(ast, i, output)?;
                Ok(())
            })?;
        writeln!(output, "        _ => false,")?;
        writeln!(output, "    }}")?;
        writeln!(
            output,
            r"}}

pub(crate) fn create_scanner() -> Scanner {{
    let dfas: Vec<Dfa> = DFAS.iter().map(|dfa| dfa.into()).collect();
    Scanner {{ dfas }}
}}

pub(crate) fn create_find_iter<'r, 'h>(
    scanner: &'r mut Scanner,
    input: &'h str,
) -> FindMatches<'r, 'h> {{
    scanner.find_iter(input, matches_char_class)
}}
"
        )?;
        Ok(())
    }
}

impl std::fmt::Debug for MultiPatternDfa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MultiPatternDfa {{ dfas: {:?} }}", self.dfas)
    }
}
