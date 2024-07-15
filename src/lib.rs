#![forbid(missing_docs)]
//! The `scangen` crate provides a library for generating code from a regex syntax.
//! The crate should fill a gap in the regex ecosystem by providing a way to generate code from a
//! regex syntax.

/// The parser module contains the regex syntax parser.
mod parser;
pub use parser::parse_regex_syntax;

/// The generator module contains the code generator.
/// The code generator generates code from the regex syntax.
mod generator;
pub use generator::generate_code;

/// The nfa module contains the NFA implementation.
mod nfa;

/// The module containing the conversions from Ast to Nfa
mod ast;

/// Module with conversion to graphviz dot format
mod dot;
pub use dot::{dfa_render_to, multi_nfa_render_to, nfa_render_to, single_dfa_render_to};

/// Module with error definitions
mod errors;
pub use errors::{RegexAutomataError, Result, ScanGenError, ScanGenErrorKind};

/// Module that provides a type for a multi-pattern NFA
/// that can be used to match multiple patterns in parallel.
mod multi_pattern_nfa;
pub use multi_pattern_nfa::MultiPatternNfa;

/// Module that provides a type for a single-pattern DFA
/// that can be used to match a single pattern.
pub mod single_pattern_dfa;
pub use single_pattern_dfa::SinglePatternDfa;

/// Module that provides a type for a multi-pattern DFA
/// that can be used to match multiple patterns in parallel.
mod multi_pattern_dfa;
pub use multi_pattern_dfa::MultiPatternDfa;

/// Module that provides functions and types related to character classes.
mod character_class;

/// Module that provides function type that can be used to decide if a character is in a character class.
mod match_function;

/// Module that provides types related to DFA
mod dfa;

/// Module that provides types related to compiled DFAs
mod compiled_dfa;

/// Module that provides code formatting
mod rust_code_formatter;
pub use rust_code_formatter::try_format;

/// Runtime module
mod runtime;
pub use runtime::{Dfa, DfaData, FindMatches, Regex};

// Reexport the `regex_automata` crate
pub use regex_automata;
