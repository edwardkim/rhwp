//! Loss reporting for lean-mode conversions.
//!
//! When the converter runs in `Mode::Lean`, any HWP-specific information that
//! cannot be represented in the DocLang v0.6 element vocabulary is recorded
//! here instead of being silently dropped.

/// Category of information that could not be represented in DocLang output.
#[derive(Debug, Clone, PartialEq)]
pub enum LossKind {
    /// Font name / face information.
    FontInfo,
    /// Inline character colour.
    CharColor,
    /// Named paragraph or character style.
    NamedStyle,
    /// Section-level layout settings (page size, margins, columns, etc.).
    SectionSettings,
    /// Floating or anchored object that has no DocLang counterpart.
    FloatingObject,
    /// Text box (글상자) content or geometry.
    TextBox,
    /// Track-changes / revision markup.
    TrackChanges,
    /// Mathematical formula that could not be converted to LaTeX.
    FormulaFallback,
    /// Figure or table caption.
    Caption,
    /// Any other uncategorised loss.
    Other(String),
}

/// A single item of information lost during lean-mode conversion.
#[derive(Debug, Clone, PartialEq)]
pub struct LossEntry {
    /// What kind of information was lost.
    pub kind: LossKind,
    /// Human-readable location within the source document (e.g. section/paragraph index).
    pub location: String,
    /// Additional detail about the lost information.
    pub detail: String,
}

/// Aggregated loss report for a single lean-mode conversion run.
///
/// Passed back to callers inside [`crate::doclang::ConvertOutcome`] so that
/// downstream tooling can audit fidelity without inspecting the XML output.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LossReport {
    entries: Vec<LossEntry>,
}

impl LossReport {
    /// Create an empty report.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a loss entry.
    pub fn push(&mut self, entry: LossEntry) {
        self.entries.push(entry);
    }

    /// Move all entries from `other` into this report, draining `other`.
    pub fn merge(&mut self, other: LossReport) {
        self.entries.extend(other.entries);
    }

    /// Returns `true` if no losses were recorded.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Number of loss entries recorded.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Iterate over all loss entries.
    pub fn iter(&self) -> impl Iterator<Item = &LossEntry> {
        self.entries.iter()
    }
}

impl IntoIterator for LossReport {
    type Item = LossEntry;
    type IntoIter = std::vec::IntoIter<LossEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl<'a> IntoIterator for &'a LossReport {
    type Item = &'a LossEntry;
    type IntoIter = std::slice::Iter<'a, LossEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_report() {
        let r = LossReport::new();
        assert!(r.is_empty());
        assert_eq!(r.len(), 0);
    }

    #[test]
    fn push_and_iterate() {
        let mut r = LossReport::new();
        r.push(LossEntry {
            kind: LossKind::FontInfo,
            location: "section[0]/para[1]".to_string(),
            detail: "Nanum Gothic".to_string(),
        });
        r.push(LossEntry {
            kind: LossKind::Other("custom".to_string()),
            location: "section[0]/para[2]".to_string(),
            detail: "unknown field".to_string(),
        });
        assert!(!r.is_empty());
        assert_eq!(r.len(), 2);

        let kinds: Vec<_> = r.iter().map(|e| &e.kind).collect();
        assert_eq!(kinds[0], &LossKind::FontInfo);

        // IntoIterator for owned
        let count = r.into_iter().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn default_is_empty() {
        let r = LossReport::default();
        assert!(r.is_empty());
    }
}
