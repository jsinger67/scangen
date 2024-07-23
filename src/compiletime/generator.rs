//! This module contains the source generator for the regex syntax.
//! The source generator is used to generate code from the regex syntax.

use crate::{compiletime::MultiPatternDfa, Result};
use log::trace;
use std::time::Instant;

/// Generate code from the regex syntax.
/// The function returns an error if the regex syntax is invalid.
/// # Arguments
/// * `pattern` - A slice of string slices that holds the regex syntax pattern.
/// # Returns
/// A `Result` of type `()` that represents the success.
/// # Errors
/// An error is returned if the regex contains unsupported syntax.
///
/// # Example
pub fn generate_code(pattern: &[&str], output: &mut dyn std::io::Write) -> Result<()> {
    let now = Instant::now();

    let mut multi_pattern_dfa = MultiPatternDfa::new();
    multi_pattern_dfa.add_patterns(pattern)?;

    multi_pattern_dfa.generate_code(output)?;

    let elapsed_time = now.elapsed();
    trace!(
        "Code generation took {} milliseconds.",
        elapsed_time.as_millis()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiletime::rust_code_formatter::try_format;
    use regex::Regex;
    use std::fs;

    // Pattern taken from parol
    const TERMINALS: &[&str] = &[
        /* 0 */ "\\r\\n|\\r|\\n",
        /* 1 */ "[\\s--\\r\\n]+",
        /* 2 */ "(//.*(\\r\\n|\\r|\\n))",
        /* 3 */ "(/\\*.*?\\*/)",
        /* 4 */ "%start",
        /* 5 */ "%title",
        /* 6 */ "%comment",
        /* 7 */ "%user_type",
        /* 8 */ "=",
        /* 9 */ "%grammar_type",
        /* 10 */ "%line_comment",
        /* 11 */ "%block_comment",
        /* 12 */ "%auto_newline_off",
        /* 13 */ "%auto_ws_off",
        /* 14 */ "%on",
        /* 15 */ "%enter",
        /* 16 */ "%%",
        /* 17 */ "::",
        /* 18 */ ":",
        /* 19 */ ";",
        /* 20 */ "\\|",
        /* 21 */ "<",
        /* 22 */ ">",
        /* 23 */ "\"(\\\\.|[^\\\\])*?\"",
        /* 24 */ "'(\\\\'|[^'])*?'",
        /* 25 */ "\\u{2F}(\\\\.|[^\\\\])*?\\u{2F}",
        /* 26 */ "\\(",
        /* 27 */ "\\)",
        /* 28 */ "\\[",
        /* 29 */ "\\]",
        /* 30 */ "\\{",
        /* 31 */ "\\}",
        /* 32 */ "[a-zA-Z_][a-zA-Z0-9_]*",
        /* 33 */ "%scanner",
        /* 34 */ ",",
        /* 35 */ "%sc",
        /* 36 */ "%push",
        /* 37 */ "%pop",
        /* 38 */ "\\^",
        /* 39 */ ".",
    ];

    #[test]
    fn test_generate_code() {
        {
            // Create a buffer to hold the generated code
            let mut out_file = fs::File::create("data/test_generate_code.rs").unwrap();
            // Generate the code
            let result = generate_code(TERMINALS, &mut out_file);
            // Assert that the code generation was successful
            assert!(result.is_ok());
        }

        // Format the generated code
        try_format("data/test_generate_code.rs").unwrap();

        // Assert that the generated code is correct
        let generated_code = fs::read_to_string("data/test_generate_code.rs").unwrap();
        let expected_generated_code =
            fs::read_to_string("data/expected/test_generate_code.rs").unwrap();
        let rx_newline: Regex = Regex::new(r"\r?\n|\r").unwrap();
        // We replace all newlines with '\n' to make the comparison platform independent
        assert_eq!(
            rx_newline.replace_all(&expected_generated_code, "\n"),
            rx_newline.replace_all(&generated_code, "\n"),
            "generation result mismatch!"
        );
    }
}
