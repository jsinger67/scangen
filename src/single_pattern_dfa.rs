use itertools::Itertools;
use std::collections::{BTreeMap, BTreeSet};

use log::trace;
use regex_automata::util::primitives::StateID;

use crate::{
    character_class::CharacterClass,
    dfa::{DfaState, Partition, StateGroup, TransitionsToPartitionGroups},
    errors::DfaError,
    MultiPatternNfa, Result, ScanGenError, ScanGenErrorKind,
};

/// A DFA type that can be used to match a single pattern.
#[derive(Debug, Clone, Default)]
pub struct SinglePatternDfa {
    // The states of the DFA. The start state is always the first state in the vector, i.e. state 0.
    states: Vec<DfaState>,
    // The pattern this DFA can match.
    pattern: String,
    // The accepting states of the DFA.
    accepting_states: BTreeSet<StateID>,
    // The character classes used in the DFA.
    char_classes: Vec<CharacterClass>,
    // The transitions of the DFA.
    transitions: BTreeMap<StateID, BTreeMap<CharacterClass, StateID>>,
}

impl SinglePatternDfa {
    /// Create a new DFA with the given pattern.
    pub fn new(pattern: String) -> Self {
        Self {
            states: Vec::new(),
            pattern,
            accepting_states: BTreeSet::new(),
            char_classes: Vec::new(),
            transitions: BTreeMap::new(),
        }
    }

    /// Add a state to the DFA.
    pub fn add_state(&mut self, state: DfaState) {
        self.states.push(state);
    }

    /// Add an accepting state to the DFA.
    pub fn add_accepting_state(&mut self, state_id: StateID) {
        self.accepting_states.insert(state_id);
    }

    /// Add a character class to the DFA.
    pub fn add_char_class(&mut self, char_class: CharacterClass) {
        self.char_classes.push(char_class);
    }

    /// Add a transition to the DFA.
    pub fn add_transition(&mut self, from: StateID, on: CharacterClass, to: StateID) {
        self.transitions.entry(from).or_default().insert(on, to);
    }

    /// Get the pattern this DFA can match.
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Get the states of the DFA.
    pub fn states(&self) -> &[DfaState] {
        &self.states
    }

    /// Get the accepting states of the DFA.
    pub fn accepting_states(&self) -> &BTreeSet<StateID> {
        &self.accepting_states
    }

    /// Get the character classes used in the DFA.
    pub fn char_classes(&self) -> &[CharacterClass] {
        &self.char_classes
    }

    /// Get the transitions of the DFA.
    pub fn transitions(&self) -> &BTreeMap<StateID, BTreeMap<CharacterClass, StateID>> {
        &self.transitions
    }

    /// Add a state to the DFA if it does not already exist.
    /// The state is identified by the NFA states that constitute the DFA state.
    /// The accepting states are used to determine if the DFA state is an accepting state.
    fn add_state_if_new<I>(
        &mut self,
        nfa_states: I,
        accepting_states: &[StateID],
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
            .position(|state| state.nfa_states() == nfa_states)
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
                .nfa_states()
                .iter()
                .filter(|nfa_state_id| accepting_states.contains(nfa_state_id))
                .count()
                <= 1
        );

        // Check if the state contains an accepting state.
        for nfa_state_id in state.nfa_states() {
            if accepting_states.contains(nfa_state_id) {
                // The state is an accepting state.
                self.add_accepting_state(state_id);
                break;
            }
        }

        self.states.push(state);
        Ok(state_id)
    }

    /// Create a DFA from a multi-pattern NFA.
    /// The DFA is created using the subset construction algorithm.
    /// The multi-pattern NFA must only contain a single pattern.
    pub fn try_from_multi_pattern_nfa(nfa: MultiPatternNfa) -> Result<Self> {
        let MultiPatternNfa {
            nfa,
            patterns,
            accepting_states,
            char_classes,
        } = nfa;
        if patterns.len() != 1 {
            return Err(ScanGenError::new(ScanGenErrorKind::DfaError(
                DfaError::SinglePatternDfaError(
                    "Only single pattern NFAs are supported".to_string(),
                ),
            )));
        }
        let accepting_states: Vec<StateID> = accepting_states.into_iter().map(|a| a.0).collect();
        let mut dfa = SinglePatternDfa::new(patterns[0].clone());
        dfa.char_classes = char_classes;
        // The initial state of the DFA is the epsilon closure of the start state of the NFA.
        let start_state = nfa.epsilon_closure(StateID::default());
        // The initial state is the start state of the DFA.
        let initial_state = dfa.add_state_if_new(start_state, &accepting_states)?;
        // The work list is used to keep track of the states that need to be processed.
        let mut work_list = vec![initial_state];
        // The marked flag is used to mark a state as visited during the subset construction algorithm.
        dfa.states[initial_state].set_marked(true);

        while let Some(state_id) = work_list.pop() {
            let nfa_states: Vec<StateID> = dfa.states[state_id].nfa_states().to_vec();
            for char_class in dfa.char_classes.clone() {
                let target_states =
                    nfa.epsilon_closure_set(nfa.move_set(&nfa_states, char_class.id()));
                if !target_states.is_empty() {
                    let target_state = dfa.add_state_if_new(target_states, &accepting_states)?;
                    dfa.transitions
                        .entry(state_id)
                        .or_default()
                        .insert(char_class.clone(), target_state);
                    if !dfa.states[target_state].marked() {
                        dfa.states[target_state].set_marked(true);
                        work_list.push(target_state);
                    }
                }
            }
        }

        Ok(dfa)
    }

    /// Add a representative state to the DFA.
    /// The representative state is the first state in the group.
    /// The accepting states are used to determine if the DFA state is an accepting state.
    /// The new state id is returned.
    fn add_representive_state(
        &mut self,
        group: &BTreeSet<StateID>,
        accepting_states: &BTreeSet<StateID>,
    ) -> Result<StateID> {
        let state_id = StateID::new(self.states.len())?;
        let state = DfaState::new(state_id, Vec::new());

        // First state in group is the representative state.
        let _representative_state_id = group.first().unwrap();

        // Insert the representative state into the accepting states if any state in its group is
        // an accepting state.
        for state_in_group in group.iter() {
            if accepting_states.contains(state_in_group) {
                self.add_accepting_state(state_id);
            }
        }

        self.states.push(state);
        Ok(state_id)
    }

    /// Minimize the DFA.
    /// The Nfa states are removed from the DFA states during minimization. They are not needed
    /// anymore after the DFA is created.
    pub fn minimize(&self) -> Result<Self> {
        let mut partition_old = self.calculate_initial_partition();
        let mut partition_new = Partition::new();
        let mut changed = true;

        while changed {
            partition_new = self.calculate_new_partition(&partition_old);
            changed = partition_new != partition_old;
            partition_old.clone_from(&partition_new);
        }

        self.create_from_partition(&partition_new)
    }

    /// The start partition is created as follows:
    /// 1. The accepting states are put in a partition with id 0.
    ///    This follows from the constraint of the DFA that only one pattern can match.
    /// 2. The non-accepting states are put together in a partition with id 1.
    ///
    /// The partitions are stored in a vector of vectors.
    ///
    /// The key building function for the Itertools::chunk_by method is used to create the
    /// partitions. For accepting states the key is the state id, for non-accepting states
    /// the key is the state id of the first non-accepting state.
    fn calculate_initial_partition(&self) -> Partition {
        let group_id_non_accepting_states: StateID = StateID::new_unchecked(1);
        self.states
            .clone()
            .into_iter()
            .chunk_by(|state| {
                if self.accepting_states.contains(&state.id()) {
                    StateID::new_unchecked(0)
                } else {
                    group_id_non_accepting_states
                }
            })
            .into_iter()
            .fold(Partition::new(), |mut partitions, (_key, group)| {
                let state_group = group.into_iter().fold(StateGroup::new(), |mut acc, state| {
                    acc.insert(state.id());
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
        for group in partition {
            // The new group receives the states from the old group which are distiguishable from
            // the other states in group.
            self.split_group(group, partition)
                .into_iter()
                .for_each(|new_group| {
                    new_partition.push(new_group);
                });
        }
        new_partition
    }

    fn split_group(&self, group: &StateGroup, partition: &[StateGroup]) -> Partition {
        // If the group contains only one state, the group can't be split further.
        if group.len() == 1 {
            return vec![group.clone()];
        }
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
            transitions_to_partition_groups
        } else {
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
    fn create_from_partition(&self, partition: &[StateGroup]) -> Result<Self> {
        let mut dfa = SinglePatternDfa::new(self.pattern.clone());
        dfa.char_classes.clone_from(&self.char_classes);
        dfa.transitions = self.transitions.clone();

        for group in partition {
            // For each group we add a representative state to the DFA.
            // It's id is the index of the group in the partition.
            // This function also updates the accepting states of the DFA.
            dfa.add_representive_state(group, &self.accepting_states)?;
        }

        // Then renumber the states in the transitions.
        dfa.update_transitions(partition);

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

impl TryFrom<MultiPatternNfa> for SinglePatternDfa {
    type Error = crate::errors::ScanGenError;

    fn try_from(nfa: MultiPatternNfa) -> Result<Self> {
        SinglePatternDfa::try_from_multi_pattern_nfa(nfa)
    }
}

impl std::fmt::Display for SinglePatternDfa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "SinglePatternDfa:")?;
        writeln!(f, "Pattern: {}", self.pattern)?;
        writeln!(f, "States:")?;
        for state in &self.states {
            writeln!(f, "{}", state.id().as_usize())?;
        }
        writeln!(f, "Accepting states: {:?}", self.accepting_states)?;
        writeln!(f, "Char classes:")?;
        for char_class in &self.char_classes {
            writeln!(f, "{}", char_class.ast.0)?;
        }
        writeln!(f, "Transitions:")?;
        for (from, transitions) in &self.transitions {
            for (on, to) in transitions {
                writeln!(
                    f,
                    "{} -{}-> {}",
                    from.as_usize(),
                    self.char_classes[on.id().as_usize()].ast.0,
                    to.as_usize()
                )?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{multi_nfa_render_to, single_dfa_render_to};

    use super::*;
    // A data type that provides test data for the DFA minimization tests.
    struct TestData {
        name: &'static str,
        pattern: &'static str,
        states: usize,
        accepting_states: usize,
        char_classes: usize,
        min_states: usize,
        min_accepting_states: usize,
    }

    // Test data for the DFA minimization tests.
    const TEST_DATA: &[TestData] = &[
        TestData {
            name: "dragon",
            pattern: "(a|b)*abb",
            states: 5,
            accepting_states: 1,
            char_classes: 2,
            min_states: 4,
            min_accepting_states: 1,
        },
        TestData {
            name: "int",
            pattern: "int",
            states: 4,
            accepting_states: 1,
            char_classes: 3,
            min_states: 4,
            min_accepting_states: 1,
        },
        TestData {
            name: "bounds",
            pattern: "a{1,2}b{2,}c{3}",
            states: 9,
            accepting_states: 1,
            char_classes: 3,
            min_states: 8,
            min_accepting_states: 1,
        },
        // "[A-Z][a-z]*([ ][A-Z][a-z]*)*[ ][A-Z][A-Z]"
        TestData {
            name: "city_and_state",
            pattern: "[A-Z][a-z]*([ ][A-Z][a-z]*)*[ ][A-Z][A-Z]",
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
    fn test_single_dfa_minimize() {
        init();

        // Iterate over the test data and run the tests.
        for data in TEST_DATA {
            let mut multi_pattern_nfa = MultiPatternNfa::new();

            let result = multi_pattern_nfa.add_pattern(data.pattern);
            if let Err(e) = result {
                panic!("Error: {}", e);
            }
            multi_nfa_render_to!(&multi_pattern_nfa, &format!("{}_nfa", data.name));

            let dfa = SinglePatternDfa::try_from(multi_pattern_nfa).unwrap();
            single_dfa_render_to!(&dfa, &format!("{}_single_dfa", data.name));

            assert_eq!(dfa.states().len(), data.states, "states of {}", data.name);
            assert_eq!(dfa.pattern, data.pattern, "patterns {}", data.name);
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
            single_dfa_render_to!(&minimized_dfa, &format!("{}_min_single_dfa", data.name));

            assert_eq!(
                minimized_dfa.states().len(),
                data.min_states,
                "min_states of {}",
                data.name
            );
            assert_eq!(
                minimized_dfa.pattern, data.pattern,
                "min_pattern of {}",
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
