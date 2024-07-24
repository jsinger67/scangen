/// Module that provides data types for the generated code
mod compiled_data;
pub use compiled_data::{DfaData, ScannerModeData};

/// Module that provides a Match type
mod match_type;
pub use match_type::Match;

/// Module that provides a Span type
mod span;
pub use span::Span;

/// Module that provides types related to matching state
mod matching_state;
pub(crate) use matching_state::MatchingState;
