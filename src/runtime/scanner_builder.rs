use crate::{DfaData, ScannerModeData};

use super::{Dfa, DfaWithTokenType, Scanner, ScannerMode};

/// A scanner builder is used to build a scanner.
///
/// Here are the steps to build a scanner with a scanner builder:
///
/// [ScannerBuilder] -> [ScannerBuilder::add_scanner_mode_data] -> [ScannerBuilderWithScannerModes]
///
/// [ScannerBuilder] -> [ScannerBuilder::add_dfa_data] -> [ScannerBuilderWithsDfas]
///
/// [ScannerBuilderWithsDfas] -> [ScannerBuilderWithsDfas::add_scanner_mode_data] -> [ScannerBuilderWithsDfasAndScannerModes]
///
/// [ScannerBuilderWithsDfas] -> [ScannerBuilderWithsDfas::build] -> [Scanner] with default mode
///
/// [ScannerBuilderWithScannerModes] -> [ScannerBuilderWithScannerModes::add_dfa_data] -> [ScannerBuilderWithsDfasAndScannerModes]
///
/// [ScannerBuilderWithsDfasAndScannerModes] -> [ScannerBuilderWithsDfasAndScannerModes::build] -> [Scanner] with configured modes
///
/// This way it is guaranteed that the scanner is built with all necessary data.
/// The build method is the only way to build the scanner from the scanner builder.
/// It is unfailable and returns a scanner directly instead of a Result.
///
/// It is advised to use the fluent notation to build the scanner, like this:
/// ```rust
/// use scangen::{DfaData, ScannerBuilder, ScannerModeData};
/// const DFAS: &[DfaData] = &[/* ... */];
/// const MODES: &[ScannerModeData] = &[/* ... */];
/// let mut scanner = ScannerBuilder::new()
///     .add_dfa_data(DFAS)
///     .add_scanner_mode_data(MODES)
///     .build();
/// ```

///
#[derive(Debug, Default)]
pub struct ScannerBuilder {}

impl ScannerBuilder {
    /// Creates a new scanner builder.
    pub fn new() -> Self {
        Self {}
    }

    /// Adds scanner mode data to the scanner builder.
    /// Creates a ScannerBuilderWithScannerModes
    pub fn add_scanner_mode_data(
        self,
        scanner_mode_data: &[ScannerModeData],
    ) -> ScannerBuilderWithScannerModes {
        let mut scanner_modes = Vec::new();
        for mode in scanner_mode_data {
            let scanner_mode = ScannerMode::new(&[], mode);
            scanner_modes.push(scanner_mode);
        }

        ScannerBuilderWithScannerModes { scanner_modes }
    }

    /// Adds DFA data to the scanner builder.
    pub fn add_dfa_data(self, dfa_data: &[DfaData]) -> ScannerBuilderWithsDfas {
        ScannerBuilderWithsDfas {
            dfas: dfa_data.iter().map(|dfa| dfa.into()).collect(),
        }
    }

    /// Creates a default mode for the scanner.
    /// The default mode is created if no scanner modes have been added to the scanner builder.
    /// The default mode contains all DFAs and assigns incrementing token type numbers to them.
    fn create_default_mode(scanner: &mut Scanner) {
        let mut token_type = 0;
        let dfas = scanner.dfas.iter().map(|dfa| {
            let dfa = DfaWithTokenType::new(dfa.clone(), token_type);
            token_type += 1;
            dfa
        });
        let default_mode = ScannerMode {
            name: "INITIAL".to_string(),
            dfas: dfas.collect(),
            // The default mode has no transitions.
            transitions: Vec::new(),
        };
        scanner.scanner_modes.push(default_mode);
    }
}

/// A scanner builder with DFAs. Remember to always starts with [ScannerBuilder].
///
/// You can add scanner mode data to the scanner builder.
/// Also you can call the build method to build the scanner.
/// if no scanner mode data is added, a default mode is created in the build method.
pub struct ScannerBuilderWithsDfas {
    pub(crate) dfas: Vec<Dfa>,
}

impl ScannerBuilderWithsDfas {
    /// Adds scanner mode data to the scanner builder.
    pub fn add_scanner_mode_data(
        self,
        scanner_mode_data: &[ScannerModeData],
    ) -> ScannerBuilderWithsDfasAndScannerModes {
        let ScannerBuilderWithsDfas { dfas } = self;
        let mut scanner_modes = Vec::new();
        for mode in scanner_mode_data {
            let scanner_mode = ScannerMode::new(&dfas, mode);
            scanner_modes.push(scanner_mode);
        }

        ScannerBuilderWithsDfasAndScannerModes {
            dfas,
            scanner_modes,
        }
    }

    /// Builds the scanner.
    /// Builds the scanner from the scanner builder.
    pub fn build(self) -> Scanner {
        let mut scanner = Scanner {
            dfas: self.dfas,
            scanner_modes: Vec::new(),
            current_mode: 0,
        };
        ScannerBuilder::create_default_mode(&mut scanner);
        scanner
    }
}

/// A scanner builder with scanner modes. Remember to always starts with [ScannerBuilder].
///
/// You can add DFA data to the scanner builder.
/// Because the scanner needs Dfas this struct has no build method.
pub struct ScannerBuilderWithScannerModes {
    pub(crate) scanner_modes: Vec<ScannerMode>,
}

impl ScannerBuilderWithScannerModes {
    /// Adds DFA data to the scanner builder.
    pub fn add_dfa_data(self, dfa_data: &[DfaData]) -> ScannerBuilderWithsDfasAndScannerModes {
        let dfas = dfa_data.iter().map(|dfa| dfa.into()).collect();
        ScannerBuilderWithsDfasAndScannerModes {
            dfas,
            scanner_modes: self.scanner_modes,
        }
    }
}

/// A scanner builder with DFAs and scanner modes. Remember to always starts with [ScannerBuilder].
///
/// You can call the build method to build the scanner.
/// If the added scanner modes are empty, a default mode is created in the build method.
pub struct ScannerBuilderWithsDfasAndScannerModes {
    pub(crate) dfas: Vec<Dfa>,
    pub(crate) scanner_modes: Vec<ScannerMode>,
}

impl ScannerBuilderWithsDfasAndScannerModes {
    /// Builds the scanner.
    /// Builds the scanner from the scanner builder.
    pub fn build(self) -> Scanner {
        let mut scanner = Scanner {
            dfas: self.dfas,
            scanner_modes: self.scanner_modes,
            current_mode: 0,
        };
        if scanner.scanner_modes.is_empty() {
            ScannerBuilder::create_default_mode(&mut scanner);
        }
        scanner
    }
}
