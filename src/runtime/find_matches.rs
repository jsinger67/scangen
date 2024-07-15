use regex_automata::Match;

use super::Regex;

/// An iterator over all non-overlapping matches.
///
/// The iterator yields a [`Match`] value until no more matches could be found.
///
/// The lifetime parameters are as follows:
///
/// * `'r` represents the lifetime of the `Regex` that produced this iterator.
/// * `'h` represents the lifetime of the haystack being searched.
///
/// This iterator can be created with the [`Regex::find_iter`] method.
#[derive(Debug)]
pub struct FindMatches<'r, 'h> {
    regex: &'r mut Regex,
    char_indices: std::str::CharIndices<'h>,
    matches_char_class: fn(char, usize) -> bool,
}

impl<'r, 'h> FindMatches<'r, 'h> {
    /// Creates a new `FindMatches` iterator.
    pub fn new(
        regex: &'r mut Regex,
        input: &'h str,
        matches_char_class: fn(char, usize) -> bool,
    ) -> Self {
        FindMatches {
            regex,
            char_indices: input.char_indices(),
            matches_char_class,
        }
    }

    /// Returns the next match in the haystack.
    ///
    /// If no match is found, `None` is returned.
    #[inline]
    pub fn next_match(&mut self) -> Option<Match> {
        let result = self
            .regex
            .find_from(self.char_indices.clone(), self.matches_char_class);
        if let Some(matched) = result {
            // Advance the char_indices iterator to the end of the match.
            let end = matched.span().end - 1;
            let mut peekable = self.char_indices.by_ref().peekable();
            while peekable.next_if(|(i, _)| *i < end).is_some() {}
        }
        result
    }
}

impl Iterator for FindMatches<'_, '_> {
    type Item = Match;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_match()
    }
}
