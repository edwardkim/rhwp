//! Tategaki (vertical writing) reading-order strategy.
//!
//! Right-to-left across columns, top-to-bottom within each column —
//! the layout convention for vertical Chinese, Japanese, and Korean
//! text. Spans whose horizontal X centers cluster together belong to
//! the same column.
//!
//! This strategy is dispatched by [`crate::pipeline::TextPipeline::process`]
//! when the per-span `wmode` tag indicates a vertical-majority page. The four
//! horizontal LTR strategies (Simple, Geometric, XYCut, StructureTree)
//! are left unchanged; tategaki always wins when the page is vertical.

use crate::error::Result;
use crate::layout::TextSpan;
use crate::pipeline::{OrderedTextSpan, ReadingOrderInfo};

use super::{ReadingOrderContext, ReadingOrderStrategy};

/// Right-to-left, top-to-bottom reading order for vertical writing
/// (CJK tategaki).
///
/// Delegates to `crate::utils::sort_vertical_tategaki`: spans are
/// clustered into columns by X-center proximity (single-linkage, median
/// span width as the tolerance), then ordered rightmost column first,
/// top-to-bottom within each column.
pub struct TategakiStrategy;

impl ReadingOrderStrategy for TategakiStrategy {
    fn apply(
        &self,
        spans: Vec<TextSpan>,
        _context: &ReadingOrderContext,
    ) -> Result<Vec<OrderedTextSpan>> {
        if spans.is_empty() {
            return Ok(Vec::new());
        }

        let sorted = crate::utils::sort_vertical_tategaki(spans, |s| &s.bbox);

        Ok(sorted
            .into_iter()
            .enumerate()
            .map(|(order, span)| {
                OrderedTextSpan::with_info(span, order, ReadingOrderInfo::simple())
            })
            .collect())
    }

    fn name(&self) -> &'static str {
        "TategakiStrategy"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;

    fn mk(text: &str, x: f32, y: f32) -> TextSpan {
        TextSpan {
            text: text.to_string(),
            bbox: Rect::new(x, y, 12.0, 12.0),
            font_size: 12.0,
            wmode: 1,
            ..TextSpan::default()
        }
    }

    /// Two columns: A,B,C at x=500 (right), D,E,F at x=300 (left).
    /// Reading order must be the right column top-down first, then
    /// the left column top-down.
    #[test]
    fn tategaki_two_columns_right_to_left_top_to_bottom() {
        let spans = vec![
            mk("D", 300.0, 700.0),
            mk("F", 300.0, 676.0),
            mk("B", 500.0, 688.0),
            mk("C", 500.0, 676.0),
            mk("A", 500.0, 700.0),
            mk("E", 300.0, 688.0),
        ];
        let strategy = TategakiStrategy;
        let context = ReadingOrderContext::new();
        let ordered = strategy.apply(spans, &context).unwrap();
        let combined: String = ordered.iter().map(|o| o.span.text.as_str()).collect();
        assert_eq!(combined, "ABCDEF");
    }

    /// A single column produces a top-down sequence.
    #[test]
    fn tategaki_single_column_top_to_bottom() {
        let spans = vec![
            mk("C", 300.0, 676.0),
            mk("A", 300.0, 700.0),
            mk("B", 300.0, 688.0),
        ];
        let strategy = TategakiStrategy;
        let context = ReadingOrderContext::new();
        let ordered = strategy.apply(spans, &context).unwrap();
        let combined: String = ordered.iter().map(|o| o.span.text.as_str()).collect();
        assert_eq!(combined, "ABC");
    }

    /// X-centers chaining within the tolerance form ONE column
    /// (single-linkage), read top-to-bottom — the input class that made
    /// the old banded/pairwise comparator non-transitive and panicked.
    #[test]
    fn tategaki_chained_centers_total_order() {
        let spans: Vec<TextSpan> = (0..64)
            .map(|i| {
                let x = i as f32 * 10.0;
                let y = ((i * 37) % 64) as f32 * 7.0; // distinct, scrambled
                mk(&format!("s{i}"), x, y)
            })
            .collect();
        let strategy = TategakiStrategy;
        let context = ReadingOrderContext::new();
        let ordered = strategy.apply(spans, &context).unwrap();
        assert_eq!(ordered.len(), 64);
        let ys: Vec<f32> = ordered.iter().map(|o| o.span.bbox.y).collect();
        assert!(
            ys.windows(2).all(|w| w[0] >= w[1]),
            "chained centers must read as one column, top-to-bottom: {ys:?}"
        );
    }

    /// Non-finite coordinates must not panic the sort, and every span
    /// must survive.
    #[test]
    fn tategaki_nan_coordinates_do_not_panic() {
        let mut spans: Vec<TextSpan> = (0..32)
            .map(|i| mk(&format!("s{i}"), (i % 8) as f32 * 10.0, i as f32 * 5.0))
            .collect();
        spans[3].bbox.x = f32::NAN;
        spans[11].bbox.y = f32::NAN;
        spans[17].bbox.width = f32::NAN;
        let strategy = TategakiStrategy;
        let context = ReadingOrderContext::new();
        let ordered = strategy.apply(spans, &context).unwrap();
        assert_eq!(ordered.len(), 32);
    }
}
