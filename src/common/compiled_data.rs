/// The data of a DFA generated as Rust code.
pub type DfaData = (
    // The pattern that this DFA recognizes.
    &'static str,
    // The states that are accepting states.
    &'static [usize],
    // The ranges of transitions in the transitions slice. The state is used as index.
    &'static [(usize, usize)],
    // The transitions of the DFA. The first usize is the state, the second usize is the char class
    // and the third usize is the target state.
    &'static [(usize, (usize, usize))],
);

/// The data of a scanner mode generated as Rust code.
pub type ScannerModeData = (
    // The name of the scanner mode.
    &'static str,
    // The DFAs of the scanner mode bundled with their associated token type numbers.
    &'static [(usize, usize)],
    // The transitions between the scanner modes triggered by a token type number.
    // The entries are tuples of the token type numbers and the new scanner mode index and are
    // sorted by token type number.
    &'static [(usize, usize)],
);
