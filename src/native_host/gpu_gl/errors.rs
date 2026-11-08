//! Typed outcomes distinguish host policy/input/resource failures from device loss.

use serde::Serialize;

use crate::native_host::presentation::NativePresentationBorrowError;
use crate::native_host::NativeHostError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NativeGlFailureKind {
    InvalidArgument,
    LimitExceeded,
    DocumentResource,
    StaleFrame,
    SourceUnavailable,
    WrongOwner,
    ContextNotCurrent,
    ContextMismatch,
    ContextUnavailable,
    ContextCreationFailed,
    TargetUnavailable,
    DeviceLost,
    AllocationFailed,
    SubmissionFailed,
    BackendUnavailable,
    RecoveryRejected,
    RecoveryFailed,
    RecoveryBudgetExhausted,
}

impl NativeGlFailureKind {
    pub fn code(self) -> &'static str {
        match self {
            Self::InvalidArgument | Self::WrongOwner => "invalidArgument",
            Self::LimitExceeded => "limitExceeded",
            Self::DocumentResource => "documentError",
            Self::StaleFrame => "staleFrame",
            Self::SourceUnavailable => "sourceUnavailable",
            Self::ContextNotCurrent => "contextNotCurrent",
            Self::ContextMismatch => "contextMismatch",
            Self::ContextUnavailable => "contextUnavailable",
            Self::ContextCreationFailed => "contextCreationFailed",
            Self::TargetUnavailable => "targetUnavailable",
            Self::DeviceLost => "deviceLost",
            Self::AllocationFailed => "allocationFailed",
            Self::SubmissionFailed => "submissionFailed",
            Self::BackendUnavailable => "backendUnavailable",
            Self::RecoveryRejected => "recoveryRejected",
            Self::RecoveryFailed => "recoveryFailed",
            Self::RecoveryBudgetExhausted => "recoveryBudgetExhausted",
        }
    }

    pub fn domain(self) -> &'static str {
        match self {
            Self::DocumentResource => "document",
            Self::StaleFrame => "stale",
            Self::SourceUnavailable => "source",
            Self::InvalidArgument | Self::WrongOwner => "argument",
            Self::LimitExceeded | Self::AllocationFailed | Self::RecoveryBudgetExhausted => {
                "resource"
            }
            Self::ContextNotCurrent
            | Self::ContextMismatch
            | Self::ContextUnavailable
            | Self::ContextCreationFailed
            | Self::TargetUnavailable
            | Self::RecoveryRejected => "context",
            _ => "gpu",
        }
    }

    pub fn requires_recreate(self) -> bool {
        matches!(
            self,
            Self::ContextUnavailable
                | Self::DeviceLost
                | Self::SubmissionFailed
                | Self::RecoveryFailed
        )
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeGlFailureInfo {
    pub kind: NativeGlFailureKind,
    pub code: &'static str,
    pub domain: &'static str,
    pub message: String,
    /// True only when Skia's actual is_device_lost query reported loss.
    pub device_failure: bool,
    pub requires_recreate: bool,
    pub retryable: bool,
}

#[derive(Debug)]
pub struct NativeGlError {
    pub kind: NativeGlFailureKind,
    pub error: NativeHostError,
}

pub type NativeGlResult<T> = Result<T, NativeGlError>;

impl NativeGlError {
    pub fn new(kind: NativeGlFailureKind, message: &str) -> Self {
        Self {
            kind,
            error: NativeHostError::Unsupported(message.into()),
        }
    }

    pub fn info(&self) -> NativeGlFailureInfo {
        let mut message = self.error.to_string();
        let mut end = message.len().min(4096);
        while !message.is_char_boundary(end) {
            end -= 1;
        }
        message.truncate(end);
        NativeGlFailureInfo {
            kind: self.kind,
            code: self.kind.code(),
            domain: self.kind.domain(),
            message,
            device_failure: self.kind == NativeGlFailureKind::DeviceLost,
            requires_recreate: self.kind.requires_recreate(),
            retryable: matches!(
                self.kind,
                NativeGlFailureKind::StaleFrame | NativeGlFailureKind::ContextNotCurrent
            ),
        }
    }
}

impl From<NativeHostError> for NativeGlError {
    fn from(error: NativeHostError) -> Self {
        let kind = match &error {
            NativeHostError::InvalidArgument(_) => NativeGlFailureKind::InvalidArgument,
            NativeHostError::LimitExceeded(_) => NativeGlFailureKind::LimitExceeded,
            NativeHostError::Core(_) => NativeGlFailureKind::DocumentResource,
            NativeHostError::StaleRevision { .. } => NativeGlFailureKind::StaleFrame,
            NativeHostError::Unsupported(_) => NativeGlFailureKind::SourceUnavailable,
            NativeHostError::FeatureDisabled => NativeGlFailureKind::BackendUnavailable,
        };
        Self { kind, error }
    }
}

impl From<crate::HwpError> for NativeGlError {
    fn from(error: crate::HwpError) -> Self {
        Self::from(NativeHostError::from(error))
    }
}

impl From<NativePresentationBorrowError> for NativeGlError {
    fn from(error: NativePresentationBorrowError) -> Self {
        match error {
            NativePresentationBorrowError::Stale => Self {
                kind: NativeGlFailureKind::StaleFrame,
                error: error.into(),
            },
            NativePresentationBorrowError::InvalidHandle => Self {
                kind: NativeGlFailureKind::InvalidArgument,
                error: error.into(),
            },
            NativePresentationBorrowError::Host(error) => error.into(),
        }
    }
}

impl std::fmt::Display for NativeGlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}
impl std::error::Error for NativeGlError {}
