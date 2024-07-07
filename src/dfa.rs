//! This module contains the DFA implementation.
//! The DFA is used to match a string against a regex pattern.
//! The DFA is generated from the NFA using the subset construction algorithm.

use itertools::Itertools;
use log::trace;
use std::collections::{BTreeMap, BTreeSet};

use crate::{character_class::CharacterClass, MultiPatternNfa, PatternId, StateId};

// The type definitions for the subset construction algorithm.
type StateGroup = BTreeSet<StateId>;
type Partition = Vec<StateGroup>;

// A data type that is calcuated from the transitions of a DFA state so that for each character
// class the target state is mapped to the partition group it belongs to.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct TransitionsToPartitionGroups(Vec<(CharacterClass, usize)>);

impl TransitionsToPartitionGroups {
    fn new() -> Self {
        TransitionsToPartitionGroups(Vec::new())
    }

    fn insert(&mut self, char_class: CharacterClass, partition_group: usize) {
        self.0.push((char_class, partition_group));
    }
}

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
        group: &BTreeSet<StateId>,
        accepting_states: &BTreeMap<StateId, PatternId>,
    ) -> StateId {
        let state_id = self.states.len();
        let state = DfaState::new(state_id.into(), Vec::new());

        // First state in group is the representative state.
        let representative_state_id = group.first().unwrap();

        trace!(
            "Add representive state {} with id {}",
            representative_state_id,
            state_id
        );

        // Insert the representative state into the accepting states if any state in its group is
        // an accepting state.
        for state_in_group in group.iter() {
            if let Some(pattern_id) = accepting_states.get(state_in_group) {
                trace!(
                    "* State {} with pattern id {} is accepting state (from state {}).",
                    state_id,
                    pattern_id,
                    state_in_group
                );
                self.accepting_states.insert(state_id.into(), *pattern_id);
            }
        }

        self.states.push(state);
        StateId::new(state_id)
    }

    /// Trace out a partition of the DFA.
    #[allow(dead_code)]
    fn trace_partition(context: &str, partition: &[StateGroup]) {
        trace!("Partition {}:", context);
        for (i, group) in partition.iter().enumerate() {
            trace!("Group {}: {:?}", i, group);
        }
    }

    #[allow(dead_code)]
    fn trace_transitions_to_groups(
        state_id: StateId,
        transitions_to_groups: &TransitionsToPartitionGroups,
    ) {
        trace!("Transitions of state {} to groups:", state_id);
        for (char_class, group) in &transitions_to_groups.0 {
            trace!(
                "{}:{} -> {}",
                char_class.ast.0,
                char_class.id.as_usize(),
                group
            );
        }
    }

    /// Minimize the DFA.
    /// The Nfa states are removed from the DFA states during minimization. They are not needed
    /// anymore after the DFA is created.
    pub fn minimize(&self) -> Self {
        trace!("Minimize DFA ----------------------------");
        let mut partition_old = self.calculate_initial_partition();
        let mut partition_new = Partition::new();
        let mut changed = true;
        Self::trace_partition("initial", &partition_old);

        while changed {
            partition_new = self.calculate_new_partition(&partition_old);
            Self::trace_partition("new", &partition_new);
            changed = partition_new != partition_old;
            partition_old.clone_from(&partition_new);
        }

        self.create_from_partition(&partition_new)
    }

    /// The start partition is created as follows:
    /// 1. The accepting states are put each in a partition with the same matched pattern id.
    ///    This follows from the constraint of the DFA that only one pattern can match.
    /// 2. The non-accepting states are put together in one partition that has the id of the
    ///    first unsued pattern id.
    ///
    /// The partitions are stored in a vector of vectors.
    ///
    /// The key building function for the Itertools::chunk_by method is used to create the
    /// partitions. For accepting states the key is the state id, for non-accepting states
    /// the key is the state id of the first non-accepting state.
    fn calculate_initial_partition(&self) -> Partition {
        let group_id_non_accepting_states: StateId = self.accepting_states.len().into();
        self.states
            .clone()
            .into_iter()
            .chunk_by(|state| {
                if let Some(pattern_id) = self.accepting_states.get(&state.id) {
                    pattern_id.as_usize().into()
                } else {
                    group_id_non_accepting_states
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
            })
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
        let mut new_partition = Partition::new();
        for (index, group) in partition.iter().enumerate() {
            // The new group receives the states from the old group which are distiguishable from
            // the other states in group.
            self.split_group(index, group, partition)
                .into_iter()
                .for_each(|new_group| {
                    new_partition.push(new_group);
                });
        }
        new_partition
    }

    fn split_group(
        &self,
        group_index: usize,
        group: &StateGroup,
        partition: &[StateGroup],
    ) -> Partition {
        // If the group contains only one state, the group can't be split further.
        if group.len() == 1 {
            return vec![group.clone()];
        }
        trace!("Split group {}: {:?}", group_index, group);
        let mut transition_map_to_states: BTreeMap<TransitionsToPartitionGroups, StateGroup> =
            BTreeMap::new();
        for state_id in group {
            let transitions_to_partition =
                self.build_transitions_to_partition_group(*state_id, partition);
            transition_map_to_states
                .entry(transitions_to_partition)
                .or_default()
                .insert(*state_id);
        }
        transition_map_to_states
            .into_values()
            .collect::<Partition>()
    }

    /// Build a modified transition data structure of a given DFA state that maps states to the
    /// partition group.
    /// The partition group is the index of the group in the partition.
    /// The modified transition data structure is returned.
    /// The modified transition data structure is used to determine if two states are distinguish
    /// based on the transitions of the DFA.
    fn build_transitions_to_partition_group(
        &self,
        state_id: StateId,
        partition: &[StateGroup],
    ) -> TransitionsToPartitionGroups {
        if let Some(transitions_of_state) = self.transitions.get(&state_id) {
            let mut transitions_to_partition_groups = TransitionsToPartitionGroups::new();
            for transition in transitions_of_state {
                let partition_group = self.find_group(*transition.1, partition).unwrap();
                transitions_to_partition_groups.insert(transition.0.clone(), partition_group);
            }
            Self::trace_transitions_to_groups(state_id, &transitions_to_partition_groups);
            transitions_to_partition_groups
        } else {
            trace!("** State {} has no transitions.", state_id);
            TransitionsToPartitionGroups::new()
        }
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
        trace!("Create DFA ------------------------------");
        trace!("from partition {:?}", partition);

        let mut dfa = Dfa {
            states: Vec::new(),
            patterns: self.patterns.clone(),
            accepting_states: BTreeMap::new(),
            char_classes: self.char_classes.clone(),
            transitions: self.transitions.clone(),
        };

        for group in partition {
            // For each group we add a representative state to the DFA.
            // It's id is the index of the group in the partition.
            // This function also updates the accepting states of the DFA.
            dfa.add_representive_state(group, &self.accepting_states);
        }

        // Then renumber the states in the transitions.
        dfa.update_transitions(partition);

        trace!("Minimized DFA:\n{}", dfa);

        dfa
    }

    fn update_transitions(&mut self, partition: &[StateGroup]) {
        // Create a vector because we dont want to mess the transitins map while renumbering.
        let mut transitions = self
            .transitions
            .iter()
            .map(|(s, t)| (*s, t.clone()))
            .collect::<Vec<_>>();

        Self::merge_transitions(partition, &mut transitions);
        Self::renumber_states_in_transitions(partition, &mut transitions);

        self.transitions = transitions.into_iter().collect();
    }

    fn merge_transitions(
        partition: &[BTreeSet<StateId>],
        transitions: &mut Vec<(StateId, BTreeMap<CharacterClass, StateId>)>,
    ) {
        // Remove all transitions that do not belong to the representive states of a group.
        // The representive states are the first states in the groups.
        for group in partition {
            debug_assert!(!group.is_empty());
            if group.len() == 1 {
                continue;
            }
            let representive_state_id = group.first().unwrap();
            for state_id in group.iter().skip(1) {
                Self::merge_transitions_of_state(*state_id, *representive_state_id, transitions);
            }
        }
    }

    fn merge_transitions_of_state(
        state_id: StateId,
        representive_state_id: StateId,
        transitions: &mut Vec<(StateId, BTreeMap<CharacterClass, StateId>)>,
    ) {
        if let Some(rep_pos) = transitions
            .iter()
            .position(|(s, _)| *s == representive_state_id)
        {
            let mut rep_trans = transitions.get_mut(rep_pos).unwrap().1.clone();
            if let Some(pos) = transitions.iter().position(|(s, _)| *s == state_id) {
                let (_, transitions_of_state) = transitions.get_mut(pos).unwrap();
                for (char_class, target_state) in transitions_of_state.iter() {
                    rep_trans.insert(char_class.clone(), *target_state);
                }
                // Remove the transitions of the state that is merged into the representative state.
                transitions.remove(pos);
            }
            transitions[rep_pos].1 = rep_trans;
        }
    }

    fn renumber_states_in_transitions(
        partition: &[StateGroup],
        transitions: &mut [(StateId, BTreeMap<CharacterClass, StateId>)],
    ) {
        let find_group_of_state = |state_id: StateId| -> StateId {
            for (group_id, group) in partition.iter().enumerate() {
                if group.contains(&state_id) {
                    return group_id.into();
                }
            }
            panic!("State {} not found in partition.", state_id);
        };

        for transition in transitions.iter_mut() {
            transition.0 = find_group_of_state(transition.0);
            for target_state in transition.1.values_mut() {
                *target_state = find_group_of_state(*target_state);
            }
        }
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

    const PATTERNS_2: &[&str] = &[
        "bitf", "ccfg", "chln", "clbl", "cnst", "devv", "drct", "expt", "hmir", "imag", "impl",
        "impt", "in", "inbl", "inhe", "inpt", "inst", "insv", "intl", "io-v", "locl", "out",
        "outp", "priv", "protd", "proto", "publ", "refr", "retn", "rflb", "ronl", "sysv", "temp",
        "timd", "tskv", "type", "unkn",
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

        assert_eq!(minimized_dfa.states().len(), 151);
        assert_eq!(minimized_dfa.patterns().len(), 40);
        assert_eq!(minimized_dfa.accepting_states().len(), 45);
        assert_eq!(minimized_dfa.char_classes().len(), 50);
    }

    #[test]
    fn test_dfa_minimize_3() {
        init();

        let mut multi_pattern_nfa = MultiPatternNfa::new();
        let result = multi_pattern_nfa.add_pattern("in");
        assert!(result.is_ok());
        let result = multi_pattern_nfa.add_pattern("int");
        assert!(result.is_ok());

        let dfa = Dfa::from(multi_pattern_nfa);
        dfa_render_to!(&dfa, "dfa_unminimized_3");

        let minimized_dfa = dfa.minimize();
        dfa_render_to!(&minimized_dfa, "dfa_minimized_3");

        assert_eq!(minimized_dfa.states().len(), 4);
        assert_eq!(minimized_dfa.patterns().len(), 2);
        assert_eq!(minimized_dfa.accepting_states().len(), 2);
        assert_eq!(minimized_dfa.char_classes().len(), 3);
    }

    #[test]
    fn test_dfa_minimize_4() {
        init();

        let mut multi_pattern_nfa = MultiPatternNfa::new();

        let result = multi_pattern_nfa.add_patterns(PATTERNS_2);
        if let Err(e) = result {
            panic!("Error: {}", e);
        }
        multi_render_to!(&multi_pattern_nfa, "nfa_4");

        let dfa = Dfa::from(multi_pattern_nfa);
        dfa_render_to!(&dfa, "dfa_unminimized_4");

        let minimized_dfa = dfa.minimize();
        dfa_render_to!(&minimized_dfa, "dfa_minimized_4");

        assert_eq!(minimized_dfa.states().len(), 107);
        assert_eq!(minimized_dfa.patterns().len(), 37);
        assert_eq!(minimized_dfa.accepting_states().len(), 37);
        assert_eq!(minimized_dfa.char_classes().len(), 23);
    }

    #[test]
    fn test_dfa_minimize_5() {
        init();

        let mut multi_pattern_nfa = MultiPatternNfa::new();

        let result = multi_pattern_nfa.add_pattern("a{1,2}b{2,}c{3}");
        if let Err(e) = result {
            panic!("Error: {}", e);
        }
        multi_render_to!(&multi_pattern_nfa, "nfa_5");

        let dfa = Dfa::from(multi_pattern_nfa);
        dfa_render_to!(&dfa, "dfa_unminimized_5");

        let minimized_dfa = dfa.minimize();
        dfa_render_to!(&minimized_dfa, "dfa_minimized_5");

        assert_eq!(minimized_dfa.states().len(), 8);
        assert_eq!(minimized_dfa.patterns().len(), 1);
        assert_eq!(minimized_dfa.accepting_states().len(), 1);
        assert_eq!(minimized_dfa.char_classes().len(), 3);
    }
}
