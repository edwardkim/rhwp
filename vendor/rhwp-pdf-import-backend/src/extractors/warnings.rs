//! Structured warning surface.
//!
//! `PdfDocument::flatten_warnings()` returns the warnings raised since
//! the document was opened, as a list of structured `Warning` records.
//! Callers who want diagnostics as data (rather than stderr text from
//! `log::warn!`) opt in to this surface. The existing `log::warn!`
//! calls continue to fire so the `setup_logging(level="WARNING")`
//! shape keeps working.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// A single structured warning raised during PDF processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warning {
    /// The category — used by callers to filter.
    pub category: WarningCategory,
    /// The page index the warning was raised on, if any. `None` means
    /// the warning is document-scoped (xref recovery, trailer parse,
    /// etc.).
    pub page: Option<usize>,
    /// Free-form message. Matches the `log::warn!` strings to
    /// preserve grep-ability for users transitioning off the stderr
    /// noise.
    pub message: String,
    /// PDF spec section the warning references, when applicable.
    /// E.g. "7.3.8.1" for the stream-keyword newline violation.
    pub spec_section: Option<&'static str>,
}

/// Coarse-grained category for filtering. Each maps to a target in
/// `log::warn!` calls — `pdf_oxide::parser`, `pdf_oxide::fonts`,
/// `pdf_oxide::content`, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum WarningCategory {
    /// PDF spec violations during xref / stream / content-stream parsing.
    /// E.g. "SPEC VIOLATION: No newline after stream keyword".
    SpecViolation,
    /// Font has no `ToUnicode` entry; falling back to AGL / CID-as-
    /// Unicode chain.
    ToUnicodeMissing,
    /// Xref table corrupt; reconstructing from `obj`/`endobj` scan.
    XrefRecovery,
    /// Content stream exceeded `MAX_OPERATORS` cap; truncating.
    OperatorCapExceeded,
    /// Type 3 font detected — may require special glyph name mapping.
    Type3Font,
    /// Unexpected EOF while reading an object header / body.
    EofPremature,
    /// Encryption / decryption related warning.
    Encryption,
    /// Other font warnings (DescendantFonts inline-dict fallback, etc.).
    Font,
    /// Layout / reading-order warnings.
    Layout,
    /// A glyph produced no rendered output while the cursor still advanced,
    /// so the page renders with an invisible gap.
    GlyphDropped,
    /// A page carries no extractable text layer and looks like a scan, so
    /// extraction returns nothing for it and OCR is what would recover it.
    ///
    /// Raised instead of writing that sentence into the extracted content:
    /// the caller decides whether to surface it, where, and in what language.
    NoTextLayer,
    /// An image was not embedded in the converted output because its encoded
    /// size exceeds the inline-image cap.
    ///
    /// Raised instead of writing an HTML comment into the markdown: the
    /// content is what the page draws, and a note about why the library
    /// declined to inline something is a diagnostic about the library.
    ImageSuppressed,
}

impl WarningCategory {
    /// Stable kebab-case string for cross-binding consumption.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SpecViolation => "spec_violation",
            Self::ToUnicodeMissing => "to_unicode_missing",
            Self::XrefRecovery => "xref_recovery",
            Self::OperatorCapExceeded => "operator_cap_exceeded",
            Self::Type3Font => "type3_font",
            Self::EofPremature => "eof_premature",
            Self::Encryption => "encryption",
            Self::Font => "font",
            Self::Layout => "layout",
            Self::GlyphDropped => "glyph_dropped",
            Self::NoTextLayer => "no_text_layer",
            Self::ImageSuppressed => "image_suppressed",
        }
    }
}

/// Thread-safe sink for warnings raised during a single `PdfDocument`
/// lifetime. Backed by a `Mutex<Vec<Warning>>` so multi-threaded usage
/// (e.g. parallel-page extraction) doesn't lose warnings to a data race.
///
/// One sink per document. The document holds it in an `Arc` so worker
/// threads can clone it.
#[derive(Debug, Default)]
pub struct WarningSink {
    warnings: Mutex<Vec<Warning>>,
}

/// global process-wide structured-warning sink for
/// the seven highest-frequency `log::warn!` sites that live in free
/// functions (where `&PdfDocument` is not available to push to a
/// per-document sink). Sites currently routed through this global
/// sink:
///
/// - `src/parser.rs::read_stream_data` (SPEC VIOLATION / Stream
///   /Length mismatch)
/// - `src/content/parser.rs::*` (operator-cap exceeded)
/// - `src/fonts/font_dict.rs::*` (Type0 ToUnicode missing, Type 3
///   font detected)
///
/// Callers retrieve via [`drain_global_warnings`] OR through
/// `PdfDocument::flatten_warnings()` which merges global +
/// per-document warnings.
///
/// The sink is **thread-local**, not process-wide.
///
/// It was process-wide, and the drain is first-caller-wins, so two documents
/// being read at the same time stole each other's warnings: whichever called
/// `structured_warnings()` first collected the other's tail and reported it
/// against the wrong file. libxml2 reached the same conclusion about its
/// global handlers and deprecated them for per-context ones.
///
/// Thread-local scope fixes the case that actually occurs — a pool reading
/// documents in parallel, one per thread. It does not fix two documents read
/// sequentially on one thread where the first never drains; that needs the
/// sink threaded into the producers, which are free functions with no
/// document in scope. The producers are listed above so that work has a
/// starting point.
///
/// Bounded, because a long-lived reader that never drains would otherwise grow
/// without limit: at the cap a single `diagnostics_truncated` entry records
/// how many were dropped rather than the vector continuing to grow.
const MAX_SINK_ENTRIES: usize = 1000;

thread_local! {
    static WARNING_SINK: std::cell::RefCell<Vec<Warning>> =
        const { std::cell::RefCell::new(Vec::new()) };
    static DROPPED: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Push a structured warning into this thread's sink. Called by
/// free-function log sites that can't access a `&PdfDocument`.
pub fn push_global_warning(warning: Warning) {
    WARNING_SINK.with(|sink| {
        let mut v = sink.borrow_mut();
        if v.len() >= MAX_SINK_ENTRIES {
            DROPPED.with(|d| d.set(d.get() + 1));
            return;
        }
        // Repeats are common — one malformed font warns once per glyph — and a
        // consumer wants to know it happened, not to read it a thousand times.
        if let Some(last) = v.iter_mut().rev().take(16).find(|w| {
            w.category == warning.category && w.page == warning.page && w.message == warning.message
        }) {
            let _ = last;
            return;
        }
        v.push(warning);
    });
}

/// Drain the process-wide structured-warning sink, returning a snapshot
/// and clearing the underlying storage. Used by
/// `PdfDocument::flatten_warnings` to surface free-function warnings
/// alongside per-document ones.
pub fn drain_global_warnings() -> Vec<Warning> {
    let mut out = WARNING_SINK.with(|sink| std::mem::take(&mut *sink.borrow_mut()));
    let dropped = DROPPED.with(|d| d.replace(0));
    if dropped > 0 {
        out.push(Warning {
            category: WarningCategory::SpecViolation,
            page: None,
            message: format!(
                "{dropped} further diagnostics were dropped after the {MAX_SINK_ENTRIES}-entry cap"
            ),
            spec_section: None,
        });
    }
    out
}

/// Snapshot this thread's sink without draining (for tests / observability).
pub fn snapshot_global_warnings() -> Vec<Warning> {
    WARNING_SINK.with(|sink| sink.borrow().clone())
}

/// Put warnings back at the front of this thread's sink.
///
/// Used to restore diagnostics belonging to a document that is mid-flight when
/// another document borrows the thread. Order is preserved so a later drain
/// sees them as they were raised.
pub(crate) fn restore_global_warnings(mut warnings: Vec<Warning>) {
    if warnings.is_empty() {
        return;
    }
    WARNING_SINK.with(|sink| {
        let mut v = sink.borrow_mut();
        warnings.append(&mut v);
        *v = warnings;
    });
}

impl WarningSink {
    /// Create an empty sink.
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a new warning. Inexpensive — no `log` macro fired here; the
    /// existing `log::warn!` sites continue to fire on their own. Use
    /// `push_with_log` from the migrated call sites to emit both.
    pub fn push(&self, warning: Warning) {
        if let Ok(mut v) = self.warnings.lock() {
            v.push(warning);
        }
        // If the mutex was poisoned, silently drop — better than panic.
    }

    /// Snapshot of all warnings raised so far. Returns owned clones so
    /// the caller can keep them past the document's lifetime.
    pub fn snapshot(&self) -> Vec<Warning> {
        self.warnings.lock().map(|v| v.clone()).unwrap_or_default()
    }

    /// Total warning count.
    pub fn len(&self) -> usize {
        self.warnings.lock().map(|v| v.len()).unwrap_or(0)
    }

    /// True if no warnings have been raised.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all warnings. Used by `PdfDocument::reset_warnings()` for
    /// callers who want to track per-operation warnings.
    pub fn clear(&self) {
        if let Ok(mut v) = self.warnings.lock() {
            v.clear();
        }
    }

    /// Push multiple warnings at once. Used by callers that merge a
    /// drained external sink (e.g. the process-wide global sink) into
    /// the per-document sink under a single lock acquisition.
    pub fn extend(&self, warnings: impl IntoIterator<Item = Warning>) {
        if let Ok(mut v) = self.warnings.lock() {
            v.extend(warnings);
        }
    }

    /// Drain and return all accumulated warnings.
    pub fn take(&self) -> Vec<Warning> {
        if let Ok(mut v) = self.warnings.lock() {
            std::mem::take(&mut *v)
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sink_starts_empty() {
        let sink = WarningSink::new();
        assert!(sink.is_empty());
        assert_eq!(sink.len(), 0);
        assert_eq!(sink.snapshot().len(), 0);
    }

    #[test]
    fn push_and_snapshot() {
        let sink = WarningSink::new();
        sink.push(Warning {
            category: WarningCategory::ToUnicodeMissing,
            page: Some(0),
            message: "Type0 font 'X' has no ToUnicode entry!".into(),
            spec_section: Some("9.10.2"),
        });
        assert_eq!(sink.len(), 1);
        let snap = sink.snapshot();
        assert_eq!(snap[0].category, WarningCategory::ToUnicodeMissing);
        assert_eq!(snap[0].page, Some(0));
        assert!(snap[0].message.contains("ToUnicode"));
    }

    #[test]
    fn category_as_str_stable() {
        assert_eq!(WarningCategory::SpecViolation.as_str(), "spec_violation");
        assert_eq!(WarningCategory::ToUnicodeMissing.as_str(), "to_unicode_missing");
        assert_eq!(WarningCategory::OperatorCapExceeded.as_str(), "operator_cap_exceeded");
    }

    #[test]
    fn clear_resets() {
        let sink = WarningSink::new();
        sink.push(Warning {
            category: WarningCategory::SpecViolation,
            page: None,
            message: "x".into(),
            spec_section: None,
        });
        assert_eq!(sink.len(), 1);
        sink.clear();
        assert!(sink.is_empty());
    }

    #[test]
    fn warning_serializes_to_json() {
        let w = Warning {
            category: WarningCategory::SpecViolation,
            page: Some(0),
            message: "No newline after stream keyword".into(),
            spec_section: Some("7.3.8.1"),
        };
        let json = serde_json::to_string(&w).unwrap();
        assert!(json.contains("\"category\":\"spec_violation\""));
        assert!(json.contains("\"page\":0"));
        assert!(json.contains("\"spec_section\":\"7.3.8.1\""));
    }

    #[test]
    fn sink_thread_safe() {
        use std::sync::Arc;
        use std::thread;

        let sink = Arc::new(WarningSink::new());
        let mut handles = Vec::new();
        for i in 0..10 {
            let s = sink.clone();
            handles.push(thread::spawn(move || {
                s.push(Warning {
                    category: WarningCategory::Font,
                    page: Some(i),
                    message: format!("font warning {}", i),
                    spec_section: None,
                });
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(sink.len(), 10);
    }
}

#[cfg(test)]
mod sink_scope_tests {
    use super::*;

    fn w(msg: &str) -> Warning {
        Warning {
            category: WarningCategory::SpecViolation,
            page: None,
            message: msg.to_string(),
            spec_section: None,
        }
    }

    /// Two readers running at once must not collect each other's warnings.
    ///
    /// The sink was process-wide and the drain is first-caller-wins, so
    /// whichever document asked first took the other's tail and reported it
    /// against the wrong file.
    #[test]
    fn one_thread_does_not_drain_anothers_warnings() {
        let _ = drain_global_warnings();
        push_global_warning(w("belongs to the main thread"));

        let other = std::thread::spawn(|| {
            push_global_warning(w("belongs to the spawned thread"));
            drain_global_warnings()
        })
        .join()
        .expect("thread");

        assert_eq!(other.len(), 1, "the other thread saw {other:?}");
        assert_eq!(other[0].message, "belongs to the spawned thread");

        let mine = drain_global_warnings();
        assert_eq!(mine.len(), 1, "this thread saw {mine:?}");
        assert_eq!(mine[0].message, "belongs to the main thread");
    }

    /// A repeated warning is recorded once, not once per occurrence.
    #[test]
    fn test_identical_warning_is_not_recorded_repeatedly() {
        let _ = drain_global_warnings();
        for _ in 0..50 {
            push_global_warning(w("one malformed font, warned per glyph"));
        }
        assert_eq!(drain_global_warnings().len(), 1);
    }

    /// A reader that never drains does not grow without bound, and is told
    /// how many were dropped rather than silently losing them.
    #[test]
    fn test_sink_is_bounded_and_reports_what_it_dropped() {
        let _ = drain_global_warnings();
        for i in 0..MAX_SINK_ENTRIES + 25 {
            push_global_warning(w(&format!("distinct {i}")));
        }
        let out = drain_global_warnings();
        assert_eq!(out.len(), MAX_SINK_ENTRIES + 1, "capped, plus one sentinel");
        assert!(
            out.last()
                .expect("sentinel")
                .message
                .contains("were dropped"),
            "the drop must be reported, not silent: {:?}",
            out.last()
        );
    }
}
