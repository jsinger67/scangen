use crate::common::Match;

use super::Scanner;

/// The result of a peek operation.
#[derive(Debug, PartialEq)]
pub enum PeekResult {
    /// The peek operation found a n matches.
    Matches(Vec<Match>),
    /// The peek operation found less than n matches because the end of the haystack was reached.
    MatchesReachedEnd(Vec<Match>),
    /// The peek operation found less than n matches because the last token type would have
    /// triggered a mode switch. The matches are returned  along with the index of the new mode that
    /// would be switched to on the last match.
    MatchesReachedModeSwitch((Vec<Match>, usize)),
    /// The peek operation found no matches.
    NotFound,
}

/// An iterator over all non-overlapping matches.
///
/// The iterator yields a [`Match`] value until no more matches could be found.
///
/// The lifetime parameters are as follows:
///
/// * `'r` represents the lifetime of the `Scanner` that produced this iterator.
/// * `'h` represents the lifetime of the haystack being searched.
///
/// This iterator can be created with the [`Scanner::find_iter`] method.
#[derive(Debug)]
pub struct FindMatches<'r, 'h> {
    scanner: &'r mut Scanner,
    char_indices: std::str::CharIndices<'h>,
    matches_char_class: fn(char, usize) -> bool,
}

impl<'r, 'h> FindMatches<'r, 'h> {
    /// Creates a new `FindMatches` iterator.
    pub fn new(
        scanner: &'r mut Scanner,
        input: &'h str,
        matches_char_class: fn(char, usize) -> bool,
    ) -> Self {
        FindMatches {
            scanner,
            char_indices: input.char_indices(),
            matches_char_class,
        }
    }

    /// Returns the next match in the haystack.
    ///
    /// If no match is found, `None` is returned.
    #[inline]
    pub fn next_match(&mut self) -> Option<Match> {
        let mut result = self
            .scanner
            .find_from(self.char_indices.clone(), self.matches_char_class);
        if let Some(matched) = result {
            self.advance_beyond_match(matched);
        } else {
            // Repeatedly advance the char_indices iterator by one character and try again until
            // a match is found or the iterator is exhausted.
            while self.char_indices.next().is_some() {
                result = self
                    .scanner
                    .find_from(self.char_indices.clone(), self.matches_char_class);
                if let Some(matched) = result {
                    self.advance_beyond_match(matched);
                    break;
                }
            }
        }
        result
    }

    /// Peeks n matches ahead without consuming the matches.
    /// The function returns [PeekResult].
    ///
    /// The peek operation always stops at the end of the haystack or when a mode switch is
    /// triggered by the last match. The mode switch is not conducted by the peek operation to not
    /// change the state of the scanner as well as to aviod a mix of tokens from different modes
    /// being returned.
    pub fn peek_n(&mut self, n: usize) -> PeekResult {
        let mut char_indices = self.char_indices.clone();
        let mut matches = Vec::with_capacity(n);
        let mut mode_switch = false;
        let mut new_mode = 0;
        for _ in 0..n {
            let result = self
                .scanner
                .peek_from(&mut char_indices, self.matches_char_class);
            if let Some(matched) = result {
                matches.push(matched);
                if let Some(mode) = self.scanner.has_transition(matched.token_type()) {
                    mode_switch = true;
                    new_mode = mode;
                    break;
                }
            } else {
                break;
            }
        }
        if matches.len() == n {
            PeekResult::Matches(matches)
        } else if mode_switch {
            PeekResult::MatchesReachedModeSwitch((matches, new_mode))
        } else if matches.is_empty() {
            PeekResult::NotFound
        } else {
            PeekResult::MatchesReachedEnd(matches)
        }
    }

    // Advance the char_indices iterator to the end of the match.
    #[inline]
    fn advance_beyond_match(&mut self, matched: Match) {
        let end = matched.span().end - 1;
        let mut peekable = self.char_indices.by_ref().peekable();
        while peekable.next_if(|(i, _)| *i < end).is_some() {}
    }
}

impl Iterator for FindMatches<'_, '_> {
    type Item = Match;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_match()
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        common::ScannerModeData, generate_code, runtime::generated::scanner_with_modes, try_format,
        Match, PeekResult,
    };
    use std::fs;

    const TERMINALS: &[&str] = &[
        /* 0 */ r"\r\n|\r|\n", // Newline
        /* 1 */ r"[\s--\r\n]+", // Whitespace
        /* 2 */ r"(//.*(\\r\\n|\\r|\\n))", // Line comment
        /* 3 */ r"(/\*[.\r\n]*?\*/)", // Block comment
        /* 4 */ r"[a-zA-Z_]\w*", // Identifier
        /* 5 */ r"\u{5c}[\u{22}\u{5c}bfnt]", // Escape sequence
        /* 6 */ r"\u{5c}[\s^\n\r]*\r?\n", // Line continuation
        /* 7 */ r"[^\u{22}\u{5c}]+", // String content
        /* 8 */ r"\u{22}", // String delimiter
        /* 9 */ r".", // Error
    ];

    const MODES: &[ScannerModeData] = &[
        (
            // Mode name
            "INITIAL",
            // Tokens that are valid in this mode
            &[
                (0, 0), // Newline
                (1, 1), // Whitespace
                (2, 2), // Line comment
                (3, 3), // Block comment
                (4, 4), // Identifier
                (8, 8), // String delimiter
                (9, 9), // Error
            ],
            // Transitions to other modes
            &[
                (8, 1), // Token "String delimiter" -> Mode "STRING"
            ],
        ),
        (
            // Mode name
            "STRING",
            // Tokens that are valid in this mode
            &[
                (0, 0), // Newline
                (1, 1), // Whitespace
                (2, 2), // Line comment
                (3, 3), // Block comment
                (5, 5), // Escape sequence
                (6, 6), // Line continuation
                (7, 7), // String content
                (8, 8), // String delimiter
                (9, 9), // Error
            ],
            // Transitions to other modes
            &[
                (8, 0), // Token "String delimiter" -> Mode "INITIAL"
            ],
        ),
    ];

    // The input string contains a string with escape sequences and line continuations.
    const INPUT: &str = r#"
Id1
"1. String"
Id2
"#;

    #[test]
    fn generate_code_for_scanner_with_modes() {
        // We bootstrap the scanner with the modes and terminals and use the generated code
        // for the tests later on.
        let file_name = "src/runtime/generated/scanner_with_modes.rs";
        {
            // Create the file where the generated code should be written to
            let mut out_file = fs::File::create(file_name).expect("Failed to create file");
            // Generate the code
            generate_code(TERMINALS, MODES, Some("crate"), &mut out_file)
                .expect("Failed to generate code");
        }

        // Format the generated code
        try_format(file_name).expect("Failed to format the generated code");
    }

    #[test]
    fn test_peek_n() {
        let mut scanner = scanner_with_modes::create_scanner();
        let mut find_iter = scanner_with_modes::create_find_iter(&mut scanner, INPUT);
        let peeked = find_iter.peek_n(3);
        assert_eq!(
            peeked,
            PeekResult::Matches(vec![
                Match::new(0, (0usize..1).into()),
                Match::new(4, (2usize..4).into()),
                Match::new(8, (5usize..6).into()),
            ])
        );
        let peeked = find_iter.peek_n(4);
        assert_eq!(
            peeked,
            PeekResult::MatchesReachedModeSwitch((
                vec![
                    Match::new(0, (0usize..1).into()),
                    Match::new(4, (2usize..4).into()),
                    Match::new(8, (5usize..6).into()),
                ],
                1
            ))
        );
    }

    #[test]
    fn test_find_iter() {
        let mut scanner = scanner_with_modes::create_scanner();
        let find_iter = scanner_with_modes::create_find_iter(&mut scanner, INPUT);
        let matches: Vec<Match> = find_iter.collect();
        assert_eq!(matches.len(), 9);
        assert_eq!(
            matches,
            vec![
                Match::new(0, (0usize..1).into()),
                Match::new(4, (1usize..4).into()),
                Match::new(0, (4usize..5).into()),
                Match::new(8, (5usize..6).into()),
                Match::new(7, (6usize..15).into()),
                Match::new(8, (15usize..16).into()),
                Match::new(0, (16usize..17).into()),
                Match::new(4, (17usize..20).into()),
                Match::new(0, (20usize..21).into()),
            ]
        );
        assert_eq!(
            matches
                .iter()
                .map(|m| {
                    let rng = m.span().start..m.span().end;
                    INPUT.get(rng).unwrap()
                })
                .collect::<Vec<_>>(),
            vec![
                "\n",
                "Id1",
                "\n",
                "\"",
                "1. String",
                "\"",
                "\n",
                "Id2",
                "\n"
            ]
        );
    }
}
