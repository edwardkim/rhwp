//! Article-thread reading order strategy (ISO 32000-1:2008 §12.4.3).
//!
//! When a page is governed by article-thread beads (`/Threads`), spans are read
//! by walking the beads in their chain (`/N`) order: all spans whose centre
//! falls inside a bead are emitted together, ordered top-to-bottom/left-to-right
//! within the bead. Spans captured by no bead are appended via the geometric
//! fallback so nothing is dropped (the "partial coverage" case).
//!
//! This strategy is only selected when [`ReadingOrderContext::bead_rects`] is
//! populated — which the canonical [`crate::pipeline::page_order`] helper does
//! only for non-tagged pages whose beads cover ≥80% of the page text. With no
//! bead rects the geometric path runs unchanged (fails closed).

use crate::error::Result;
use crate::geometry::{Point, Rect};
use crate::layout::TextSpan;
use crate::pipeline::{OrderedTextSpan, ReadingOrderInfo};

use super::{ReadingOrderContext, ReadingOrderStrategy, XYCutStrategy};

/// Article-thread (`/Threads`) reading order strategy.
pub struct ArticleThreadStrategy {
    /// Fallback for spans not captured by any bead, and for the no-bead case.
    fallback: XYCutStrategy,
}

impl ArticleThreadStrategy {
    /// Construct a new strategy with a default XY-cut fallback.
    pub fn new() -> Self {
        Self {
            fallback: XYCutStrategy::new(),
        }
    }
}

impl Default for ArticleThreadStrategy {
    fn default() -> Self {
        Self::new()
    }
}

/// Centre point of a span's bounding box.
fn span_center(span: &TextSpan) -> Point {
    Point {
        x: span.bbox.x + span.bbox.width * 0.5,
        y: span.bbox.y + span.bbox.height * 0.5,
    }
}

/// Sort indices into `spans` top-to-bottom (Y descending), then left-to-right
/// (X ascending) — matching `SimpleStrategy`'s convention for a single region.
fn sort_reading_within_region(indices: &mut [usize], spans: &[TextSpan]) {
    indices.sort_by(|&a, &b| {
        let y = crate::utils::safe_float_cmp(spans[b].bbox.y, spans[a].bbox.y);
        if y != std::cmp::Ordering::Equal {
            return y;
        }
        crate::utils::safe_float_cmp(spans[a].bbox.x, spans[b].bbox.x)
    });
}

impl ReadingOrderStrategy for ArticleThreadStrategy {
    fn apply(
        &self,
        spans: Vec<TextSpan>,
        context: &ReadingOrderContext,
    ) -> Result<Vec<OrderedTextSpan>> {
        // No bead rects → behave exactly like the geometric fallback.
        let beads: &[Rect] = match &context.bead_rects {
            Some(b) if !b.is_empty() => b,
            _ => return self.fallback.apply(spans, context),
        };

        // Assign each span to the first bead (in chain order) that contains its
        // centre; spans matching no bead are left for the geometric fallback.
        let mut per_bead: Vec<Vec<usize>> = vec![Vec::new(); beads.len()];
        let mut leftover: Vec<usize> = Vec::new();
        for (i, span) in spans.iter().enumerate() {
            let c = span_center(span);
            match beads.iter().position(|r| r.contains_point(&c)) {
                Some(b) => per_bead[b].push(i),
                None => leftover.push(i),
            }
        }

        // Within each bead, order spans geometrically; emit beads in chain order.
        let mut order_for_index: Vec<Option<usize>> = vec![None; spans.len()];
        let mut next_order = 0usize;
        for bead_indices in per_bead.iter_mut() {
            sort_reading_within_region(bead_indices, &spans);
            for &i in bead_indices.iter() {
                order_for_index[i] = Some(next_order);
                next_order += 1;
            }
        }

        // Build the captured output (beads), tagged as ArticleThread.
        let mut captured: Vec<(usize, TextSpan)> = Vec::new();
        // Leftover spans are ordered by the geometric fallback and appended
        // after all bead content, preserving their relative geometric order.
        let leftover_spans: Vec<TextSpan> = leftover.iter().map(|&i| spans[i].clone()).collect();

        for (i, span) in spans.into_iter().enumerate() {
            if let Some(order) = order_for_index[i] {
                captured.push((order, span));
            }
        }
        captured.sort_by_key(|(order, _)| *order);

        let mut result: Vec<OrderedTextSpan> = captured
            .into_iter()
            .map(|(order, span)| {
                OrderedTextSpan::with_info(span, order, ReadingOrderInfo::article_thread())
            })
            .collect();

        if !leftover_spans.is_empty() {
            let tail = self.fallback.apply(leftover_spans, context)?;
            let base = result.len();
            for (k, mut o) in tail.into_iter().enumerate() {
                o.reading_order = base + k;
                result.push(o);
            }
        }

        Ok(result)
    }

    fn name(&self) -> &'static str {
        "ArticleThreadStrategy"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;
    use crate::layout::TextSpan;
    use crate::pipeline::ReadingOrderSource;

    fn span(text: &str, x: f32, y: f32) -> TextSpan {
        TextSpan {
            text: text.to_string(),
            bbox: Rect::new(x, y, 20.0, 10.0),
            font_size: 10.0,
            ..TextSpan::default()
        }
    }

    fn texts(ordered: &[OrderedTextSpan]) -> Vec<String> {
        ordered.iter().map(|o| o.span.text.clone()).collect()
    }

    #[test]
    fn two_column_beads_read_left_column_then_right() {
        // Two bead columns: left bead (x 0..100), right bead (x 200..300).
        // Spans are supplied in a scrambled order; thread order must be the
        // left column top-to-bottom, then the right column top-to-bottom.
        let spans = vec![
            span("R-top", 210.0, 500.0),
            span("L-bot", 10.0, 400.0),
            span("R-bot", 210.0, 400.0),
            span("L-top", 10.0, 500.0),
        ];
        let ctx = ReadingOrderContext::new().with_bead_rects(vec![
            Rect::from_points(0.0, 380.0, 100.0, 520.0), // left column
            Rect::from_points(200.0, 380.0, 300.0, 520.0), // right column
        ]);

        let ordered = ArticleThreadStrategy::new().apply(spans, &ctx).unwrap();
        assert_eq!(texts(&ordered), vec!["L-top", "L-bot", "R-top", "R-bot"]);
        assert!(ordered
            .iter()
            .all(|o| o.order_info.source == ReadingOrderSource::ArticleThread));
    }

    #[test]
    fn spans_outside_all_beads_are_appended_not_dropped() {
        let spans = vec![
            span("in-bead", 10.0, 500.0),
            span("orphan", 400.0, 100.0), // outside every bead
        ];
        let ctx = ReadingOrderContext::new()
            .with_bead_rects(vec![Rect::from_points(0.0, 480.0, 100.0, 520.0)]);

        let ordered = ArticleThreadStrategy::new().apply(spans, &ctx).unwrap();
        let t = texts(&ordered);
        assert_eq!(t.len(), 2, "no span may be dropped");
        assert_eq!(t[0], "in-bead", "bead content comes first");
        assert!(t.contains(&"orphan".to_string()), "orphan must be appended");
    }

    #[test]
    fn no_bead_rects_falls_back_to_geometric() {
        // Empty/absent bead rects → identical to the geometric fallback.
        let spans = vec![span("a", 10.0, 500.0), span("b", 10.0, 400.0)];
        let ctx = ReadingOrderContext::new();
        let ordered = ArticleThreadStrategy::new().apply(spans, &ctx).unwrap();
        assert_eq!(ordered.len(), 2);
    }
}
