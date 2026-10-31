//! Paragraph-block policy adapter for the shared model work visitor.
//! The owned walk separately accounts skipped vpos snapshots at each depth.
use crate::{error::HwpError, model::paragraph::Paragraph};
use serde::Serialize;

pub(super) fn measure(
    paras: &[Paragraph],
    count: usize,
    limit: usize,
    skipped: usize,
) -> Result<usize, HwpError> {
    let bytes = crate::model::owned_work::measure(paras, limit / count, skipped)
        .map_err(|error| super::invalid(error.to_string()))?;
    bytes
        .checked_mul(count)
        .ok_or_else(|| super::invalid("structure size overflow"))
}

pub(super) fn measure_value<T: Serialize + ?Sized>(
    value: &T,
    limit: usize,
) -> Result<usize, HwpError> {
    crate::model::owned_work::measure(value, limit, 0)
        .map_err(|error| super::invalid(error.to_string()))
}
