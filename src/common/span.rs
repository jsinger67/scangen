/// A span in a source file.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Span {
    /// The start offset of the span, inclusive.
    pub start: usize,
    /// The end offset of the span, exclusive.
    pub end: usize,
}
impl Span {
    /// Create a new span.
    #[inline]
    pub fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }

    /// Check if the span is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Get the length of the span.
    #[inline]
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }
}
