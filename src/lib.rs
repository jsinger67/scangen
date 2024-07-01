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
pub use dot::{multi_render_to, render_to};

/// Module with error definitions
mod errors;
pub use errors::{Result, ScanGenError};

/// Module that provides a type for integer ids that can also be used to index into slices.
mod index;
pub use index::{Index, PatternId, StateId};

/// Module that provides a type for a multi-pattern NFA
/// that can be used to match multiple patterns in parallel.
mod multi_pattern_nfa;
pub use multi_pattern_nfa::MultiPatternNfa;
