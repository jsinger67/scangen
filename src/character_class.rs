use std::ops::Index;

use regex_automata::util::primitives::SmallIndex;
use regex_syntax::ast::{Ast, Position, Span};

/// A character class that can match a character.
#[derive(Default, Clone)]
pub(crate) struct CharacterClass {
    pub(crate) id: CharClassID,
    pub(crate) ast: ComparableAst,
}

impl CharacterClass {
    pub(crate) fn new(id: CharClassID, ast: Ast) -> Self {
        CharacterClass {
            id,
            ast: ComparableAst(ast),
        }
    }

    pub(crate) fn id(&self) -> CharClassID {
        self.id
    }
}

impl std::fmt::Debug for CharacterClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CharacterClass {{ id: {:?}, ast: {:?} }}",
            self.id, self.ast
        )
    }
}

impl std::hash::Hash for CharacterClass {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.ast.hash(state);
        // Do not hash the match function, because it is not relevant for equality.
        // Actually it is calculated from the AST, so it would be redundant.
    }
}

impl PartialEq for CharacterClass {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.ast == other.ast
    }
}

impl Eq for CharacterClass {}

impl PartialOrd for CharacterClass {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl Ord for CharacterClass {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

/// A comparable AST in regard of a character class.
/// It only compares AST types that are relevant for handling of character classes.
#[derive(Debug, Clone, Eq)]
pub(crate) struct ComparableAst(pub(crate) Ast);

impl PartialEq for ComparableAst {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Ast::Empty(_), Ast::Empty(_))
            | (Ast::Dot(_), Ast::Dot(_))
            | (Ast::Literal(_), Ast::Literal(_))
            | (Ast::ClassUnicode(_), Ast::ClassUnicode(_))
            | (Ast::ClassPerl(_), Ast::ClassPerl(_))
            | (Ast::ClassBracketed(_), Ast::ClassBracketed(_)) => {
                // Compare the string representation of the ASTs.
                // This is a workaround because the AST's implementation of PartialEq also
                // compares the span, which is not relevant for the character class handling here.
                self.0.to_string() == other.0.to_string()
            }
            _ => false,
        }
    }
}

impl std::hash::Hash for ComparableAst {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash the string representation of the AST.
        self.0.to_string().hash(state);
    }
}

impl Default for ComparableAst {
    fn default() -> Self {
        ComparableAst(Ast::Empty(Box::new(Span {
            start: Position {
                offset: 0,
                line: 0,
                column: 0,
            },
            end: Position {
                offset: 0,
                line: 0,
                column: 0,
            },
        })))
    }
}

/// The identifier for a character class in the NFA/DFA.
/// This is used to identify the character class in the transition table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CharClassID(SmallIndex);

impl CharClassID {
    /// Create a new character class id.
    #[inline]
    pub(crate) fn new(index: usize) -> Self {
        CharClassID(SmallIndex::new_unchecked(index))
    }

    /// Get the character class id as usize.
    #[inline]
    pub fn as_usize(&self) -> usize {
        self.0.as_usize()
    }
}

impl core::ops::Add<usize> for CharClassID {
    type Output = CharClassID;

    #[inline]
    fn add(self, rhs: usize) -> Self::Output {
        CharClassID(SmallIndex::new_unchecked(self.0.as_usize() + rhs))
    }
}

impl core::ops::AddAssign<usize> for CharClassID {
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        self.0 = SmallIndex::new_unchecked(self.0.as_usize() + rhs);
    }
}

impl<T> Index<CharClassID> for [T] {
    type Output = T;

    #[inline]
    fn index(&self, index: CharClassID) -> &Self::Output {
        &self[index.0]
    }
}

impl std::fmt::Display for CharClassID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.as_usize())
    }
}

impl From<usize> for CharClassID {
    fn from(index: usize) -> Self {
        CharClassID::new(index)
    }
}
