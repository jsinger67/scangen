macro_rules! impl_id {
    ($name:ident) => {
        /// The ID type $name.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        pub(crate) struct $name(usize);

        impl $name {
            /// Create a new id.
            #[inline]
            pub(crate) const fn new(index: usize) -> Self {
                $name(index)
            }

            /// Get the id as usize.
            /// It is dead code in case of CharClassID.
            #[allow(dead_code)]
            #[inline]
            pub(crate) fn as_usize(&self) -> usize {
                self.0
            }
        }

        impl core::ops::Add<usize> for $name {
            type Output = $name;

            #[inline]
            fn add(self, rhs: usize) -> Self::Output {
                $name(self.0 + rhs)
            }
        }

        impl core::ops::AddAssign<usize> for $name {
            #[inline]
            fn add_assign(&mut self, rhs: usize) {
                self.0 = self.0 + rhs;
            }
        }

        impl<T> std::ops::Index<$name> for [T] {
            type Output = T;

            #[inline]
            fn index(&self, index: $name) -> &Self::Output {
                &self[index.0]
            }
        }

        impl<T> std::ops::IndexMut<$name> for [T] {
            #[inline]
            fn index_mut(&mut self, index: $name) -> &mut T {
                &mut self[index.0]
            }
        }

        impl<T> std::ops::Index<$name> for Vec<T> {
            type Output = T;

            #[inline]
            fn index(&self, index: $name) -> &Self::Output {
                &self[index.0]
            }
        }

        impl<T> std::ops::IndexMut<$name> for Vec<T> {
            #[inline]
            fn index_mut(&mut self, index: $name) -> &mut T {
                &mut self[index.0]
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<usize> for $name {
            fn from(index: usize) -> Self {
                $name::new(index)
            }
        }
    };
}

impl_id!(StateID);
impl_id!(CharClassID);
impl_id!(PatternID);
