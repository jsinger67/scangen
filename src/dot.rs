//! The `dot` module contains the conversion from an NFA to a graphviz dot format.

use std::io::Write;

use dot_writer::{Attributes, DotWriter, RankDirection};

use crate::nfa::Nfa;

pub(crate) fn render_to<W: Write>(nfa: &Nfa, label: &str, output: &mut W) {
    let mut writer = DotWriter::from(output);
    writer.set_pretty_print(true);
    let mut digraph = writer.digraph();
    digraph
        .set_label(label)
        .set_rank_direction(RankDirection::LeftRight);
    for state in nfa.states() {
        let source_id = {
            let mut source_node = digraph.node_auto();
            source_node.set_label(&state.state().to_string());
            if state.state() == nfa.start_state() {
                source_node
                    .set_shape(dot_writer::Shape::Circle)
                    .set_color(dot_writer::Color::Blue)
                    .set_pen_width(3.0);
            }
            if state.state() == nfa.end_state() {
                source_node
                    .set_shape(dot_writer::Shape::Circle)
                    .set_color(dot_writer::Color::Red)
                    .set_pen_width(3.0);
            }
            source_node.id()
        };
        for transition in state.transitions() {
            let target_state = transition.target_state();
            digraph
                .edge(source_id.clone(), &format!("node_{}", target_state))
                .attributes()
                .set_label(&format!("{}", transition.chars()));
        }
        for epsilon_transition in state.epsilon_transitions() {
            let target_state = epsilon_transition.target_state();
            digraph
                .edge(source_id.clone(), &format!("node_{}", target_state))
                .attributes()
                .set_label("ε");
        }
    }
}

// use std::{borrow::Cow, io::Write};

// use crate::nfa::{EpsilonTransition, Nfa, NfaState, NfaTransition, StateId};

// #[derive(Debug, Clone)]
// enum DotEdge {
//     EpsilonTransition(StateId, EpsilonTransition),
//     NfaTransition(StateId, NfaTransition),
// }

// type Nd = NfaState;
// type Ed = DotEdge;

// impl<'a> dot::Labeller<'a, Nd, Ed> for Nfa {
//     fn graph_id(&'a self) -> dot::Id<'a> {
//         dot::Id::new("NFA").unwrap()
//     }

//     fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
//         dot::Id::new(format!("N{}", n.state())).unwrap()
//     }

//     fn node_label(&'a self, n: &Nd) -> dot::LabelText<'a> {
//         dot::LabelText::LabelStr(format!("{}", n.state()).into())
//     }

//     fn edge_label(&'a self, e: &Ed) -> dot::LabelText<'a> {
//         match e {
//             Ed::EpsilonTransition(_, _) => dot::LabelText::LabelStr("".into()),
//             Ed::NfaTransition(_, nfa_transition) => {
//                 dot::LabelText::LabelStr(format!("{}", nfa_transition.chars()).into())
//             }
//         }
//     }
// }

// impl<'a> dot::GraphWalk<'a, Nd, Ed> for Nfa {
//     fn nodes(&self) -> dot::Nodes<'a, Nd> {
//         Cow::Owned(self.states().to_vec())
//     }
//     fn edges(&'a self) -> dot::Edges<'a, Ed> {
//         let mut edges = Vec::new();
//         for state in self.states() {
//             for transition in state.transitions() {
//                 edges.push(DotEdge::NfaTransition(state.state(), transition.clone()));
//             }
//             for epsilon_transition in state.epsilon_transitions() {
//                 edges.push(DotEdge::EpsilonTransition(
//                     state.state(),
//                     epsilon_transition.clone(),
//                 ));
//             }
//         }
//         Cow::Owned(edges)
//     }
//     fn source(&self, e: &Ed) -> Nd {
//         match e {
//             Ed::EpsilonTransition(state, _) => NfaState::new(*state),
//             Ed::NfaTransition(state, _) => NfaState::new(*state),
//         }
//     }
//     fn target(&self, e: &Ed) -> Nd {
//         match e {
//             Ed::EpsilonTransition(_, epsilon_transition) => {
//                 NfaState::new(epsilon_transition.target_state())
//             }
//             Ed::NfaTransition(_, nfa_transition) => NfaState::new(nfa_transition.target_state()),
//         }
//     }
// }

// pub(crate) fn render_to<W: Write>(nfa: &Nfa, output: &mut W) {
//     dot::render(nfa, output).unwrap()
// }
