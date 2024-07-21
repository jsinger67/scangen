//! This module contails a TryFrom implementation for converting the AST to an NFA.

use regex_syntax::ast::{Ast, RepetitionKind, RepetitionRange};

use crate::compiletime::{nfa::Nfa, Result, ScanGenError};

macro_rules! unsupported {
    ($feature:expr) => {
        ScanGenError::new($crate::ScanGenErrorKind::UnsupportedFeature(
            $feature.to_string(),
        ))
    };
}

impl TryFrom<Ast> for Nfa {
    type Error = ScanGenError;

    fn try_from(ast: Ast) -> Result<Self> {
        let mut nfa = Nfa::new();
        match ast {
            Ast::Empty(_) => Ok(nfa),
            Ast::Flags(_) => Err(unsupported!(format!("{:?}", ast))),
            Ast::Literal(ref l) => {
                let start_state = nfa.end_state();
                let end_state = nfa.new_state();
                nfa.set_end_state(end_state);
                nfa.add_transition(start_state, Ast::Literal(l.clone()), end_state);
                Ok(nfa)
            }
            Ast::Dot(ref d) => {
                let start_state = nfa.end_state();
                let end_state = nfa.new_state();
                nfa.set_end_state(end_state);
                nfa.add_transition(start_state, Ast::Dot(d.clone()), end_state);
                Ok(nfa)
            }
            Ast::Assertion(ref a) => Err(unsupported!(format!("Assertion {:?}", a.kind))),
            Ast::ClassUnicode(_) | Ast::ClassPerl(_) | Ast::ClassBracketed(_) => {
                let start_state = nfa.end_state();
                let end_state = nfa.new_state();
                nfa.set_end_state(end_state);
                nfa.add_transition(start_state, ast.clone(), end_state);
                Ok(nfa)
            }
            Ast::Repetition(ref r) => {
                let mut nfa2: Nfa = r.ast.as_ref().clone().try_into()?;
                match &r.op.kind {
                    RepetitionKind::ZeroOrOne => {
                        nfa2.zero_or_one();
                        nfa = nfa2;
                    }
                    RepetitionKind::ZeroOrMore => {
                        nfa2.zero_or_more();
                        nfa = nfa2;
                    }
                    RepetitionKind::OneOrMore => {
                        nfa2.one_or_more();
                        nfa = nfa2;
                    }
                    RepetitionKind::Range(r) => match r {
                        RepetitionRange::Exactly(c) => {
                            for _ in 0..*c {
                                nfa.concat(nfa2.clone());
                            }
                        }
                        RepetitionRange::AtLeast(c) => {
                            for _ in 0..*c {
                                nfa.concat(nfa2.clone());
                            }
                            let mut nfa_zero_or_more: Nfa = nfa2.clone();
                            nfa_zero_or_more.zero_or_more();
                            nfa.concat(nfa_zero_or_more);
                        }
                        RepetitionRange::Bounded(least, most) => {
                            for _ in 0..*least {
                                nfa.concat(nfa2.clone());
                            }
                            let mut nfa_zero_or_one: Nfa = nfa2.clone();
                            nfa_zero_or_one.zero_or_one();
                            for _ in *least..*most {
                                nfa.concat(nfa_zero_or_one.clone());
                            }
                        }
                    },
                }
                Ok(nfa)
            }
            Ast::Group(ref g) => {
                nfa = g.ast.as_ref().clone().try_into()?;
                Ok(nfa)
            }
            Ast::Alternation(ref a) => {
                for ast in a.asts.iter() {
                    let nfa2: Nfa = ast.clone().try_into()?;
                    nfa.alternation(nfa2);
                }
                Ok(nfa)
            }
            Ast::Concat(ref c) => {
                for ast in c.asts.iter() {
                    let nfa2: Nfa = ast.clone().try_into()?;
                    nfa.concat(nfa2);
                }
                Ok(nfa)
            }
        }
    }
}
