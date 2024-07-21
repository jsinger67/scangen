/// Module with error definitions
mod errors;
pub use errors::{Result, ScanGenError, ScanGenErrorKind};

/// Module for sevearl ID types.
mod ids;
pub(crate) use ids::{CharClassID, PatternID, StateID};

/// The parser module contains the regex syntax parser.
mod parser;
pub(crate) use parser::parse_regex_syntax;

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

/// Module that provides a type for a multi-pattern NFA
/// that can be used to match multiple patterns in parallel.
mod multi_pattern_nfa;
pub(crate) use multi_pattern_nfa::MultiPatternNfa;

/// Module that provides a type for a multi-pattern DFA
/// that can be used to match multiple patterns in parallel.
mod multi_pattern_dfa;
pub(crate) use multi_pattern_dfa::MultiPatternDfa;

/// Module that provides functions and types related to character classes.
mod character_class;
pub(crate) use character_class::CharacterClass;

/// Module that provides function type that can be used to decide if a character is in a character class.
mod match_function;
pub(crate) use match_function::MatchFunction;

/// Module that provides types related to DFA
mod dfa;

/// Module that provides types related to compiled DFAs
mod compiled_dfa;

/// Module that provides code formatting
mod rust_code_formatter;
pub use rust_code_formatter::try_format;
