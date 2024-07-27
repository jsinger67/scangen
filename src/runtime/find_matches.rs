use crate::common::Match;

use super::Scanner;

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
            self.advance_beyound_match(matched);
        } else {
            // Repeatedly advance the char_indices iterator by one character and try again until
            // a match is found or the iterator is exhausted.
            while self.char_indices.next().is_some() {
                result = self
                    .scanner
                    .find_from(self.char_indices.clone(), self.matches_char_class);
                if let Some(matched) = result {
                    self.advance_beyound_match(matched);
                    break;
                }
            }
        }
        result
    }

    // Advance the char_indices iterator to the end of the match.
    #[inline]
    fn advance_beyound_match(&mut self, matched: Match) {
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
