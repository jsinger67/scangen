//! This module contains the DFA implementation.
//! The DFA is used to match a string against a regex pattern.
//! The DFA is generated from the NFA using the subset construction algorithm.

use itertools::Itertools;
use log::trace;
use regex_automata::{util::primitives::StateID, PatternID};
use std::collections::{BTreeMap, BTreeSet};

use crate::{character_class::CharacterClass, MultiPatternNfa, Result};

// The type definitions for the subset construction algorithm.
pub(crate) type StateGroup = BTreeSet<StateID>;
pub(crate) type Partition = Vec<StateGroup>;

// A data type that is calcuated from the transitions of a DFA state so that for each character
// class the target state is mapped to the partition group it belongs to.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct TransitionsToPartitionGroups(pub(crate) Vec<(CharacterClass, usize)>);

impl TransitionsToPartitionGroups {
    pub(crate) fn new() -> Self {
        TransitionsToPartitionGroups(Vec::new())
    }

    pub(crate) fn insert(&mut self, char_class: CharacterClass, partition_group: usize) {
        self.0.push((char_class, partition_group));
    }
}

/// The DFA implementation.
#[derive(Debug, Default)]
pub struct Dfa {
    // The states of the DFA. The start state is always the first state in the vector, i.e. state 0.
    states: Vec<DfaState>,
    // The patterns for the accepting states.
    patterns: Vec<String>,
    // The accepting states of the DFA as well as the corresponding pattern id.
    accepting_states: BTreeMap<StateID, PatternID>,
    // The character classes used in the DFA.
    char_classes: Vec<CharacterClass>,
    // The transitions of the DFA.
    transitions: BTreeMap<StateID, BTreeMap<CharacterClass, StateID>>,
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
    pub(crate) fn accepting_states(&self) -> &BTreeMap<StateID, PatternID> {
        &self.accepting_states
    }

    /// Get the pattern id if the given state is an accepting state.
    pub(crate) fn pattern_id(&self, state_id: StateID) -> Option<PatternID> {
        self.accepting_states.get(&state_id).copied()
    }

    /// Get the character classes used in the DFA.
    pub(crate) fn char_classes(&self) -> &[CharacterClass] {
        &self.char_classes
    }

    /// Get the transitions of the DFA.
    pub(crate) fn transitions(&self) -> &BTreeMap<StateID, BTreeMap<CharacterClass, StateID>> {
        &self.transitions
    }

    /// Create a DFA from a multi-pattern NFA.
    /// The DFA is created using the subset construction algorithm.
    fn try_from_nfa(nfa: MultiPatternNfa) -> Result<Self> {
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
        let start_state = nfa.epsilon_closure(StateID::default());
        // The initial state is the start state of the DFA.
        let initial_state = dfa.add_state_if_new(start_state, &accepting_states)?;
        // The work list is used to keep track of the states that need to be processed.
        let mut work_list = vec![initial_state];
        // The marked flag is used to mark a state as visited during the subset construction algorithm.
        dfa.states[initial_state].marked = true;

        while let Some(state_id) = work_list.pop() {
            let nfa_states = dfa.states[state_id].nfa_states.clone();
            for char_class in dfa.char_classes.clone() {
                let target_states =
                    nfa.epsilon_closure_set(nfa.move_set(&nfa_states, char_class.id()));
                if !target_states.is_empty() {
                    let target_state = dfa.add_state_if_new(target_states, &accepting_states)?;
                    dfa.transitions
                        .entry(state_id)
                        .or_default()
                        .insert(char_class.clone(), target_state);
                    if !dfa.states[target_state].marked {
                        dfa.states[target_state].marked = true;
                        work_list.push(target_state);
                    }
                }
            }
        }

        Ok(dfa)
    }

    /// Add a state to the DFA if it does not already exist.
    /// The state is identified by the NFA states that constitute the DFA state.
    /// The accepting states are used to determine if the DFA state is an accepting state.
    fn add_state_if_new<I>(
        &mut self,
        nfa_states: I,
        accepting_states: &BTreeMap<StateID, PatternID>,
    ) -> Result<StateID>
    where
        I: IntoIterator<Item = StateID>,
    {
        let mut nfa_states: Vec<StateID> = nfa_states.into_iter().collect();
        nfa_states.sort_unstable();
        nfa_states.dedup();
        if let Some(state_id) = self
            .states
            .iter()
            .position(|state| state.nfa_states == nfa_states)
        {
            return Ok(StateID::new(state_id)?);
        }

        let state_id = StateID::new(self.states.len())?;
        let state = DfaState::new(state_id, nfa_states);

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
                self.accepting_states.insert(state_id, *pattern_id);
                break;
            }
        }

        // trace!("Add state: {}: {:?}", state.id.as_usize(), state.nfa_states);

        self.states.push(state);
        Ok(state_id)
    }

    /// Add a representative state to the DFA.
    /// The representative state is the first state in the group.
    /// The accepting states are used to determine if the DFA state is an accepting state.
    /// The new state id is returned.
    fn add_representive_state(
        &mut self,
        group: &BTreeSet<StateID>,
        accepting_states: &BTreeMap<StateID, PatternID>,
    ) -> Result<StateID> {
        let state_id = StateID::new(self.states.len())?;
        let state = DfaState::new(state_id, Vec::new());

        // First state in group is the representative state.
        // let representative_state_id = group.first().unwrap();

        // trace!(
        //     "Add representive state {} with id {}",
        //     representative_state_id.as_usize(),
        //     state_id.as_usize()
        // );

        // Insert the representative state into the accepting states if any state in its group is
        // an accepting state.
        for state_in_group in group.iter() {
            if let Some(pattern_id) = accepting_states.get(state_in_group) {
                // trace!(
                //     "* State {} with pattern id {} is accepting state (from state {}).",
                //     state_id.as_usize(),
                //     pattern_id.as_usize(),
                //     state_in_group.as_usize()
                // );
                self.accepting_states.insert(state_id, *pattern_id);
            }
        }

        self.states.push(state);
        Ok(state_id)
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
        _state_id: StateID,
        transitions_to_groups: &TransitionsToPartitionGroups,
    ) {
        // trace!("Transitions of state {} to groups:", state_id.as_usize());
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
    pub fn minimize(&self) -> Result<Self> {
        // trace!("Minimize DFA ----------------------------");
        let mut partition_old = self.calculate_initial_partition();
        let mut partition_new = Partition::new();
        let mut changed = true;
        // Self::trace_partition("initial", &partition_old);

        while changed {
            partition_new = self.calculate_new_partition(&partition_old);
            // Self::trace_partition("new", &partition_new);
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
        let group_id_non_accepting_states: StateID =
            StateID::new_unchecked(self.accepting_states.len());
        self.states
            .clone()
            .into_iter()
            .chunk_by(|state| {
                if let Some(pattern_id) = self.pattern_id(state.id) {
                    StateID::new_unchecked(pattern_id.as_usize())
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
        _group_index: usize,
        group: &StateGroup,
        partition: &[StateGroup],
    ) -> Partition {
        // If the group contains only one state, the group can't be split further.
        if group.len() == 1 {
            return vec![group.clone()];
        }
        // trace!("Split group {}: {:?}", group_index, group);
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
        state_id: StateID,
        partition: &[StateGroup],
    ) -> TransitionsToPartitionGroups {
        if let Some(transitions_of_state) = self.transitions.get(&state_id) {
            let mut transitions_to_partition_groups = TransitionsToPartitionGroups::new();
            for transition in transitions_of_state {
                let partition_group = self.find_group(*transition.1, partition).unwrap();
                transitions_to_partition_groups.insert(transition.0.clone(), partition_group);
            }
            // Self::trace_transitions_to_groups(state_id, &transitions_to_partition_groups);
            transitions_to_partition_groups
        } else {
            // trace!("** State {} has no transitions.", state_id.as_usize());
            TransitionsToPartitionGroups::new()
        }
    }

    fn find_group(&self, state_id: StateID, partition: &[StateGroup]) -> Option<usize> {
        partition.iter().position(|group| group.contains(&state_id))
    }

    /// Create a DFA from a partition.
    /// If a StateGroup contains more than one state, the states are merged into one state.
    /// The transitions are updated accordingly.
    /// The accepting states are updated accordingly.
    /// The new DFA is returned.
    fn create_from_partition(&self, partition: &[StateGroup]) -> Result<Dfa> {
        // trace!("Create DFA ------------------------------");
        // trace!("from partition {:?}", partition);

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
            dfa.add_representive_state(group, &self.accepting_states)?;
        }

        // Then renumber the states in the transitions.
        dfa.update_transitions(partition);

        // trace!("Minimized DFA:\n{}", dfa);

        Ok(dfa)
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
        partition: &[BTreeSet<StateID>],
        transitions: &mut Vec<(StateID, BTreeMap<CharacterClass, StateID>)>,
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
        state_id: StateID,
        representive_state_id: StateID,
        transitions: &mut Vec<(StateID, BTreeMap<CharacterClass, StateID>)>,
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
        transitions: &mut [(StateID, BTreeMap<CharacterClass, StateID>)],
    ) {
        let find_group_of_state = |state_id: StateID| -> StateID {
            for (group_id, group) in partition.iter().enumerate() {
                if group.contains(&state_id) {
                    return StateID::new_unchecked(group_id);
                }
            }
            panic!("State {} not found in partition.", state_id.as_usize());
        };

        for transition in transitions.iter_mut() {
            transition.0 = find_group_of_state(transition.0);
            for target_state in transition.1.values_mut() {
                *target_state = find_group_of_state(*target_state);
            }
        }
    }
}

impl TryFrom<MultiPatternNfa> for Dfa {
    type Error = crate::errors::ScanGenError;

    fn try_from(nfa: MultiPatternNfa) -> Result<Self> {
        Dfa::try_from_nfa(nfa)
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
            writeln!(f, "{}: {}", state_id.as_usize(), pattern_id.as_usize())?;
        }
        writeln!(f, "Char classes:")?;
        for char_class in &self.char_classes {
            writeln!(f, "{:?}", char_class)?;
        }
        writeln!(f, "Transitions:")?;
        for (source_id, targets) in &self.transitions {
            write!(f, "{} -> ", source_id.as_usize())?;
            for (char_class, target_id) in targets {
                write!(f, "{}:{} ", char_class.ast.0, target_id.as_usize())?;
            }
            writeln!(f)?
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DfaState {
    id: StateID,
    // The ids of the NFA states that constitute this DFA state. The id can only be used as indices
    // into the NFA states.
    nfa_states: Vec<StateID>,
    // The marked flag is used to mark a state as visited during the subset construction algorithm.
    marked: bool,
}

impl DfaState {
    /// Create a new DFA state solely from the NFA states that constitute the DFA state.
    pub fn new(id: StateID, nfa_states: Vec<StateID>) -> Self {
        DfaState {
            id,
            nfa_states,
            marked: false,
        }
    }

    /// Get the id of the DFA state.
    pub fn id(&self) -> StateID {
        self.id
    }

    /// Get the NFA states that constitute the DFA state.
    pub fn nfa_states(&self) -> &[StateID] {
        &self.nfa_states
    }

    /// Get the marked flag of the DFA state.
    pub fn marked(&self) -> bool {
        self.marked
    }

    /// Set the marked flag of the DFA state.
    pub fn set_marked(&mut self, marked: bool) {
        self.marked = marked;
    }
}

#[cfg(test)]
mod tests {

    use crate::{dfa_render_to, multi_nfa_render_to};

    use super::*;

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

    // A data type that provides test data for the DFA minimization tests.
    struct TestData {
        name: &'static str,
        patterns: &'static [&'static str],
        states: usize,
        accepting_states: usize,
        char_classes: usize,
        min_states: usize,
        min_accepting_states: usize,
    }

    // Test data for the DFA minimization tests.
    const TEST_DATA: &[TestData] = &[
        TestData {
            name: "parol",
            patterns: PATTERNS,
            states: 154,
            accepting_states: 45,
            char_classes: 50,
            min_states: 151,
            min_accepting_states: 45,
        },
        TestData {
            name: "sym_spec",
            patterns: PATTERNS_2,
            states: 107,
            accepting_states: 37,
            char_classes: 23,
            min_states: 107,
            min_accepting_states: 37,
        },
        TestData {
            name: "dragon",
            patterns: &["(a|b)*abb"],
            states: 5,
            accepting_states: 1,
            char_classes: 2,
            min_states: 4,
            min_accepting_states: 1,
        },
        TestData {
            name: "in_int",
            patterns: &["in", "int"],
            states: 4,
            accepting_states: 2,
            char_classes: 3,
            min_states: 4,
            min_accepting_states: 2,
        },
        TestData {
            name: "bounds",
            patterns: &["a{1,2}b{2,}c{3}"],
            states: 9,
            accepting_states: 1,
            char_classes: 3,
            min_states: 8,
            min_accepting_states: 1,
        },
        // "[A-Z][a-z]*([ ][A-Z][a-z]*)*[ ][A-Z][A-Z]"
        TestData {
            name: "city_and_state",
            patterns: &["[A-Z][a-z]*([ ][A-Z][a-z]*)*[ ][A-Z][A-Z]"],
            states: 7,
            accepting_states: 1,
            char_classes: 3,
            min_states: 6,
            min_accepting_states: 1,
        },
    ];

    // Initialize the logger for the tests
    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_dfa_from_nfa() {
        let mut multi_pattern_nfa = MultiPatternNfa::new();
        let result = multi_pattern_nfa.add_pattern("b|a{2,3}");
        assert!(result.is_ok());
        let result = multi_pattern_nfa.add_pattern("(a|b)*abb");
        assert!(result.is_ok());
        multi_nfa_render_to!(&multi_pattern_nfa, "input_nfa");

        let dfa = Dfa::try_from(multi_pattern_nfa).unwrap();

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

        let dfa = Dfa::try_from(multi_pattern_nfa).unwrap();

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

        let dfa = Dfa::try_from(multi_pattern_nfa).unwrap();

        dfa_render_to!(&dfa, "dfa_from_nfa_3");

        assert_eq!(dfa.states().len(), 154);
        assert_eq!(dfa.patterns().len(), 40);
        assert_eq!(dfa.accepting_states().len(), 45);
        assert_eq!(dfa.char_classes().len(), 50);
    }

    #[test]
    fn test_dfa_minimize() {
        init();

        // Iterate over the test data and run the tests.
        for data in TEST_DATA {
            let mut multi_pattern_nfa = MultiPatternNfa::new();

            let result = multi_pattern_nfa.add_patterns(data.patterns);
            if let Err(e) = result {
                panic!("Error: {}", e);
            }
            multi_nfa_render_to!(&multi_pattern_nfa, &format!("{}_nfa", data.name));

            let dfa = Dfa::try_from(multi_pattern_nfa).unwrap();
            dfa_render_to!(&dfa, &format!("{}_dfa", data.name));

            assert_eq!(dfa.states().len(), data.states, "states of {}", data.name);
            assert_eq!(
                dfa.patterns().len(),
                data.patterns.len(),
                "patterns {}",
                data.name
            );
            assert_eq!(
                dfa.accepting_states().len(),
                data.accepting_states,
                "accepting_states of {}",
                data.name
            );
            assert_eq!(
                dfa.char_classes().len(),
                data.char_classes,
                "char_classes of {}",
                data.name
            );

            let minimized_dfa = dfa.minimize().unwrap();
            dfa_render_to!(&minimized_dfa, &format!("{}_min_dfa", data.name));

            assert_eq!(
                minimized_dfa.states().len(),
                data.min_states,
                "min_states of {}",
                data.name
            );
            assert_eq!(
                minimized_dfa.patterns().len(),
                data.patterns.len(),
                "min_patterns of {}",
                data.name
            );
            assert_eq!(
                minimized_dfa.accepting_states().len(),
                data.min_accepting_states,
                "min_accepting_states of {}",
                data.name
            );
            assert_eq!(
                minimized_dfa.char_classes().len(),
                data.char_classes,
                "char_classes of {}",
                data.name
            );
        }
    }
}
