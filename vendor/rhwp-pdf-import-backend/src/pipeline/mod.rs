//! PDF text extraction pipeline with clean abstraction layers.
//!
//! This module provides a PDF-spec-compliant pipeline for text extraction:
//!
//! ```text
//! PDF File
//!     ↓
//! [TextExtractor] (content stream → TextSpan[])
//!     ↓
//! TextSpan[] (single intermediate representation)
//!     ↓
//! [ReadingOrderStrategy] (pluggable ordering)
//!     ↓
//! OrderedTextSpan[]
//!     ↓
//! [OutputConverter] (Markdown/HTML/Text)
//!     ↓
//! Output String
//! ```
//!
//! # Key Design Principles
//!
//! 1. **Single Intermediate Representation**: TextSpan is the only representation
//!    between PDF parsing and output conversion.
//!
//! 2. **PDF Spec Compliance**: Per ISO 32000-1:2008, text strings from Tj/TJ
//!    operators are preserved as-is. No linguistic heuristics for word segmentation.
//!
//! 3. **Pluggable Strategies**: Reading order and output conversion are trait-based
//!    for extensibility.
//!
//! 4. **Unified Configuration**: All settings in TextPipelineConfig.

pub mod config;
pub mod converters;
pub mod logging;
pub mod metrics;
// pub mod input_parsers;  // Keep disabled - for PDF creation feature later
pub mod ordered_span;
pub mod page_order;
pub mod reading_order;
pub mod text_processing;

// Re-export main types
pub use config::{
    BoldMarkerBehavior, LogLevel, OutputConfig, ReadingOrderConfig, ReadingOrderStrategyType,
    SpacingConfig, TextPipelineConfig, TjThresholdConfig, WordBoundaryMode,
};
pub use converters::{
    HtmlOutputConverter, MarkdownOutputConverter, OutputConverter, PlainTextConverter,
};
pub use logging::{
    extract_log_debug, extract_log_error, extract_log_info, extract_log_trace, extract_log_warn,
};
pub use metrics::{BatchMetrics, ExtractionMetrics};
pub use ordered_span::{
    OrderedSpans, OrderedTextSpan, ReadingOrderInfo, ReadingOrderSource, StructRole,
};
pub use page_order::{page_reading_order, page_reading_order_no_artifacts};
pub use reading_order::{
    ReadingOrderContext, ReadingOrderStrategy, TategakiStrategy, XYCutStrategy,
};
pub use text_processing::WhitespaceNormalizer;

use crate::error::Result;
use crate::layout::TextSpan;
use reading_order::create_strategy;

/// The text extraction pipeline - orchestrates the full flow.
///
/// This is the main entry point for the new pipeline architecture.
/// It processes TextSpans through reading order determination and
/// prepares them for output conversion.
pub struct TextPipeline {
    config: TextPipelineConfig,
    reading_order_strategy: Box<dyn ReadingOrderStrategy>,
}

impl TextPipeline {
    /// Create a new pipeline with default configuration.
    pub fn new() -> Self {
        Self::with_config(TextPipelineConfig::default())
    }

    /// Create a pipeline with custom configuration.
    pub fn with_config(config: TextPipelineConfig) -> Self {
        let strategy = create_strategy(&config.reading_order);
        Self {
            config,
            reading_order_strategy: strategy,
        }
    }

    /// Process spans through the pipeline.
    ///
    /// 1. Detect vertical-majority (tategaki) pages from the per-span
    ///    `wmode` tag and route them through [`TategakiStrategy`]. The
    ///    four horizontal LTR strategies (Simple, Geometric, XYCut,
    ///    StructureTree) are unchanged; tategaki always wins when the
    ///    page is vertical because none of the LTR strategies can
    ///    produce right-to-left column ordering correctly.
    /// 2. Otherwise apply the configured reading-order strategy.
    /// 3. Reverse RTL word runs (#557b) and return ordered spans.
    pub fn process(
        &self,
        spans: Vec<TextSpan>,
        context: ReadingOrderContext,
    ) -> Result<Vec<OrderedTextSpan>> {
        let mut ordered = if context.preserve_input_order && context.mcid_order.is_none() {
            // The caller already established a non-geometric reading order the
            // strategies cannot reproduce (two-column-prose column-major, #734):
            // wrap the spans in their current order without re-sorting, so the
            // geometric XY-cut fallback does not re-derive a row-major order and
            // interleave the columns.
            log::trace!(
                "TextPipeline: preserving caller-supplied order for {} spans \
                 (reading-order strategy skipped).",
                spans.len()
            );
            spans
                .into_iter()
                .enumerate()
                .map(|(order, span)| {
                    OrderedTextSpan::with_info(span, order, ReadingOrderInfo::simple())
                })
                .collect()
        } else if is_vertical_majority(&spans) {
            log::trace!(
                "TextPipeline: vertical-majority page detected ({}/{} spans wmode=1) — \
                 routing through TategakiStrategy regardless of configured strategy.",
                spans.iter().filter(|s| s.wmode == 1).count(),
                spans.len()
            );
            reading_order::TategakiStrategy.apply(spans, &context)?
        } else {
            self.reading_order_strategy.apply(spans, &context)?
        };
        reorder_rtl_word_runs(&mut ordered);
        Ok(ordered)
    }

    /// Get the current configuration.
    pub fn config(&self) -> &TextPipelineConfig {
        &self.config
    }
}

impl Default for TextPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Vertical-majority detection: at least half the spans carry
/// `wmode == 1`. Mixed-mode pages with a horizontal majority keep their
/// configured strategy; the rare pure-vertical mixed-page case stays
/// governed by the dominant mode here. Per-span wmode is preserved on
/// every span either way, so downstream consumers (export, search) can
/// still distinguish them.
fn is_vertical_majority(spans: &[TextSpan]) -> bool {
    if spans.is_empty() {
        return false;
    }
    let vertical_count = spans.iter().filter(|s| s.wmode == 1).count();
    vertical_count * 2 >= spans.len()
}

/// #557b: right-to-left scripts read their words in the opposite direction
/// from the left-to-right reading order the strategies assign. For each
/// maximal run of consecutive same-line spans that is purely RTL (every
/// non-space span holds RTL letters and no Latin letters), reverse the run's
/// order so md/html/plain-text emit logical word order — matching the
/// `extract_text` path's `reverse_rtl_visual_order_runs`. Each word's
/// characters are left untouched (they are already logical).
fn reorder_rtl_word_runs(ordered: &mut [OrderedTextSpan]) {
    use crate::text::rtl_detector::is_rtl_text;
    ordered.sort_by_key(|o| o.reading_order);

    let is_space = |o: &OrderedTextSpan| o.span.text.trim().is_empty();
    let is_rtl_word = |o: &OrderedTextSpan| {
        let mut has_rtl = false;
        for c in o.span.text.chars() {
            if c.is_ascii_alphabetic() {
                return false; // Latin letter → not a pure-RTL word
            }
            if is_rtl_text(c as u32) {
                has_rtl = true;
            }
        }
        has_rtl
    };

    let mut i = 0;
    let mut changed = false;
    while i < ordered.len() {
        if !is_rtl_word(&ordered[i]) {
            i += 1;
            continue;
        }
        let y = ordered[i].span.bbox.y;
        let start = i;
        let mut end = i + 1;
        while end < ordered.len()
            && (ordered[end].span.bbox.y - y).abs() < 2.0
            && (is_rtl_word(&ordered[end]) || is_space(&ordered[end]))
        {
            end += 1;
        }
        // Keep trailing space spans as separators outside the reversed run.
        let mut last = end;
        while last > start + 1 && is_space(&ordered[last - 1]) {
            last -= 1;
        }
        if last - start >= 2 {
            ordered[start..last].reverse();
            changed = true;
        }
        i = end;
    }

    if changed {
        // Re-number reading order to match the new sequence so downstream
        // converters (which may re-sort by reading_order) stay consistent.
        for (idx, o) in ordered.iter_mut().enumerate() {
            o.reading_order = idx;
        }
    }
}

#[cfg(test)]
mod rtl_reorder_tests {
    use super::*;
    use crate::geometry::Rect;
    use crate::layout::TextSpan;

    fn ordered(text: &str, x: f32, y: f32, order: usize) -> OrderedTextSpan {
        let span = TextSpan {
            text: text.to_string(),
            bbox: Rect::new(x, y, 10.0, 12.0),
            font_size: 12.0,
            ..TextSpan::default()
        };
        OrderedTextSpan::new(span, order)
    }

    // #557b: per-word RTL spans arrive in left-to-right reading order
    // (x ascending). The pipeline must reverse the WORD order so md/html/
    // plain-text read right-to-left, without char-flipping each word.
    #[test]
    fn reorder_rtl_word_runs_reverses_word_order() {
        // "اﻧﻮاع اﳋﻄﻮط اﻟﻌﺮﺑﻴﺔ": logical-first word "اﻧﻮاع" is rightmost.
        let mut v = vec![
            ordered("اﻟﻌﺮﺑﻴﺔ", 160.0, 700.0, 0),
            ordered(" ", 277.0, 700.0, 1),
            ordered("اﳋﻄﻮط", 288.0, 700.0, 2),
            ordered(" ", 409.0, 700.0, 3),
            ordered("اﻧﻮاع", 420.0, 700.0, 4),
        ];
        reorder_rtl_word_runs(&mut v);
        let texts: Vec<&str> = v.iter().map(|o| o.span.text.as_str()).collect();
        assert_eq!(texts, vec!["اﻧﻮاع", " ", "اﳋﻄﻮط", " ", "اﻟﻌﺮﺑﻴﺔ"]);
        // reading_order renumbered to match new sequence.
        assert!(v.iter().enumerate().all(|(i, o)| o.reading_order == i));
    }

    // LTR content must be left untouched.
    #[test]
    fn reorder_rtl_word_runs_leaves_ltr_alone() {
        let mut v = vec![
            ordered("The", 10.0, 700.0, 0),
            ordered(" ", 30.0, 700.0, 1),
            ordered("quick", 40.0, 700.0, 2),
        ];
        reorder_rtl_word_runs(&mut v);
        let texts: Vec<&str> = v.iter().map(|o| o.span.text.as_str()).collect();
        assert_eq!(texts, vec!["The", " ", "quick"]);
    }
}
