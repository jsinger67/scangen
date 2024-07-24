use crate::{DfaData, ScannerModeData};

use super::{Dfa, Scanner, ScannerMode};

/// A scanner builder is used to build a scanner.
#[derive(Debug, Default)]
pub struct ScannerBuilder {
    /// The DFAs that are used to search for matches.
    pub(crate) dfas: Vec<Dfa>,
    /// The scanner modes that are used to search for matches.
    pub(crate) scanner_modes: Vec<ScannerMode>,
}

impl ScannerBuilder {
    /// Creates a new scanner builder.
    pub fn new() -> Self {
        Self {
            dfas: Vec::new(),
            scanner_modes: Vec::new(),
        }
    }

    /// Adds DFA data to the scanner builder.
    pub fn add_dfa_data(&mut self, dfa_data: &[DfaData]) {
        self.dfas = dfa_data.iter().map(|dfa| dfa.into()).collect();
    }

    /// Adds a scanner mode data to the scanner builder.
    pub fn add_scanner_mode_data(&mut self, scanner_mode_data: &[ScannerModeData]) {
        for mode in scanner_mode_data {
            let scanner_mode = ScannerMode::new(&self.dfas, mode);
            self.scanner_modes.push(scanner_mode);
        }
    }

    /// Builds the scanner from the scanner builder.
    pub fn build(self) -> Scanner {
        let mut scanner = Scanner {
            dfas: self.dfas,
            scanner_modes: self.scanner_modes,
            current_mode: 0,
        };
        if scanner.scanner_modes.is_empty() {
            Self::create_default_mode(&mut scanner);
        }
        scanner
    }

    /// Creates a default mode for the scanner.
    /// The default mode is created if no scanner modes have been added to the scanner builder.
    /// The default mode contains all DFAs and assigns incrementing token type numbers to them.
    fn create_default_mode(scanner: &mut Scanner) {
        let mut token_type = 0;
        let dfas = scanner.dfas.iter().map(|dfa| {
            let dfa = dfa.clone();
            let dfa_with_token_type = (dfa, token_type);
            token_type += 1;
            dfa_with_token_type
        });
        let default_mode = ScannerMode {
            name: "INITIAL".to_string(),
            dfas: dfas.collect(),
        };
        scanner.scanner_modes.push(default_mode);
    }
}
