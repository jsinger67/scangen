use thiserror::Error;

/// The result type for the `scangen` crate.
pub type Result<T> = std::result::Result<T, Box<ScanGenError>>;

/// The error type for the `scangen` crate.
#[derive(Error, Debug)]
pub enum ScanGenError {
    /// An error occurred during the parsing of the regex syntax.
    #[error(transparent)]
    RegexSyntaxError(#[from] regex_syntax::ast::Error),

    /// Used regex features that are not supported.
    /// The error message contains the unsupported features.
    #[error("Unsupported regex feature: {0}")]
    UnsupportedFeature(String),
}
