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
    /// The script nests deeper than [`crate::doclang::eqedit::parser::MAX_EQ_DEPTH`]
    /// (adversarially deep groups / commands / fraction / script chains). Parsing
    /// is aborted before it can overflow the stack; the caller falls back to a
    /// placeholder. See the equation-DoS hardening for the rationale.
    TooDeep,
}

impl std::fmt::Display for EqError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EqError::UnbalancedBrace => write!(f, "unbalanced braces in EqEdit script"),
            EqError::TooDeep => write!(f, "EqEdit script nests too deeply"),
        }
    }
}

impl std::error::Error for EqError {}
