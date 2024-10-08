//! This module contains the parser for the regex syntax.
//! The parser is used to parse the regex syntax into an abstract syntax tree (AST).
//! We use the `regex_syntax` crate to parse the regex syntax, although we will only support a
//! subset of the regex syntax.

use crate::Result;
use log::trace;
use std::time::Instant;

use regex_syntax::ast::{parse::Parser, Ast};

/// Parse the regex syntax into an abstract syntax tree (AST).
/// The function returns an error if the regex syntax is invalid.
/// # Arguments
/// * `input` - A string slice that holds the regex syntax.
/// # Returns
/// An `Ast` that represents the abstract syntax tree of the regex syntax.
/// # Errors
/// An error is returned if the regex syntax is invalid.
pub fn parse_regex_syntax(input: &str) -> Result<Ast> {
    let now = Instant::now();
    match Parser::new().parse(input) {
        Ok(syntax_tree) => {
            let elapsed_time = now.elapsed();
            trace!("Parsing took {} milliseconds.", elapsed_time.as_millis());
            Ok(syntax_tree)
        }
        Err(e) => Err(e.into()),
    }
}
#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{ScanGenError, ScanGenErrorKind};

    use super::*;

    #[test]
    fn test_parse_regex_syntax_valid() {
        // Valid regex syntax
        let input = r"\d";
        let ast = parse_regex_syntax(input).unwrap();
        // Add assertions here to validate the AST
        assert_eq!(format!("{:?}", ast),
            "ClassPerl(ClassPerl { span: Span(Position(o: 0, l: 1, c: 1), Position(o: 2, l: 1, c: 3)), kind: Digit, negated: false })");
    }

    #[test]
    fn test_parse_regex_syntax_invalid() {
        // Invalid regex syntax
        let input = r"^\d{4}-\d{2}-\d{2}$[";
        let result = parse_regex_syntax(input);
        assert!(result.is_err());
        // Add assertions here to validate the error message or behavior
        assert!(matches!(
            result,
            Err(ref e) if matches!(e, ScanGenError{ source } if matches!(**source, ScanGenErrorKind::RegexSyntaxError(_)))
        ));
        assert_eq!(
            result.unwrap_err().source().unwrap().to_string(),
            r#"regex parse error:
    ^\d{4}-\d{2}-\d{2}$[
                       ^
error: unclosed character class"#
        );
    }

    #[test]
    fn test_parse_regex_syntax_empty() {
        // Empty regex syntax
        let input = "";
        let result = parse_regex_syntax(input);
        assert!(result.is_ok());
    }

    // This may hinder the use of the regex_syntax crate because it does not support lookaround
    // assertions. We'll have to evaluate if we can live with this limitation.
    #[test]
    #[should_panic(expected = "RegexSyntaxError(Error { kind: UnsupportedLookAround")]
    fn test_a_only_if_followed_by_b() {
        // Scanner syntax that matches 'a' only if it is followed by 'b'
        let input = r"a(?=b)";
        let _ = parse_regex_syntax(input).unwrap();
    }
}
