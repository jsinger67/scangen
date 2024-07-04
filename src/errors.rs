use thiserror::Error;

/// The result type for the `scangen` crate.
pub type Result<T> = std::result::Result<T, Box<ScanGenError>>;

/// A macro that constructs a new ScanGenError::UnsupportedFeature variant.
#[macro_export]
macro_rules! unsupported {
    ($feature:expr) => {
        Box::new(ScanGenError::UnsupportedFeature($feature.to_string()))
    };
}

/// The error type for the `scangen` crate.
#[derive(Error, Debug)]
pub enum ScanGenError {
    /// An error occurred during the parsing of the regex syntax.
    #[error(transparent)]
    RegexSyntaxError(#[from] regex_syntax::ast::Error),

    /// Used regex features that are not supported (yet).
    #[error("Unsupported regex feature: {0}")]
    UnsupportedFeature(String),
}
