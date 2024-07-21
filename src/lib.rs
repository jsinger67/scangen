#![forbid(missing_docs)]
//! The `scangen` crate provides a library for generating code from a regex syntax.
//! The crate should fill a gap in the regex ecosystem by providing a way to generate code from a
//! regex syntax.

/// Module with common types and functions
mod common;
pub use common::{Match, Span};

/// Compiletime module
mod compiletime;
pub use compiletime::{generate_code, try_format, Result, ScanGenError, ScanGenErrorKind};

/// Runtime module
mod runtime;
pub use runtime::{Dfa, DfaData, FindMatches, Regex};
