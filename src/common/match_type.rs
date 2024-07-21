use super::Span;

/// A match in the haystack.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Match {
    /// The pattern ID.
    pattern: usize,
    /// The underlying match span.
    span: Span,
}

impl Match {
    /// Create a new match.
    pub fn new(pattern: usize, span: Span) -> Self {
        Self { pattern, span }
    }

    /// Get the start of the match.
    pub fn start(&self) -> usize {
        self.span.start
    }

    /// Get the end of the match.
    pub fn end(&self) -> usize {
        self.span.end
    }

    /// Get the span of the match.
    pub fn span(&self) -> Span {
        self.span
    }

    /// Get the pattern ID.
    pub fn pattern(&self) -> usize {
        self.pattern
    }
}
