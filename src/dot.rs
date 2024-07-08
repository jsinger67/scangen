//! The `dot` module contains the conversion from an NFA to a graphviz dot format.

use std::io::Write;

use dot_writer::{Attributes, DotWriter, RankDirection};
use regex_automata::util::primitives::StateID;

use crate::{dfa::Dfa, multi_pattern_nfa::MultiPatternNfa, nfa::Nfa};

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
            source_node.set_label(&state.id().as_usize().to_string());
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
                .edge(
                    source_id.clone(),
                    &format!("node_{}", target_state.as_usize()),
                )
                .attributes()
                .set_label(&format!("{}", transition.chars()));
        }
        for epsilon_transition in state.epsilon_transitions() {
            let target_state = epsilon_transition.target_state();
            digraph
                .edge(
                    source_id.clone(),
                    &format!("node_{}", target_state.as_usize()),
                )
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
            source_node.set_label(&state.id().as_usize().to_string());
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
                        "{}\n'{}':{}",
                        state.id().as_usize(),
                        nfa.patterns()[pattern_id.as_usize()].escape_default(),
                        pattern_id.as_usize(),
                    ));
            }
            source_node.id()
        };
        for transition in state.transitions() {
            let target_state = transition.target_state();
            digraph
                .edge(
                    source_id.clone(),
                    &format!("node_{}", target_state.as_usize()),
                )
                .attributes()
                .set_label(&format!(
                    "{}:{}",
                    nfa.char_classes()[transition.chars()]
                        .ast
                        .0
                        .to_string()
                        .escape_default(),
                    transition.chars()
                ));
        }
        for epsilon_transition in state.epsilon_transitions() {
            let target_state = epsilon_transition.target_state();
            digraph
                .edge(
                    source_id.clone(),
                    &format!("node_{}", target_state.as_usize()),
                )
                .attributes()
                .set_label("ε");
        }
    }
}

/// Render a DFA to a graphviz dot format.
pub fn dfa_render_to<W: Write>(dfa: &Dfa, label: &str, output: &mut W) {
    let mut writer = DotWriter::from(output);
    writer.set_pretty_print(true);
    let mut digraph = writer.digraph();
    digraph
        .set_label(label)
        .set_rank_direction(RankDirection::LeftRight);
    // Render the states of the DFA
    for state_id in 0..dfa.states().len() {
        let mut source_node = digraph.node_auto();
        source_node.set_label(&state_id.to_string());
        if state_id == 0 {
            source_node
                .set_shape(dot_writer::Shape::Circle)
                .set_color(dot_writer::Color::Blue)
                .set_pen_width(3.0);
        }
        if let Some(pattern_id) = dfa
            .accepting_states()
            .get(&StateID::new_unchecked(state_id))
        {
            source_node
                .set_color(dot_writer::Color::Red)
                .set_pen_width(3.0)
                .set_label(&format!(
                    "{}\n'{}':{}",
                    state_id,
                    dfa.patterns()[pattern_id.as_usize()].escape_default(),
                    pattern_id.as_usize(),
                ));
        }
    }
    // Render the transitions of the DFA
    for (source_id, targets) in dfa.transitions() {
        for (char_id, target_id) in targets.iter() {
            // Label the edge with the character class used to transition to the target state.
            digraph
                .edge(
                    &format!("node_{}", source_id.as_usize()),
                    &format!("node_{}", target_id.as_usize()),
                )
                .attributes()
                .set_label(&format!(
                    "{}:{}",
                    dfa.char_classes()[char_id.id()]
                        .ast
                        .0
                        .to_string()
                        .escape_default(),
                    char_id.id()
                ));
        }
    }
}
