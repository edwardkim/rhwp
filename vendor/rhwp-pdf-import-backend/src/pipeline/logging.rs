//! Logging utilities for the text extraction pipeline.
//!
//! These macros forward to the `log` crate so the configured log level
//! (via `log::set_max_level`, `env_logger`, `pyo3_log`, etc.) is honored.
//! When the `logging` feature is disabled, all logging is compiled out.
//!
//! Each macro accepts two forms for backward compatibility:
//! 1. A single expression: `extract_log_info!(my_string_var)` — logged as `{}`.
//! 2. A format string with optional args: `extract_log_info!("page {}", n)`.

/// Log an INFO level message. Forwards to `log::info!`.
#[macro_export]
macro_rules! extract_log_info {
    ($fmt:literal $(, $($arg:tt)*)?) => {
        #[cfg(feature = "logging")]
        ::log::info!($fmt $(, $($arg)*)?);
    };
    ($msg:expr) => {
        #[cfg(feature = "logging")]
        ::log::info!("{}", $msg);
    };
}

/// Log a WARN level message. Forwards to `log::warn!`.
#[macro_export]
macro_rules! extract_log_warn {
    ($fmt:literal $(, $($arg:tt)*)?) => {
        #[cfg(feature = "logging")]
        ::log::warn!($fmt $(, $($arg)*)?);
    };
    ($msg:expr) => {
        #[cfg(feature = "logging")]
        ::log::warn!("{}", $msg);
    };
}

/// Log a DEBUG level message. Forwards to `log::debug!`.
#[macro_export]
macro_rules! extract_log_debug {
    ($fmt:literal $(, $($arg:tt)*)?) => {
        #[cfg(feature = "logging")]
        ::log::debug!($fmt $(, $($arg)*)?);
    };
    ($msg:expr) => {
        #[cfg(feature = "logging")]
        ::log::debug!("{}", $msg);
    };
}

/// Log a TRACE level message. Forwards to `log::trace!`.
#[macro_export]
macro_rules! extract_log_trace {
    ($fmt:literal $(, $($arg:tt)*)?) => {
        #[cfg(feature = "logging")]
        ::log::trace!($fmt $(, $($arg)*)?);
    };
    ($msg:expr) => {
        #[cfg(feature = "logging")]
        ::log::trace!("{}", $msg);
    };
}

/// Log an ERROR level message. Forwards to `log::error!`.
#[macro_export]
macro_rules! extract_log_error {
    ($fmt:literal $(, $($arg:tt)*)?) => {
        #[cfg(feature = "logging")]
        ::log::error!($fmt $(, $($arg)*)?);
    };
    ($msg:expr) => {
        #[cfg(feature = "logging")]
        ::log::error!("{}", $msg);
    };
}

// Re-export the macros for convenience
pub use extract_log_debug;
pub use extract_log_error;
pub use extract_log_info;
pub use extract_log_trace;
pub use extract_log_warn;
