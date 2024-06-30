//! This module contains the source generator for the regex syntax.
//! The source generator is used to generate code from the regex syntax.

use crate::Result;
use log::trace;
use std::time::Instant;

use regex_syntax::ast::Ast;

/// Generate code from the regex syntax.
/// The function returns an error if the regex syntax is invalid.
/// # Arguments
/// * `ast` - An `Ast` that represents the abstract syntax tree (AST) of the regex syntax.
/// * a Write trait object that holds the generated code.
/// # Returns
/// A `Result` of type `()` that represents the success.
/// # Errors
/// An error is returned if the regex contains unsupported syntax.
///
/// # Example
pub fn generate_code(_ast: &Ast, _output: &mut dyn std::io::Write) -> Result<()> {
    let now = Instant::now();
    // Add code generation logic here
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
    use crate::parser::parse_regex_syntax;

    #[test]
    fn test_generate_code() {
        // Create an example AST
        let ast = parse_regex_syntax("").unwrap();

        // Create a buffer to hold the generated code
        let mut buffer = Vec::new();

        // Generate the code
        let result = generate_code(&ast, &mut buffer);

        // Assert that the code generation was successful
        assert!(result.is_ok());

        // Assert that the generated code is correct
        let generated_code = String::from_utf8(buffer).unwrap();
        assert_eq!(generated_code, "");

        // Add additional assertions if needed
    }
}
