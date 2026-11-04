//! SVG syntax backing with an optional finite output/request boundary.
use std::{fmt, ops::Deref, sync::Arc};

#[derive(Default)]
pub(super) struct SvgBuffer {
    text: String,
    limit: Option<usize>,
    reservation: Option<Arc<dyn crate::storage::Reservation>>,
    failure: Option<String>,
}

impl SvgBuffer {
    pub(super) fn bounded(
        &mut self,
        maximum: usize,
        reservation: Arc<dyn crate::storage::Reservation>,
    ) {
        self.limit = Some(maximum);
        self.reservation = Some(reservation);
    }

    fn reserve(&mut self, additional: usize) -> bool {
        if self.failure.is_some() {
            return false;
        }
        let Some(limit) = self.limit else {
            return true;
        };
        let required = self.text.len().checked_add(additional);
        let Some(required) = required.filter(|n| *n <= limit) else {
            self.failure = Some(self.reservation.as_ref().map_or_else(
                || "PDF_RESOURCE_LIMIT: SVG output byte limit".into(),
                |reservation| reservation.reject("SVG output byte limit"),
            ));
            return false;
        };
        if required <= self.text.capacity() {
            return true;
        }
        let capacity = required
            .max(self.text.capacity().saturating_mul(2))
            .max(64)
            .min(limit);
        if let Some(reservation) = &self.reservation {
            if let Err(error) = reservation.hash_payload(1, capacity) {
                self.failure = Some(error);
                return false;
            }
        }
        if self
            .text
            .try_reserve_exact(capacity - self.text.len())
            .is_err()
        {
            self.failure = Some("PDF_WORKER_MEMORY: SVG output allocation failed".into());
            return false;
        }
        true
    }

    pub(super) fn push_str(&mut self, value: &str) {
        if self.reserve(value.len()) {
            self.text.push_str(value);
        }
    }
    pub(super) fn insert_str(&mut self, index: usize, value: &str) {
        if self.reserve(value.len()) {
            self.text.insert_str(index, value);
        }
    }
    pub(super) fn clear(&mut self) {
        self.text.clear();
    }
    pub(super) fn check(&self) -> Result<(), crate::error::HwpError> {
        match &self.failure {
            Some(error) => Err(crate::error::HwpError::RenderError(error.clone())),
            None => Ok(()),
        }
    }
    pub(super) fn finish(self) -> Result<String, crate::error::HwpError> {
        self.check()?;
        Ok(self.text)
    }
}

impl Deref for SvgBuffer {
    type Target = str;
    fn deref(&self) -> &str {
        &self.text
    }
}
impl fmt::Write for SvgBuffer {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        self.push_str(value);
        if self.failure.is_some() {
            Err(fmt::Error)
        } else {
            Ok(())
        }
    }
}
