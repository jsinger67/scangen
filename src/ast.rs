//! This module contails several TryFrom implementations for converting the AST to an NFA.

use anyhow::{anyhow, Result};
use regex_syntax::ast::Ast;

use crate::nfa::Nfa;

impl TryFrom<Ast> for Nfa {
    // TODO: Use thiserror to create custom errors
    type Error = anyhow::Error;

    fn try_from(ast: Ast) -> Result<Self> {
        let mut nfa = Nfa::new();
        match ast {
            Ast::Empty(_) => Ok(nfa),
            Ast::Flags(_) => Err(anyhow!("Flags are not supported")),
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
            Ast::Assertion(_) => todo!(),
            Ast::ClassUnicode(_) => todo!(),
            Ast::ClassPerl(_) => todo!(),
            Ast::ClassBracketed(_) => todo!(),
            Ast::Repetition(ref r) => {
                let mut nfa2: Nfa = r.ast.as_ref().clone().try_into()?;
                match &r.op.kind {
                    regex_syntax::ast::RepetitionKind::ZeroOrOne => {
                        nfa2.zero_or_one();
                        nfa = nfa2;
                    }
                    regex_syntax::ast::RepetitionKind::ZeroOrMore => {
                        nfa2.zero_or_more();
                        nfa = nfa2;
                    }
                    regex_syntax::ast::RepetitionKind::OneOrMore => {
                        nfa2.one_or_more();
                        nfa = nfa2;
                    }
                    regex_syntax::ast::RepetitionKind::Range(r) => match r {
                        regex_syntax::ast::RepetitionRange::Exactly(c) => {
                            for _ in 0..*c {
                                nfa.concat(nfa2.clone());
                            }
                        }
                        regex_syntax::ast::RepetitionRange::AtLeast(c) => {
                            for _ in 0..*c {
                                nfa.concat(nfa2.clone());
                            }
                            let mut nfa_zero_or_more: Nfa = nfa2.clone();
                            nfa_zero_or_more.zero_or_more();
                            nfa.concat(nfa_zero_or_more);
                        }
                        regex_syntax::ast::RepetitionRange::Bounded(_, _) => todo!(),
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
