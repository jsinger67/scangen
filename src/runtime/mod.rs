mod dfa;
pub use dfa::Dfa;
pub(crate) use dfa::DfaWithTokenType;

mod scanner;
pub use scanner::Scanner;

mod scanner_builder;
pub use scanner_builder::{
    ScannerBuilder, ScannerBuilderWithScannerModes, ScannerBuilderWithsDfas,
    ScannerBuilderWithsDfasAndScannerModes,
};

mod scanner_mode;
pub use scanner_mode::ScannerMode;

mod find_matches;
pub use find_matches::{FindMatches, PeekResult};

#[cfg(test)]
mod generated;
