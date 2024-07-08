use thiserror::Error;

/// The result type for the `scangen` crate.
pub type Result<T> = std::result::Result<T, ScanGenError>;

/// A macro that constructs a new ScanGenError::UnsupportedFeature variant.
#[macro_export]
macro_rules! unsupported {
    ($feature:expr) => {
        ScanGenError::new($crate::errors::ScanGenErrorKind::UnsupportedFeature(
            $feature.to_string(),
        ))
    };
}

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

/// The error kine type.
#[derive(Error, Debug)]
pub enum ScanGenErrorKind {
    /// An error occurred during the parsing of the regex syntax.
    #[error(transparent)]
    RegexSyntaxError(#[from] regex_syntax::ast::Error),

    /// Used regex features that are not supported (yet).
    #[error("Unsupported regex feature: {0}")]
    UnsupportedFeature(String),

    /// An error originated from the regex-automata crate.
    /// This error is used when the regex-automata crate returns an error.
    #[error(transparent)]
    RegexAutomataError(RegexAutomataError),
}

impl From<regex_automata::util::primitives::PatternIDError> for ScanGenError {
    fn from(error: regex_automata::util::primitives::PatternIDError) -> Self {
        ScanGenError::new(ScanGenErrorKind::RegexAutomataError(
            RegexAutomataError::PatternIDError(error),
        ))
    }
}

impl From<regex_automata::util::primitives::StateIDError> for ScanGenError {
    fn from(error: regex_automata::util::primitives::StateIDError) -> Self {
        ScanGenError::new(ScanGenErrorKind::RegexAutomataError(
            RegexAutomataError::StateIDError(error),
        ))
    }
}

impl From<regex_syntax::ast::Error> for ScanGenError {
    fn from(error: regex_syntax::ast::Error) -> Self {
        ScanGenError::new(ScanGenErrorKind::RegexSyntaxError(error))
    }
}

/// An error originated from the regex-automata crate.
#[derive(Error, Debug)]
pub enum RegexAutomataError {
    /// An error occurred during creation of a new pattern ID.
    #[error("Regex automata error: {0}")]
    PatternIDError(regex_automata::util::primitives::PatternIDError),

    /// An error occurred during creation of a new state ID.
    #[error("Regex automata error: {0}")]
    StateIDError(regex_automata::util::primitives::StateIDError),
}
