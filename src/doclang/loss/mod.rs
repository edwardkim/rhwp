//! Loss reporting for lean-mode conversions.
//!
//! Re-exports the primary types from [`report`] at the module root for
//! ergonomic use by the writer and adapter layers.

pub mod report;

pub use report::{LossEntry, LossKind, LossReport};
