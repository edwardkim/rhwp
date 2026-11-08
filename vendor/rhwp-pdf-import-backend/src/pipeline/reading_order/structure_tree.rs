//! Structure tree reading order strategy.
//!
//! Uses PDF Tagged structure tree (ISO 32000-1:2008 Section 14.7) to determine
//! the correct reading order for text spans based on their MCID values.

use crate::error::Result;
use crate::layout::TextSpan;
use crate::pipeline::{OrderedTextSpan, ReadingOrderInfo};
use crate::structure::McidScope;

use super::{ArticleThreadStrategy, ReadingOrderContext, ReadingOrderStrategy, XYCutStrategy};

/// A marked-content id together with the content stream that defines it.
///
/// ISO 32000-1:2008 §14.7.4.2 makes an `/MCID` unique only "within its content
/// stream", so the id alone does not identify a piece of marked content: a
/// page and a Form XObject may each number theirs from 0.
pub(crate) type McidKey = (McidScope, u32);

/// The scope-qualified key for `span`, defaulting to the page's own scope when
/// the span carries none — the convention the rest of the crate uses.
fn span_mcid_key(span: &TextSpan, mcid: u32, page: u32) -> McidKey {
    (span.mcid_scope.clone().unwrap_or(McidScope::Page(page)), mcid)
}

/// Structure tree-based reading order strategy.
///
/// This is the PDF-spec-compliant approach for Tagged PDFs (ISO 32000-1:2008
/// Section 14.7). It uses the structure tree's pre-order traversal to determine
/// the logical reading order of marked content.
///
/// For spans without MCIDs or when no structure tree is available, it falls
/// back to the XYCutStrategy (recursive spatial partitioning).
pub struct StructureTreeStrategy {
    /// Fallback for spans without MCIDs and for MCID orderings that
    /// fail the column-respecting sanity check.
    fallback: XYCutStrategy,
}

impl StructureTreeStrategy {
    /// Construct a new strategy with a default XY-cut fallback.
    pub fn new() -> Self {
        Self {
            fallback: XYCutStrategy::new(),
        }
    }

    /// Order spans when the structure tree cannot be trusted for this page.
    ///
    /// Prefers article-thread order (`/Threads`, §12.4.3) when the canonical
    /// helper supplied bead rectangles for the page; otherwise uses the
    /// geometric XY-cut fallback. With no bead rects this is exactly the prior
    /// behaviour (fails closed).
    fn fallback_order(
        &self,
        spans: Vec<TextSpan>,
        context: &ReadingOrderContext,
    ) -> Result<Vec<OrderedTextSpan>> {
        if context.bead_rects.as_ref().is_some_and(|b| !b.is_empty()) {
            return ArticleThreadStrategy::new().apply(spans, context);
        }
        self.fallback.apply(spans, context)
    }
}

/// Detect whether applying `mcid_order` to `spans` would produce a
/// horizontal zigzag pattern — a reliable signal that MCIDs were
/// assigned in content-stream order rather than visual reading order
/// on a multi-column page.
///
/// Heuristic:
/// 1. Project span X-centers onto a 1-D histogram to detect 2+ columns
///    (requires a gap wider than the column width estimate).
/// 2. Walk the MCID-ordered sequence of spans and count how many times
///    it crosses between different column clusters.
/// 3. If crossings exceed `2 * (num_columns - 1)` the order is zigzagging
///    (a column-respecting order crosses columns only at bottom-of-one /
///    top-of-next transitions).
fn mcid_order_zigzags_columns(spans: &[TextSpan], mcid_order: &[McidKey], page: u32) -> bool {
    // Build ordered list of (span_index, x_center) in MCID order
    let mcid_to_idx: std::collections::HashMap<McidKey, usize> = spans
        .iter()
        .enumerate()
        .filter_map(|(i, s)| s.mcid.map(|m| (span_mcid_key(s, m, page), i)))
        .collect();
    let ordered_x: Vec<f32> = mcid_order
        .iter()
        .filter_map(|k| mcid_to_idx.get(k))
        .map(|&i| spans[i].bbox.x + spans[i].bbox.width * 0.5)
        .collect();
    if ordered_x.len() < 10 {
        return false;
    }

    // 1-D k-means-lite: detect if there are 2+ clusters of X positions
    // separated by a meaningful gap.
    let mut xs_sorted: Vec<f32> = ordered_x.clone();
    xs_sorted.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
    let x_min = xs_sorted[0];
    let x_max = xs_sorted[xs_sorted.len() - 1];
    let x_extent = x_max - x_min;
    if x_extent < 50.0 {
        return false; // single column
    }

    // Find the largest gap in sorted X positions
    let mut largest_gap = 0.0_f32;
    let mut largest_gap_at = x_min;
    for w in xs_sorted.windows(2) {
        let gap = w[1] - w[0];
        if gap > largest_gap {
            largest_gap = gap;
            largest_gap_at = (w[0] + w[1]) * 0.5;
        }
    }
    // The gap must be a substantial fraction of the page width to be
    // considered a column gutter (not just inter-word whitespace).
    if largest_gap < x_extent * 0.1 || largest_gap < 30.0 {
        return false;
    }

    // Classify each span as left-column (0) or right-column (1).
    let columns: Vec<u8> = ordered_x
        .iter()
        .map(|&x| if x < largest_gap_at { 0 } else { 1 })
        .collect();

    // Count transitions between columns in the MCID-ordered sequence.
    let crossings = columns.windows(2).filter(|w| w[0] != w[1]).count();
    // For proper column reading order: left-column finished, then a
    // SINGLE crossing to right-column. More than 3 crossings means
    // the order is interleaving columns rather than respecting them.
    crossings > 3
}

impl Default for StructureTreeStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadingOrderStrategy for StructureTreeStrategy {
    fn apply(
        &self,
        spans: Vec<TextSpan>,
        context: &ReadingOrderContext,
    ) -> Result<Vec<OrderedTextSpan>> {
        // If structure tree has suspect content, fall back to geometric ordering
        // Per ISO 32000-1:2008 Section 14.7.1, suspects=true means the structure
        // tree may contain errors or unreliable content.
        if context.suspects {
            log::debug!("Structure tree marked as suspect, falling back to geometric ordering");
            return self.fallback_order(spans, context);
        }

        // If no structure tree or MCID order, fall back (article threads first,
        // then geometric) — see `fallback_order`.
        let mcid_order = match &context.mcid_order {
            Some(order) if !order.is_empty() => order,
            _ => return self.fallback_order(spans, context),
        };

        // Trust-check: if the MCID ordering would zigzag horizontally
        // across a clear two-column layout, the structure tree is
        // untrustworthy for reading order (common in PDFs where the
        // authoring tool assigned MCIDs in content-stream order without
        // respecting column visual order). Fall back to geometric.
        if mcid_order_zigzags_columns(&spans, mcid_order, context.page_number) {
            log::debug!("MCID order zigzags across columns, falling back to geometric ordering");
            return self.fallback_order(spans, context);
        }

        // Create (scope, MCID) -> reading order mapping.
        //
        // The key carries the scope because ISO 32000-1:2008 §14.7.4.2 makes
        // an /MCID unique only "within its content stream": a page and a Form
        // XObject may each number theirs from 0. Keyed on the bare id, a
        // form's MCID 0 collided with the page's and the form's text was
        // ordered into the page's structural slot.
        let mcid_to_order: std::collections::HashMap<McidKey, usize> = mcid_order
            .iter()
            .enumerate()
            .map(|(order, key)| (key.clone(), order))
            .collect();

        // The spans' own keys, collected before the loop below consumes them.
        let span_keys: Vec<McidKey> = spans
            .iter()
            .filter_map(|s| s.mcid.map(|m| span_mcid_key(s, m, context.page_number)))
            .collect();

        // A span and the reference that names it can disagree about scope
        // without colliding. §14.7.4.3 lets a marked-content reference name its
        // stream with /Stm; a structure element whose kid is a bare integer
        // carries none and so resolves against the page's own stream — while a
        // producer that draws part of the page through a Form XObject numbers
        // one continuous id space across both. Every such run then failed to
        // find its slot and was demoted to "untagged", which appends it after
        // the whole tagged page: a callout label printed between two paragraphs
        // came out at the end. Match across scopes, but only where the bare id
        // is numbered once on each side — see `unambiguous_mcid_scopes`.
        let drawn_once = crate::structure::unambiguous_mcid_scopes(span_keys.iter());
        let referenced_once = crate::structure::unambiguous_mcid_scopes(mcid_to_order.keys());

        // Separate spans with and without MCIDs
        let mut with_mcid: Vec<(TextSpan, usize)> = Vec::new();
        let mut without_mcid: Vec<TextSpan> = Vec::new();

        for span in spans {
            if let Some(mcid) = span.mcid {
                let key = span_mcid_key(&span, mcid, context.page_number);
                let slot = if drawn_once.contains_key(&mcid) {
                    crate::structure::resolve_mcid_key(&mcid_to_order, &referenced_once, &key)
                        .and_then(|k| mcid_to_order.get(&k).copied())
                } else {
                    mcid_to_order.get(&key).copied()
                };
                if let Some(order) = slot {
                    with_mcid.push((span, order));
                } else {
                    // MCID not in structure tree - treat as untagged
                    without_mcid.push(span);
                }
            } else {
                without_mcid.push(span);
            }
        }

        // Sort spans by their structure tree order
        with_mcid.sort_by_key(|(_, order)| *order);

        // Build result: tagged spans first (in structure order), then untagged
        let mut result = Vec::new();
        let mut reading_order = 0;

        // Add tagged spans with StructureTree source
        for (span, _) in with_mcid {
            result.push(OrderedTextSpan::with_info(
                span,
                reading_order,
                ReadingOrderInfo::structure_tree(),
            ));
            reading_order += 1;
        }

        // Add untagged spans using fallback ordering with Fallback source
        if !without_mcid.is_empty() {
            let untagged_ordered = self.fallback.apply(without_mcid, context)?;
            for mut ordered_span in untagged_ordered {
                ordered_span.reading_order = reading_order;
                // Mark as fallback since these spans lack structure tree info
                ordered_span.order_info = ReadingOrderInfo::fallback();
                result.push(ordered_span);
                reading_order += 1;
            }
        }

        Ok(result)
    }

    fn name(&self) -> &'static str {
        "StructureTreeStrategy"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;
    use crate::layout::{Color, FontWeight};

    /// Page-scoped keys for the fixtures below, which build spans with no
    /// explicit `mcid_scope` (so they default to `Page(0)`).
    fn page_keys(ids: &[u32]) -> Vec<McidKey> {
        ids.iter().map(|&m| (McidScope::Page(0), m)).collect()
    }

    fn make_span(text: &str, x: f32, y: f32, mcid: Option<u32>) -> TextSpan {
        TextSpan {
            provenance: None,
            text_rise: 0.0,
            artifact_type: None,
            text: text.to_string(),
            bbox: Rect::new(x, y, 50.0, 12.0),
            font_name: "Test".to_string(),
            font_size: 12.0,
            font_weight: FontWeight::Normal,
            is_italic: false,
            is_monospace: false,
            color: Color::black(),
            mcid,
            mcid_scope: None,
            sequence: 0,
            offset_semantic: false,
            split_boundary_before: false,
            char_spacing: 0.0,
            word_spacing: 0.0,
            horizontal_scaling: 100.0,
            primary_detected: false,
            char_widths: vec![],
            char_x_offsets: Vec::new(),
            heading_level: None,
            rotation_degrees: 0.0,
            wmode: 0,
            rtl_draw_logical: false,
            mirrored: false,
            page_rotation_applied: 0,
        }
    }

    #[test]
    fn test_structure_tree_ordering() {
        // Spans with MCIDs in "wrong" visual order
        let spans = vec![
            make_span("Third", 0.0, 100.0, Some(2)),
            make_span("First", 0.0, 50.0, Some(0)),
            make_span("Second", 0.0, 75.0, Some(1)),
        ];

        let strategy = StructureTreeStrategy::new();
        let context = ReadingOrderContext::new().with_mcid_order(page_keys(&[0, 1, 2]));
        let ordered = strategy.apply(spans, &context).unwrap();

        assert_eq!(ordered[0].span.text, "First");
        assert_eq!(ordered[1].span.text, "Second");
        assert_eq!(ordered[2].span.text, "Third");
    }

    #[test]
    fn test_fallback_for_untagged() {
        // Mix of tagged and untagged spans
        let spans = vec![
            make_span("Tagged", 0.0, 100.0, Some(0)),
            make_span("Untagged", 0.0, 50.0, None),
        ];

        let strategy = StructureTreeStrategy::new();
        let context = ReadingOrderContext::new().with_mcid_order(page_keys(&[0]));
        let ordered = strategy.apply(spans, &context).unwrap();

        // Tagged comes first, then untagged
        assert_eq!(ordered[0].span.text, "Tagged");
        assert_eq!(ordered[1].span.text, "Untagged");
    }

    #[test]
    fn test_no_structure_tree_fallback() {
        let spans = vec![
            make_span("Bottom", 0.0, 50.0, None),
            make_span("Top", 0.0, 100.0, None),
        ];

        let strategy = StructureTreeStrategy::new();
        let context = ReadingOrderContext::new(); // No MCID order
        let ordered = strategy.apply(spans, &context).unwrap();

        // Should use geometric strategy fallback: top to bottom within column
        // Both spans are in same column (x=0), so ordered by Y (top first)
        assert_eq!(ordered[0].span.text, "Top");
        assert_eq!(ordered[1].span.text, "Bottom");
    }

    #[test]
    fn test_geometric_fallback_multi_column() {
        // Phase 8: Updated test to work with adaptive column detection
        // Test that multi-column documents are handled correctly via GeometricStrategy
        // Added word-level spans to provide realistic gap distribution for adaptive threshold
        let spans = vec![
            // Left column - multiple words with small gaps
            // (each make_span emits a 50pt-wide span; first three
            // share Y=100 and stride 5pt apart so the left column's
            // total X extent is 0..60pt + 50pt span width = 0..110pt,
            // which clears the MIN_RESULT_WIDTH_PT = 60pt floor that
            // find_horizontal_split applies to reject sliver columns).
            make_span("Left Top Word1", 0.0, 100.0, None),
            make_span("Left Top Word2", 5.0, 100.0, None), // 5pt word gap
            make_span("Left Top Word3", 10.0, 100.0, None), // 5pt word gap
            make_span("Left Bottom", 0.0, 50.0, None),
            // Right column (gap >> word gaps). Two spans at x=200
            // give a right-column extent of 200..250 = 50pt — below
            // the 60pt floor. Add a second word so the right column
            // has extent 200..305 = 105pt, well above the floor.
            make_span("Right Top", 200.0, 100.0, None),
            make_span("Right Top2", 255.0, 100.0, None),
            make_span("Right Bottom", 200.0, 50.0, None),
            make_span("Right Bottom2", 255.0, 50.0, None),
        ];

        let mut strategy = StructureTreeStrategy::new();
        strategy.fallback = XYCutStrategy::new().with_prefer_horizontal(true);
        let context = ReadingOrderContext::new(); // No MCID order
        let ordered = strategy.apply(spans, &context).unwrap();

        // Should process left column first, then right column
        // Verify all left spans come before right spans in order
        let left_indices: Vec<_> = ordered
            .iter()
            .enumerate()
            .filter(|(_, s)| s.span.text.starts_with("Left"))
            .map(|(i, _)| i)
            .collect();
        let right_indices: Vec<_> = ordered
            .iter()
            .enumerate()
            .filter(|(_, s)| s.span.text.starts_with("Right"))
            .map(|(i, _)| i)
            .collect();

        assert!(
            left_indices
                .iter()
                .all(|&l| right_indices.iter().all(|&r| l < r)),
            "Left column should be processed before right column"
        );
    }

    #[test]
    fn test_suspects_fallback_to_geometric() {
        // When suspects=true, structure tree order should be ignored
        // and geometric ordering should be used instead
        let spans = vec![
            make_span("StructOrder2", 0.0, 100.0, Some(1)), // MCID 1 = second
            make_span("StructOrder1", 0.0, 50.0, Some(0)),  // MCID 0 = first
        ];

        let strategy = StructureTreeStrategy::new();

        // With suspects=false, structure tree order is used
        let context = ReadingOrderContext::new()
            .with_mcid_order(page_keys(&[0, 1]))
            .with_suspects(false);
        let ordered = strategy.apply(spans.clone(), &context).unwrap();
        assert_eq!(ordered[0].span.text, "StructOrder1"); // MCID order
        assert_eq!(ordered[1].span.text, "StructOrder2");

        // With suspects=true, geometric order is used (top-to-bottom)
        let context = ReadingOrderContext::new()
            .with_mcid_order(page_keys(&[0, 1]))
            .with_suspects(true);
        let ordered = strategy.apply(spans, &context).unwrap();
        assert_eq!(ordered[0].span.text, "StructOrder2"); // Geometric: y=100 first (top)
        assert_eq!(ordered[1].span.text, "StructOrder1"); // y=50 second (bottom)
    }
}
