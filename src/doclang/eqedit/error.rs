//! Error type for EqEdit → LaTeX conversion.

/// Errors produced by [`crate::doclang::eqedit::convert`].
///
/// Conversion is permissive: unknown commands and identifiers never error.
/// The only failure mode is structurally irrecoverable input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EqError {
    /// Braces in the script are not balanced (an unmatched `{` or `}`),
    /// and recovery is not possible.
    UnbalancedBrace,
}

impl std::fmt::Display for EqError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EqError::UnbalancedBrace => write!(f, "unbalanced braces in EqEdit script"),
        }
    }
}

impl std::error::Error for EqError {}
