use regex_syntax::ast::{Ast, Position, Span};

use crate::{match_function::MatchFunction, CharClassId};

/// A character class that can match a character.
pub(crate) struct CharacterClass {
    pub(crate) id: CharClassId,
    pub(crate) ast: ComparableAst,
    pub(crate) matches: Option<MatchFunction>,
}

impl CharacterClass {
    pub(crate) fn new(id: CharClassId, ast: Ast) -> Self {
        CharacterClass {
            id,
            ast: ComparableAst(ast),
            matches: None,
        }
    }

    pub(crate) fn id(&self) -> CharClassId {
        self.id
    }

    pub(crate) fn matches(&self, c: char) -> bool {
        self.matches.as_ref().map_or(false, |f| f.0(c))
    }
}

impl std::fmt::Debug for CharacterClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CharacterClass {{ id: {:?}, matches: {} }}",
            self.id,
            if self.matches.is_some() {
                "Some"
            } else {
                "None"
            }
        )
    }
}

impl Default for CharacterClass {
    fn default() -> Self {
        CharacterClass {
            id: CharClassId::default(),
            ast: ComparableAst(Ast::Empty(Box::new(Span {
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
            }))),
            matches: None,
        }
    }
}

/// A comparable AST in regard of a character class.
/// It only compares AST types that are relevant for handling of character classes.
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
