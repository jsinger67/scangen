/// An index into a slice or vector.
/// This type is used for other types like StateId and TerminalId.
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

/// The StateId is a unique identifier for a state in the NFA/DFA.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct StateId(Index);

impl StateId {
    /// Create a new state id.
    #[inline]
    pub(crate) fn new(index: usize) -> Self {
        StateId(Index::new(index))
    }

    /// Get the state id as usize.
    #[inline]
    pub fn as_usize(&self) -> usize {
        self.0.as_usize()
    }

    /// Get the state id as index.
    #[inline]
    pub fn as_index(&self) -> Index {
        self.0
    }
}

impl core::ops::Add<usize> for StateId {
    type Output = StateId;

    #[inline]
    fn add(self, rhs: usize) -> Self::Output {
        StateId(self.0 + rhs)
    }
}

impl core::ops::AddAssign<usize> for StateId {
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl std::fmt::Display for StateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The identifier for a terminal the scanner has matched when reaching an accepting state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TerminalId(Index);

impl TerminalId {
    /// Create a new terminal id.
    // #[inline]
    // pub(crate) fn new(index: usize) -> Self {
    //     TerminalId(Index::new(index))
    // }

    /// Get the terminal id as usize.
    #[inline]
    pub fn as_usize(&self) -> usize {
        self.0.as_usize()
    }

    /// Get the terminal id as index.
    #[inline]
    pub fn as_index(&self) -> Index {
        self.0
    }
}

impl core::ops::Add<usize> for TerminalId {
    type Output = TerminalId;

    #[inline]
    fn add(self, rhs: usize) -> Self::Output {
        TerminalId(self.0 + rhs)
    }
}

impl core::ops::AddAssign<usize> for TerminalId {
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl std::fmt::Display for TerminalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
