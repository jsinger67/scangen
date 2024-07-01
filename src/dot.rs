//! The `dot` module contains the conversion from an NFA to a graphviz dot format.

use std::io::Write;

use dot_writer::{Attributes, DotWriter, RankDirection};

use crate::{multi_pattern_nfa::MultiPatternNfa, nfa::Nfa};

/// Render the NFA to a graphviz dot format.
pub fn render_to<W: Write>(nfa: &Nfa, label: &str, output: &mut W) {
    let mut writer = DotWriter::from(output);
    writer.set_pretty_print(true);
    let mut digraph = writer.digraph();
    digraph
        .set_label(label)
        .set_rank_direction(RankDirection::LeftRight);
    for state in nfa.states() {
        let source_id = {
            let mut source_node = digraph.node_auto();
            source_node.set_label(&state.id().to_string());
            if state.id() == nfa.start_state() {
                source_node
                    .set_shape(dot_writer::Shape::Circle)
                    .set_color(dot_writer::Color::Blue)
                    .set_pen_width(3.0);
            }
            if state.id() == nfa.end_state() {
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

/// Render the multi-pattern NFA to a graphviz dot format.
pub fn multi_render_to<W: Write>(nfa: &MultiPatternNfa, label: &str, output: &mut W) {
    let mut writer = DotWriter::from(output);
    writer.set_pretty_print(true);
    let mut digraph = writer.digraph();
    digraph
        .set_label(label)
        .set_rank_direction(RankDirection::LeftRight);
    for state in nfa.nfa().states() {
        let source_id = {
            let mut source_node = digraph.node_auto();
            source_node.set_label(&state.id().to_string());
            // The start state of the multi-pattern NFA is always 0
            if state.id().as_usize() == 0 {
                source_node
                    .set_shape(dot_writer::Shape::Circle)
                    .set_color(dot_writer::Color::Blue)
                    .set_pen_width(3.0);
            }
            if let Some(pattern_id) = nfa.accepting_states().get(&state.id()) {
                source_node
                    .set_color(dot_writer::Color::Red)
                    .set_pen_width(3.0)
                    .set_label(&format!(
                        "{} '{}':{}",
                        state.id(),
                        nfa.patterns()[pattern_id.as_index()],
                        pattern_id,
                    ));
            }
            source_node.id()
        };
        for transition in state.transitions() {
            let target_state = transition.target_state();
            digraph
                .edge(source_id.clone(), &format!("node_{}", target_state))
                .attributes()
                .set_label(&format!(
                    "{}:{}",
                    nfa.char_classes()[transition.chars().as_index()].ast.0,
                    transition.chars()
                ));
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
