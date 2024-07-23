use super::Dfa;

/// A ScannerMode is a set of active DFAs with their associated token type numbers.
/// The DFAs are clones from the Scanner's `dfas` field.
/// The token type numbers are of type `usize` bundled with the DFAs.
#[derive(Debug)]
pub struct ScannerMode {
    /// The name of the mode.
    pub name: String,
    /// The DFAs and their associated token type numbers.
    pub dfas: Vec<(Dfa, usize)>,
}
