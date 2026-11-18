//! Bounded observations of actual native text typeface selections.
//!
//! This data never participates in matching, layout, or glyph replay. Document
//! text and codepoints are not retained.
use std::cell::RefCell;
use std::collections::BTreeSet;

use serde::Serialize;

use crate::renderer::TextStyle;

const MAX_RECORDS: usize = 256;
const MAX_NAME_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeFontFallbackRecord {
    pub requested: String,
    pub resolved: String,
    pub source: String,
    pub bold: bool,
    pub italic: bool,
}

/// Observation scope excludes fonts selected inside embedded SVG/metafiles.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeFontFallbackReport {
    pub state: &'static str,
    pub scope: &'static str,
    pub records: Vec<NativeFontFallbackRecord>,
    pub missing_requests: Vec<String>,
    pub dropped_observations: usize,
    pub record_limit: usize,
    pub name_byte_limit: usize,
}

#[derive(Default)]
struct Observations {
    records: BTreeSet<NativeFontFallbackRecord>,
    missing: BTreeSet<String>,
    name_bytes: usize,
    dropped: usize,
}

#[derive(Default)]
pub(super) struct FontFallbackObserver(RefCell<Observations>);

impl FontFallbackObserver {
    pub(super) fn selected(
        &self,
        requested: &str,
        resolved: String,
        source: &'static str,
        style: &TextStyle,
    ) {
        let bytes = requested.len().saturating_add(resolved.len());
        if bytes > MAX_NAME_BYTES {
            let mut observations = self.0.borrow_mut();
            observations.dropped = observations.dropped.saturating_add(1);
            return;
        }
        let record = NativeFontFallbackRecord {
            requested: requested.into(),
            resolved,
            source: source.into(),
            bold: style.bold,
            italic: style.italic,
        };
        let mut observations = self.0.borrow_mut();
        if observations.records.contains(&record) {
            return;
        }
        if !observations.reserve(bytes) {
            return;
        }
        observations.records.insert(record);
    }

    pub(super) fn missing(&self, requested: &str) {
        let mut observations = self.0.borrow_mut();
        if observations.missing.contains(requested) || !observations.reserve(requested.len()) {
            return;
        }
        observations.missing.insert(requested.into());
    }

    pub(super) fn snapshot(&self) -> NativeFontFallbackReport {
        let observations = self.0.borrow();
        NativeFontFallbackReport {
            state: if observations.dropped == 0 {
                "complete"
            } else {
                "partial"
            },
            scope: "nativeTextTypefaceSelection",
            records: observations.records.iter().cloned().collect(),
            missing_requests: observations.missing.iter().cloned().collect(),
            dropped_observations: observations.dropped,
            record_limit: MAX_RECORDS,
            name_byte_limit: MAX_NAME_BYTES,
        }
    }
}

impl Observations {
    fn reserve(&mut self, bytes: usize) -> bool {
        if self.records.len().saturating_add(self.missing.len()) >= MAX_RECORDS
            || self.name_bytes.saturating_add(bytes) > MAX_NAME_BYTES
        {
            self.dropped = self.dropped.saturating_add(1);
            return false;
        }
        self.name_bytes += bytes;
        true
    }
}
