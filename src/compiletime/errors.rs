use thiserror::Error;

/// The result type for the `scangen` crate.
pub type Result<T> = std::result::Result<T, ScanGenError>;

/// The error type for the `scangen` crate.
#[derive(Error, Debug)]
pub struct ScanGenError {
    /// The source of the error.
    pub source: Box<ScanGenErrorKind>,
}

impl ScanGenError {
    /// Create a new `ScanGenError`.
    pub fn new(kind: ScanGenErrorKind) -> Self {
        ScanGenError {
            source: Box::new(kind),
        }
    }
}

impl std::fmt::Display for ScanGenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.source)
    }
}

/// The error kind type.
#[derive(Error, Debug)]
pub enum ScanGenErrorKind {
    /// An error occurred during the parsing of the regex syntax.
    #[error(transparent)]
    RegexSyntaxError(#[from] regex_syntax::ast::Error),

    /// A std::io error occurred.
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// Used regex features that are not supported (yet).
    #[error("Unsupported regex feature: {0}")]
    UnsupportedFeature(String),

    /// An error occurred during construction of the DFA.
    #[error(transparent)]
    DfaError(DfaError),
}

impl From<regex_syntax::ast::Error> for ScanGenError {
    fn from(error: regex_syntax::ast::Error) -> Self {
        ScanGenError::new(ScanGenErrorKind::RegexSyntaxError(error))
    }
}

impl From<std::io::Error> for ScanGenError {
    fn from(error: std::io::Error) -> Self {
        ScanGenError::new(ScanGenErrorKind::IoError(error))
    }
}

/// An error type for the DFA.
#[derive(Error, Debug)]
pub enum DfaError {
    /// An error occurred during the construction of the DFA.
    #[error("DFA construction error: {0}")]
    ConstructionError(String),

    /// An error occurred during the construction of a single-pattern DFA.
    #[error("Single-pattern DFA construction error: {0}")]
    SinglePatternDfaError(String),
}
