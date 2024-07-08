/// An index into a slice or vector.
/// This type is used for other types like StateID and TerminalId.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Index(usize);

impl Index {
    /// Create a new index.
    #[inline]
    pub(crate) fn new(index: usize) -> Self {
        Index(index)
    }

    /// Get the index as usize.
    #[inline]
    pub fn as_usize(&self) -> usize {
        self.0
    }
}

impl<T> core::ops::Index<Index> for [T] {
    type Output = T;

    #[inline]
    fn index(&self, index: Index) -> &Self::Output {
        &self[index.as_usize()]
    }
}

impl<T> core::ops::IndexMut<Index> for [T] {
    #[inline]
    fn index_mut(&mut self, index: Index) -> &mut Self::Output {
        &mut self[index.as_usize()]
    }
}

impl<T> core::ops::Index<Index> for Vec<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: Index) -> &Self::Output {
        &self[index.as_usize()]
    }
}

impl<T> core::ops::IndexMut<Index> for Vec<T> {
    #[inline]
    fn index_mut(&mut self, index: Index) -> &mut Self::Output {
        &mut self[index.as_usize()]
    }
}

impl core::ops::Add<usize> for Index {
    type Output = Index;

    #[inline]
    fn add(self, rhs: usize) -> Self::Output {
        Index(self.0 + rhs)
    }
}

impl core::ops::AddAssign<usize> for Index {
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl std::fmt::Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The identifier for a character class in the NFA/DFA.
/// This is used to identify the character class in the transition table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CharClassId(Index);

impl CharClassId {
    /// Create a new character class id.
    #[inline]
    pub(crate) fn new(index: usize) -> Self {
        CharClassId(Index::new(index))
    }

    /// Get the character class id as usize.
    #[inline]
    pub fn as_usize(&self) -> usize {
        self.0.as_usize()
    }

    /// Get the character class id as index.
    #[inline]
    pub fn as_index(&self) -> Index {
        self.0
    }
}

impl core::ops::Add<usize> for CharClassId {
    type Output = CharClassId;

    #[inline]
    fn add(self, rhs: usize) -> Self::Output {
        CharClassId(self.0 + rhs)
    }
}

impl core::ops::AddAssign<usize> for CharClassId {
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl std::fmt::Display for CharClassId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<usize> for CharClassId {
    fn from(index: usize) -> Self {
        CharClassId::new(index)
    }
}
