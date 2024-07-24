use crate::ScannerModeData;

use super::Dfa;

/// A ScannerMode is a set of active DFAs with their associated token type numbers.
/// The DFAs are clones from the Scanner's `dfas` field for the sake of performance.
/// The token type numbers are of type `usize` bundled with the DFAs.
#[derive(Debug)]
pub struct ScannerMode {
    /// The name of the mode.
    pub name: String,
    /// The DFAs and their associated token type numbers.
    pub(crate) dfas: Vec<(Dfa, usize)>,
}

impl ScannerMode {
    /// Creates a new scanner mode from the Scanner's DFAs and the ScannerModeData.
    pub fn new(dfas: &[Dfa], scanner_mode_data: &ScannerModeData) -> Self {
        let name = scanner_mode_data.0.to_string();
        let dfas = scanner_mode_data
            .1
            .iter()
            .map(|(dfa_index, token_type)| (dfas[*dfa_index].clone(), *token_type))
            .collect();
        Self { name, dfas }
    }
}
