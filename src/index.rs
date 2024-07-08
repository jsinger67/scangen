use std::ops::Index;

use regex_automata::util::primitives::SmallIndex;

/// The identifier for a character class in the NFA/DFA.
/// This is used to identify the character class in the transition table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CharClassId(SmallIndex);

impl CharClassId {
    /// Create a new character class id.
    #[inline]
    pub(crate) fn new(index: usize) -> Self {
        CharClassId(SmallIndex::new_unchecked(index))
    }

    /// Get the character class id as usize.
    #[inline]
    pub fn as_usize(&self) -> usize {
        self.0.as_usize()
    }
}

impl core::ops::Add<usize> for CharClassId {
    type Output = CharClassId;

    #[inline]
    fn add(self, rhs: usize) -> Self::Output {
        CharClassId(SmallIndex::new_unchecked(self.0.as_usize() + rhs))
    }
}

impl core::ops::AddAssign<usize> for CharClassId {
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        self.0 = SmallIndex::new_unchecked(self.0.as_usize() + rhs);
    }
}

impl<T> Index<CharClassId> for [T] {
    type Output = T;

    #[inline]
    fn index(&self, index: CharClassId) -> &Self::Output {
        &self[index.0]
    }
}

impl std::fmt::Display for CharClassId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.as_usize())
    }
}

impl From<usize> for CharClassId {
    fn from(index: usize) -> Self {
        CharClassId::new(index)
    }
}
