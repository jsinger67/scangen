use regex_syntax::ast::{Ast, Literal, Position, Span};

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

/// A comparator for the AST of a character class. It only compares AST types that are relevant for
/// handling of character classes.
pub(crate) struct ComparableAst(pub(crate) Ast);

impl PartialEq for ComparableAst {
    fn eq(&self, other: &Self) -> bool {
        match &self.0 {
            Ast::Empty(_) => matches!(other.0, Ast::Empty(_)),
            Ast::Literal(ll) => {
                let Literal { kind, c, .. } = &**ll;
                match &other.0 {
                    Ast::Literal(lr) => {
                        let Literal {
                            kind: kr, c: cr, ..
                        } = &**lr;
                        kind == kr && c == cr
                    }
                    _ => false,
                }
            }
            Ast::ClassUnicode(cl) => {
                let regex_syntax::ast::ClassUnicode { kind, negated, .. } = &**cl;
                match &other.0 {
                    Ast::ClassUnicode(cr) => {
                        let regex_syntax::ast::ClassUnicode {
                            kind: kr,
                            negated: nr,
                            ..
                        } = &**cr;
                        kind == kr && negated == nr
                    }
                    _ => false,
                }
            }
            Ast::ClassPerl(cl) => {
                let regex_syntax::ast::ClassPerl { kind, negated, .. } = &**cl;
                match &other.0 {
                    Ast::ClassPerl(cr) => {
                        let regex_syntax::ast::ClassPerl {
                            kind: kr,
                            negated: nr,
                            ..
                        } = &**cr;
                        kind == kr && negated == nr
                    }
                    _ => false,
                }
            }
            Ast::ClassBracketed(cl) => {
                let regex_syntax::ast::ClassBracketed { kind, negated, .. } = &**cl;
                match &other.0 {
                    Ast::ClassBracketed(cr) => {
                        let regex_syntax::ast::ClassBracketed {
                            kind: kr,
                            negated: nr,
                            ..
                        } = &**cr;
                        kind == kr && negated == nr
                    }
                    _ => false,
                }
            }
            Ast::Dot(_) => matches!(other.0, Ast::Dot(_)),

            Ast::Flags(_)
            | Ast::Assertion(_)
            | Ast::Repetition(_)
            | Ast::Group(_)
            | Ast::Alternation(_)
            | Ast::Concat(_) => false,
        }
    }
}
