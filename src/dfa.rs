//! This module contains the DFA implementation.
//! The DFA is used to match a string against a regex pattern.
//! The DFA is generated from the NFA using the subset construction algorithm.

use itertools::Itertools;
use log::trace;
use std::{
    collections::{BTreeMap, BTreeSet},
    mem,
};

use crate::{character_class::CharacterClass, MultiPatternNfa, PatternId, StateId};

// The type definitions for the subset construction algorithm.
type StateGroup = BTreeSet<StateId>;
type Partition = Vec<StateGroup>;

/// The DFA implementation.
#[derive(Debug, Default)]
pub struct Dfa {
    // The states of the DFA.
    states: Vec<DfaState>,
    // The patterns for the accepting states.
    patterns: Vec<String>,
    // The accepting states of the DFA as well as the corresponding pattern id.
    accepting_states: BTreeMap<StateId, PatternId>,
    // The character classes used in the DFA.
    char_classes: Vec<CharacterClass>,
    // The transitions of the DFA.
    transitions: BTreeMap<StateId, BTreeMap<CharacterClass, StateId>>,
}

impl Dfa {
    /// Get the states of the DFA.
    pub(crate) fn states(&self) -> &[DfaState] {
        &self.states
    }

    /// Get the patterns for the accepting states.
    pub(crate) fn patterns(&self) -> &[String] {
        &self.patterns
    }

    /// Get the accepting states of the DFA.
    pub(crate) fn accepting_states(&self) -> &BTreeMap<StateId, PatternId> {
        &self.accepting_states
    }

    /// Get the character classes used in the DFA.
    pub(crate) fn char_classes(&self) -> &[CharacterClass] {
        &self.char_classes
    }

    /// Get the transitions of the DFA.
    pub(crate) fn transitions(&self) -> &BTreeMap<StateId, BTreeMap<CharacterClass, StateId>> {
        &self.transitions
    }

    /// Create a DFA from a multi-pattern NFA.
    /// The DFA is created using the subset construction algorithm.
    fn from_nfa(nfa: MultiPatternNfa) -> Self {
        let MultiPatternNfa {
            nfa,
            patterns,
            accepting_states,
            char_classes,
        } = nfa;
        let mut dfa = Dfa {
            states: Vec::new(),
            patterns,
            accepting_states: BTreeMap::new(),
            char_classes,
            transitions: BTreeMap::new(),
        };
        // The initial state of the DFA is the epsilon closure of the start state of the NFA.
        let start_state = nfa.epsilon_closure(StateId::default());
        // The initial state is the start state of the DFA.
        let initial_state = dfa.add_state_if_new(start_state, &accepting_states);
        // The work list is used to keep track of the states that need to be processed.
        let mut work_list = vec![initial_state];
        // The marked flag is used to mark a state as visited during the subset construction algorithm.
        dfa.states[initial_state.as_index()].marked = true;

        while let Some(state_id) = work_list.pop() {
            let nfa_states = dfa.states[state_id.as_index()].nfa_states.clone();
            for char_class in dfa.char_classes.clone() {
                let target_states =
                    nfa.epsilon_closure_set(nfa.move_set(&nfa_states, char_class.id()));
                if !target_states.is_empty() {
                    let target_state = dfa.add_state_if_new(target_states, &accepting_states);
                    dfa.transitions
                        .entry(state_id)
                        .or_default()
                        .insert(char_class.clone(), target_state);
                    if !dfa.states[target_state.as_index()].marked {
                        dfa.states[target_state.as_index()].marked = true;
                        work_list.push(target_state);
                    }
                }
            }
        }

        dfa
    }

    /// Add a state to the DFA if it does not already exist.
    /// The state is identified by the NFA states that constitute the DFA state.
    /// The accepting states are used to determine if the DFA state is an accepting state.
    fn add_state_if_new<I>(
        &mut self,
        nfa_states: I,
        accepting_states: &BTreeMap<StateId, PatternId>,
    ) -> StateId
    where
        I: IntoIterator<Item = StateId>,
    {
        let mut nfa_states: Vec<StateId> = nfa_states.into_iter().collect();
        nfa_states.sort_unstable();
        nfa_states.dedup();
        if let Some(state_id) = self
            .states
            .iter()
            .position(|state| state.nfa_states == nfa_states)
        {
            return StateId::new(state_id);
        }

        let state_id = self.states.len();
        let state = DfaState::new(state_id.into(), nfa_states);

        // Check if the constraint holds that only one pattern can match, i.e. the DFA
        // state only contains one accpting NFA state. This should always be the case since
        // the NFA is a multi-pattern NFA.
        debug_assert!(
            state
                .nfa_states
                .iter()
                .filter(|nfa_state_id| accepting_states.contains_key(nfa_state_id))
                .count()
                <= 1
        );

        // Check if the state contains an accepting state.
        for nfa_state_id in &state.nfa_states {
            if let Some(pattern_id) = accepting_states.get(nfa_state_id) {
                // The state is an accepting state.
                self.accepting_states
                    .insert(StateId::new(state_id), *pattern_id);
                break;
            }
        }

        trace!("Add state: {}: {:?}", state.id, state.nfa_states);

        self.states.push(state);
        StateId::new(state_id)
    }

    /// Add a representative state to the DFA.
    /// The representative state is the first state in the group.
    /// The accepting states are used to determine if the DFA state is an accepting state.
    /// The new state id is returned.
    fn add_representive_state(
        &mut self,
        group: &&BTreeSet<StateId>,
        accepting_states: &BTreeMap<StateId, PatternId>,
    ) -> StateId {
        let state_id = self.states.len();
        let state = DfaState::new(state_id.into(), Vec::new());

        // First state in group is the representative state.
        let representative_state_id = group.first().unwrap();
        if let Some(pattern_id) = accepting_states.get(representative_state_id) {
            self.accepting_states.insert(state_id.into(), *pattern_id);
        }

        trace!(
            "Add representive state {} with id {}",
            representative_state_id,
            state_id
        );

        self.states.push(state);
        StateId::new(state_id)
    }

    /// Trace out a partition of the DFA.
    #[allow(dead_code)]
    fn trace_partition(&self, context: &str, partition: &[StateGroup]) {
        trace!("Partition: {}", context);
        for (i, group) in partition.iter().enumerate() {
            trace!("Group {}: {:?}", i, group);
        }
    }

    /// Minimize the DFA.
    /// The Nfa states are removed from the DFA states during minimization. They are not needed
    /// anymore after the DFA is created.
    fn minimize(&self) -> Self {
        // The start partition is created as follows:
        // 1. The accepting states are put each in one partition with only one state.
        //    This follows from the constraint of the DFA that only one pattern can match.
        // 2. The non-accepting states are put together in one partition.
        //
        // The partitions are stored in a vector of vectors.
        //
        // The key building function for the Itertools::chunk_by method is used to create the
        // partitions. For accepting states the key is the state id, for non-accepting states
        // the key is the state id of the first non-accepting state.
        //
        // If there are no non-accepting states, the first non-accepting state is the default state.
        // This is no problem since then there will be no group with non-accepting states.

        let first_non_accepting_state = self
            .states
            .iter()
            .find(|state| !self.accepting_states.contains_key(&state.id))
            .map(|state| state.id)
            .unwrap_or(StateId::default());

        let mut partition_old = self
            .states
            .clone()
            .into_iter()
            .chunk_by(|state| {
                if self.accepting_states.contains_key(&state.id) {
                    state.id
                } else {
                    first_non_accepting_state
                }
            })
            .into_iter()
            .fold(Partition::new(), |mut partitions, (_key, group)| {
                let state_group = group.into_iter().fold(StateGroup::new(), |mut acc, state| {
                    acc.insert(state.id);
                    acc
                });
                partitions.push(state_group);
                partitions
            });

        let mut partition_new = Partition::new();
        let mut changed = true;

        while changed {
            self.trace_partition("old", &partition_old);
            partition_new = self.calculate_new_partition(&partition_old);
            self.trace_partition("new", &partition_new);
            changed = partition_new != partition_old;
            partition_old.clone_from(&partition_new);
        }

        self.create_from_partition(&partition_new)
    }

    /// Calculate the new partition based on the old partition.
    /// We try to split the groups of the partition based on the transitions of the DFA.
    /// The new partition is calculated by iterating over the old partition and the states
    /// in the groups. For each state in a group we check if the transitions to the states in the
    /// old partition's groups are the same. If the transitions are the same, the state is put in
    /// the same group as the other states with the same transitions. If the transitions are
    /// different, the state is put in a new group.
    /// The new partition is returned.
    fn calculate_new_partition(&self, partition: &[StateGroup]) -> Partition {
        let mut partitions_new = Partition::new();
        for (group_number, group) in partition.iter().enumerate() {
            // The new group receives the states from the old group which are distiguishable from
            // the other states in group.
            self.split_group(group, group_number, partition)
                .into_iter()
                .for_each(|group_new| {
                    partitions_new.push(group_new);
                });
        }
        partitions_new
    }

    fn split_group(
        &self,
        group: &StateGroup,
        group_number: usize,
        partition: &[StateGroup],
    ) -> Partition {
        // If the group contains only one state, the group is not split.
        if group.len() == 1 {
            return vec![group.clone()];
        }
        let mut group1 = StateGroup::new();
        let mut group2 = StateGroup::new();
        group
            .iter()
            .tuple_windows()
            .for_each(|(state_id1, state_id2)| {
                let state1 = &self.states[state_id1.as_index()];
                let state2 = &self.states[state_id2.as_index()];
                if let Some((target_group1, _target_group2)) =
                    self.distinguishable(state1, state2, partition)
                {
                    if target_group1 == group_number {
                        group1.insert(*state_id1);
                        group2.insert(*state_id2);
                    } else {
                        group2.insert(*state_id1);
                        group1.insert(*state_id2);
                    }
                } else {
                    group1.insert(*state_id1);
                    group1.insert(*state_id2);
                }
            });
        let mut partitions = Partition::new();
        partitions.push(group1);
        if !group2.is_empty() {
            partitions.push(group2);
        }
        partitions
    }

    /// States are distinguishable if they have transitions to different groups in the partition on
    /// the same character class.
    fn distinguishable(
        &self,
        state1: &DfaState,
        state2: &DfaState,
        partition: &[StateGroup],
    ) -> Option<(usize, usize)> {
        for char_class in &self.char_classes {
            let target_state1 = self.transitions[&state1.id].get(char_class).cloned();
            let target_state2 = self.transitions[&state2.id].get(char_class).cloned();
            if let (Some(target_state1), Some(target_state2)) = (target_state1, target_state2) {
                let target_group1 = self.find_group(target_state1, partition);
                let target_group2 = self.find_group(target_state2, partition);
                if let (Some(target_group1), Some(target_group2)) = (target_group1, target_group2) {
                    if target_group1 != target_group2 {
                        trace!(
                            "States {}->{} and {}->{} are distinguishable on char class {:?}",
                            state1.id,
                            target_state1,
                            state2.id,
                            target_state2,
                            char_class
                        );
                        return Some((target_group1, target_group2));
                    }
                }
            }
        }
        trace!(
            "States {} and {} are not distinguishable",
            state1.id,
            state2.id,
        );
        None
    }

    fn find_group(&self, state_id: StateId, partition: &[StateGroup]) -> Option<usize> {
        partition.iter().position(|group| group.contains(&state_id))
    }

    /// Create a DFA from a partition.
    /// If a StateGroup contains more than one state, the states are merged into one state.
    /// The transitions are updated accordingly.
    /// The accepting states are updated accordingly.
    /// The new DFA is returned.
    fn create_from_partition(&self, partition: &[StateGroup]) -> Dfa {
        let mut dfa = Dfa {
            states: Vec::new(),
            patterns: self.patterns.clone(),
            accepting_states: BTreeMap::new(),
            char_classes: self.char_classes.clone(),
            transitions: self.transitions.clone(),
        };

        // The state renumber map is used to map the old state ids to the new state ids.
        let mut state_renumber_map = BTreeMap::new();
        // The state merge map is used to merge the states into the representative state.
        let mut state_merge_map: BTreeMap<StateId, Vec<StateId>> = BTreeMap::new();

        for group in partition {
            let new_state_id = dfa.add_representive_state(&group, &self.accepting_states);
            // First state in group is the representative state.
            let representative_state_id = group.first().unwrap();
            state_renumber_map.insert(*representative_state_id, new_state_id);
            for state_id in group.iter().skip(1) {
                // Take over the accepting state if the state is an accepting state.
                if let Some(pattern_id) = self.accepting_states.get(state_id) {
                    dfa.accepting_states.insert(new_state_id, *pattern_id);
                }
                // Merge the states into the representative state.
                state_merge_map
                    .entry(*representative_state_id)
                    .or_default()
                    .push(*state_id);
            }
        }

        // First we merge the states into the representative state.
        for group in partition {
            trace!("Merging group: {:?}", group);
            if group.len() > 1 {
                let representative_state_id = group.first().unwrap();
                let states_to_merge = state_merge_map
                    .get(representative_state_id)
                    .unwrap()
                    .clone();
                trace!(
                    "States to merge: {:?} into {}",
                    states_to_merge,
                    representative_state_id
                );
                for state_to_merge in states_to_merge {
                    // Remove the transitions of the states to merge.
                    // They are redundant and equal to the ones of the representative state's transitions.
                    dfa.remove_tansitions_of_state(state_to_merge);
                }
                // self.merge_states_into_dfa_state(
                //     &mut dfa,
                //     *representative_state_id,
                //     states_to_merge,
                // );
            }
        }

        // Then we renumber the states according to the renumber map.
        dfa.renumber_states(state_renumber_map);

        trace!("Minimized DFA:\n{}", dfa);

        dfa
    }

    // fn merge_states_into_dfa_state<I>(
    //     &self,
    //     dfa: &mut Dfa,
    //     representative_state_id: StateId,
    //     states_to_merge: I,
    // ) where
    //     I: IntoIterator<Item = StateId>,
    // {
    //     for state_to_merge in states_to_merge {
    //         // Remove the transitions of the states to merge.
    //         // They are redundant and equal to the ones of the representative state's transitions.
    //         dfa.remove_tansitions_of_state(state_to_merge);
    //     }
    // }

    fn renumber_transitions(&mut self, state_renumber_map: BTreeMap<StateId, StateId>) {
        // Create a vector because we dont want to mess the transitins map while renumbering.
        let mut transitions = self
            .transitions
            .iter()
            .map(|(s, t)| (*s, t.clone()))
            .collect::<Vec<_>>();

        // Remove transitions with source states that are not in the renumber map as old number.
        transitions.retain(|(s, _)| state_renumber_map.contains_key(s));

        // Renumber the source states in the transitions.
        let mut new_transitions = Vec::new();
        for (old_state_id, new_state_id) in &state_renumber_map {
            if let Some((_, trans)) = transitions.iter().find(|(s, _)| *s == *old_state_id) {
                new_transitions.push((new_state_id, trans));
            }
        }

        for (old_state_id, new_state_id) in state_renumber_map {
            self.renumber_state_in_tansitions(old_state_id, new_state_id, &mut transitions);
        }

        // Then we re-insert the transitions after the renumbering.
        self.transitions = transitions.into_iter().collect();
    }

    /// Replace all transitions to the old state with transitions to the representative state
    /// while keeping the char class.
    fn renumber_state_in_tansitions(
        &mut self,
        old_state_id: StateId,
        new_state_id: StateId,
        transitions: &mut [(StateId, BTreeMap<CharacterClass, StateId>)],
    ) {
        if old_state_id == new_state_id {
            return;
        }

        trace!("Renumber state {} to {}", old_state_id, new_state_id);
        // Update source states in transitions.
        transitions.iter_mut().for_each(|transition| {
            if transition.0 == old_state_id {
                transition.0 = new_state_id;
            }
        });

        // Update target states in transitions.
        transitions.iter_mut().for_each(|transition| {
            for (_, target_state) in transition.1.iter_mut() {
                if *target_state == old_state_id {
                    *target_state = new_state_id;
                }
            }
        });

        // Update the accepting states.
        // if let Some(pattern_id) = self.accepting_states.remove(&old_state_id) {
        //     trace!(
        //         "Update accepting state {} to {}",
        //         old_state_id,
        //         new_state_id
        //     );
        //     self.accepting_states.insert(new_state_id, pattern_id);
        // }
    }

    #[inline]
    fn remove_tansitions_of_state(&mut self, state_id: StateId) {
        // Remove the given state's transitions.
        trace!("Remove transitions of state {}", state_id);
        self.transitions.remove(&state_id);
    }

    fn renumber_states(&mut self, state_renumber_map: BTreeMap<StateId, StateId>) {
        // Then we re-insert the states according to the renumber map.
        let mut states: Vec<DfaState> = Vec::new();
        mem::swap(&mut self.states, &mut states);

        trace!("Renumber states: {:?}", state_renumber_map);

        // Check if the renumber map is correct.
        // The first state should have id 0 and the last state should have id states.len() - 1.
        debug_assert!(matches!(
            state_renumber_map.iter().next(),
            Some((_, s)) if s.as_usize() == 0));
        debug_assert!(matches!(
            state_renumber_map.iter().last(),
            Some((_, s)) if s.as_usize() == states.len() - 1));

        for state_id in 0..state_renumber_map.len() {
            trace!("Insert state {}", state_id);
            self.states.push(DfaState::new(state_id.into(), Vec::new()));
        }

        // Update the accepting states.
        self.accepting_states = self
            .accepting_states
            .iter()
            .map(|(state_id, pattern_id)| {
                if let Some(new_state_id) = state_renumber_map.get(state_id) {
                    (*new_state_id, *pattern_id)
                } else {
                    (*state_id, *pattern_id)
                }
            })
            .collect();

        // Renumber states in the transitions.
        self.renumber_transitions(state_renumber_map);
    }
}

impl From<MultiPatternNfa> for Dfa {
    fn from(nfa: MultiPatternNfa) -> Self {
        Dfa::from_nfa(nfa)
    }
}

impl std::fmt::Display for Dfa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "DFA")?;
        writeln!(f, "States:")?;
        for state in &self.states {
            writeln!(f, "{:?}", state)?;
        }
        writeln!(f, "Patterns:")?;
        for pattern in &self.patterns {
            writeln!(f, "{}", pattern)?;
        }
        writeln!(f, "Accepting states:")?;
        for (state_id, pattern_id) in &self.accepting_states {
            writeln!(f, "{}: {}", state_id, pattern_id)?;
        }
        writeln!(f, "Char classes:")?;
        for char_class in &self.char_classes {
            writeln!(f, "{:?}", char_class)?;
        }
        writeln!(f, "Transitions:")?;
        for (source_id, targets) in &self.transitions {
            write!(f, "{} -> ", source_id)?;
            for (char_class, target_id) in targets {
                write!(f, "{}:{} ", char_class.ast.0, target_id)?;
            }
            writeln!(f)?
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct DfaState {
    id: StateId,
    // The ids of the NFA states that constitute this DFA state. The id can only be used as indices
    // into the NFA states.
    nfa_states: Vec<StateId>,
    // The marked flag is used to mark a state as visited during the subset construction algorithm.
    marked: bool,
}

impl DfaState {
    /// Create a new DFA state solely from the NFA states that constitute the DFA state.
    pub(crate) fn new(id: StateId, nfa_states: Vec<StateId>) -> Self {
        DfaState {
            id,
            nfa_states,
            marked: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dot::{dfa_render_to, multi_render_to};
    use std::fs::File;

    const PATTERNS: &[&str] = &[
        "\\r\\n|\\r|\\n",
        "[\\s--\\r\\n]+",
        "(//.*(\\r\\n|\\r|\\n))",
        "(/\\*.*?\\*/)",
        "%start",
        "%title",
        "%comment",
        "%user_type",
        "=",
        "%grammar_type",
        "%line_comment",
        "%block_comment",
        "%auto_newline_off",
        "%auto_ws_off",
        "%on",
        "%enter",
        "%%",
        "::",
        ":",
        ";",
        "\\|",
        "<",
        ">",
        "\"(\\\\.|[^\\\\])*?\"",
        "'(\\\\'|[^'])*?'",
        "\\u{2F}(\\\\.|[^\\\\])*?\\u{2F}",
        "\\(",
        "\\)",
        "\\[",
        "\\]",
        "\\{",
        "\\}",
        "[a-zA-Z_][a-zA-Z0-9_]*",
        "%scanner",
        ",",
        "%sc",
        "%push",
        "%pop",
        "\\^",
        ".",
    ];

    // Initialize the logger for the tests
    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    // A macro that simplifies the rendering of a dot file for test purposes
    macro_rules! dfa_render_to {
        ($nfa:expr, $label:expr) => {
            let mut f = File::create(concat!($label, ".dot")).unwrap();
            dfa_render_to($nfa, $label, &mut f);
        };
    }

    // A macro that simplifies the rendering of a dot file for test purposes
    macro_rules! multi_render_to {
        ($nfa:expr, $label:expr) => {
            let mut f = File::create(concat!($label, ".dot")).unwrap();
            multi_render_to($nfa, $label, &mut f);
        };
    }

    #[test]
    fn test_dfa_from_nfa() {
        let mut multi_pattern_nfa = MultiPatternNfa::new();
        let result = multi_pattern_nfa.add_pattern("b|a{2,3}");
        assert!(result.is_ok());
        let result = multi_pattern_nfa.add_pattern("(a|b)*abb");
        assert!(result.is_ok());
        multi_render_to!(&multi_pattern_nfa, "input_nfa");

        let dfa = Dfa::from(multi_pattern_nfa);

        dfa_render_to!(&dfa, "dfa_from_nfa");

        assert_eq!(dfa.states().len(), 9);
        assert_eq!(dfa.patterns().len(), 2);
        assert_eq!(dfa.accepting_states().len(), 4);
        assert_eq!(dfa.char_classes().len(), 2);
    }

    #[test]
    fn test_dfa_from_nfa_2() {
        let mut multi_pattern_nfa = MultiPatternNfa::new();
        let result = multi_pattern_nfa.add_pattern("in");
        assert!(result.is_ok());
        let result = multi_pattern_nfa.add_pattern("int");
        assert!(result.is_ok());

        let dfa = Dfa::from(multi_pattern_nfa);

        dfa_render_to!(&dfa, "dfa_from_nfa_2");

        assert_eq!(dfa.states().len(), 4);
        assert_eq!(dfa.patterns().len(), 2);
        assert_eq!(dfa.accepting_states().len(), 2);
        assert_eq!(dfa.char_classes().len(), 3);
    }

    #[test]
    fn test_dfa_from_nfa_3() {
        let mut multi_pattern_nfa = MultiPatternNfa::new();

        let result = multi_pattern_nfa.add_patterns(PATTERNS);
        if let Err(e) = result {
            panic!("Error: {}", e);
        }

        let dfa = Dfa::from(multi_pattern_nfa);

        dfa_render_to!(&dfa, "dfa_from_nfa_3");

        assert_eq!(dfa.states().len(), 154);
        assert_eq!(dfa.patterns().len(), 40);
        assert_eq!(dfa.accepting_states().len(), 45);
        assert_eq!(dfa.char_classes().len(), 50);
    }

    #[test]
    fn test_dfa_minimize() {
        init();

        let mut multi_pattern_nfa = MultiPatternNfa::new();
        let result = multi_pattern_nfa.add_pattern("(a|b)*abb");
        assert!(result.is_ok());
        // let result = multi_pattern_nfa.add_pattern("ab");
        // assert!(result.is_ok());
        // let result = multi_pattern_nfa.add_pattern("cd");
        // assert!(result.is_ok());
        // let result = multi_pattern_nfa.add_pattern("ef");
        // assert!(result.is_ok());

        let dfa = Dfa::from(multi_pattern_nfa);
        dfa_render_to!(&dfa, "dfa_unminimized");

        let minimized_dfa = dfa.minimize();
        dfa_render_to!(&minimized_dfa, "dfa_minimized");

        assert_eq!(minimized_dfa.states().len(), 4);
        assert_eq!(minimized_dfa.patterns().len(), 1);
        assert_eq!(minimized_dfa.accepting_states().len(), 1);
        assert_eq!(minimized_dfa.char_classes().len(), 2);
    }

    #[test]
    fn test_dfa_minimize_2() {
        init();

        let mut multi_pattern_nfa = MultiPatternNfa::new();

        let result = multi_pattern_nfa.add_patterns(PATTERNS);
        if let Err(e) = result {
            panic!("Error: {}", e);
        }

        let dfa = Dfa::from(multi_pattern_nfa);
        dfa_render_to!(&dfa, "dfa_unminimized_2");

        let minimized_dfa = dfa.minimize();
        dfa_render_to!(&minimized_dfa, "dfa_minimized_2");

        assert_eq!(minimized_dfa.states().len(), 73);
        assert_eq!(minimized_dfa.patterns().len(), 40);
        assert_eq!(
            minimized_dfa.accepting_states().len(),
            minimized_dfa.patterns().len()
        );
        assert_eq!(minimized_dfa.char_classes().len(), 50);
    }
}
