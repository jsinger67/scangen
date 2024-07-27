#![forbid(missing_docs)]
//! # scangen
//! The `scangen` crate provides a library for generating code from multiple regexes, i.e. tokens
//! that can be used to scan text for matches.
//! The crate should fill a gap in the regex ecosystem by providing a way to generate code from a
//! regex syntax and thus having a compile ahead of time (AOT) solution for regexes.
//! # Crate features
//! The crate has two features:
//! - `generate`: This feature enables the compiletime module which can be used to generate code
//!   from a regex syntax.
//! - `runtime`: This feature enables the runtime module which can be used to scan text for matches.
//!
//! To use only the runtime feature, use the following in your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! scangen = { version = "0.1", default-features = false, features = ["runtime"] }
//! ```
//!
//! Otherwise, to only use the compiletime feature, use the following:
//! ```toml
//! [dependencies]
//! scangen = { version = "0.1", default-features = false, features = ["generate"] }
//! ```
//!
//! # Example
//! The following example shows how to generate code from a set of regexes and format the generated
//! code.
//!
//! ```rust
//! use scangen::{generate_code, try_format};
//! use std::fs;
//!
//! const TERMINALS: &[&str] = &[
//!     /* 0 */ "\\r\\n|\\r|\\n",   // Newline
//!     /* 1 */ "[\\s--\\r\\n]+",   // Whitespace
//!     /* 2 */ "(//.*(\\r\\n|\\r|\\n))",   // Line comment
//!     /* 3 */ "(/\\*.*?\\*/)",    // Block comment
//!     /* 4 */ r",",   // Comma
//!     /* 5 */ r"0|[1-9][0-9]*",   // Number
//!     /* 6 */ ".",    // Any character, i.e. error
//! ];
//!
//! let file_name = "data/scanner.rs";
//! {
//!     // Create a buffer to hold the generated code
//!     let mut out_file = fs::File::create(file_name.clone()).expect("Failed to create file");
//!     // Generate the code
//!     let result = generate_code(TERMINALS, &[], &mut out_file);
//!     // Assert that the code generation was successful
//!     assert!(result.is_ok());
//! }
//!
//! // Format the generated code
//! try_format(file_name).expect("Failed to format the generated code");
//! ```

/// Module with common types and functions
mod common;
pub use common::{DfaData, Match, ScannerModeData, Span};

/// Compiletime module
#[cfg(feature = "generate")]
mod compiletime;
#[cfg(feature = "generate")]
pub use compiletime::{generate_code, try_format, Result, ScanGenError, ScanGenErrorKind};

/// Runtime module
#[cfg(feature = "runtime")]
mod runtime;
#[cfg(feature = "runtime")]
pub use runtime::{
    Dfa, FindMatches, Scanner, ScannerBuilder, ScannerBuilderWithScannerModes,
    ScannerBuilderWithsDfas, ScannerBuilderWithsDfasAndScannerModes, ScannerMode,
};
