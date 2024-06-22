#![forbid(missing_docs)]
//! The `scangen` crate provides a library for generating code from a regex syntax.
//! The crate should fill a gap in the regex ecosystem by providing a way to generate code from a
//! regex syntax.

/// The parser module contains the regex syntax parser.
mod parser;

/// The generator module contains the code generator.
/// The code generator generates code from the regex syntax.
mod generator;

/// The nfa module contains the NFA implementation.
mod nfa;

/// The module containing the conversions from Ast to Nfa
mod ast;

/// Module with conversion to graphviz dot format
mod dot;
