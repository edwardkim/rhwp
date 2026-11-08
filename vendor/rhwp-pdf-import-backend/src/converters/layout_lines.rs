//! Span-to-line grouping for layout-preserving office writers.
//!
//! All three layout-preserving writers (`docx_layout`, `pptx_layout`,
//! `xlsx_layout`) used to emit one frame/shape per source PDF text
//! span. That looks great when the renderer can substitute the source
//! font (so each shape's exact bbox width matches the rendered text),
//! but Word/PowerPoint/Excel rarely have the source's TeX/LaTeX
//! fonts installed and silently substitute Helvetica — wider than
//! TeXGyre or NimbusSan by ~5% per character. Adjacent shapes that
//! sat flush in the source PDF then overflow into each other:
//! visible "text on top of text" on every PDF→office→PDF round-trip
//! of academic papers.
//!
//! Grouping spans into *lines* (by Y position) and emitting one
//! frame per line — with the line's spans as separate runs — fixes
//! it: a single frame contains the whole line, the renderer's own
//! kerning fills inter-run gaps, and there's no inter-frame overflow.
//! Per-glyph x positions don't survive but the line-level layout
//! does, which is the right trade-off for visual fidelity when fonts
//! substitute.

use crate::layout::text_block::TextSpan;

/// A horizontal line of text spans extracted from a PDF page.
/// All spans share approximately the same Y baseline (within
/// [`LINE_Y_TOLERANCE_PT`]). The line's bounding box is the union of
/// its spans' bboxes; `font_size` is the largest span size on the
/// line so frame heights cover the tallest run.
#[derive(Debug, Clone)]
pub(crate) struct Line {
    pub spans: Vec<TextSpan>,
    /// Left edge of the line (min x across spans), in PDF points.
    pub x_pt: f32,
    /// Bottom-left baseline Y of the line (PDF coords, y-up), in points.
    pub y_pt: f32,
    /// Height of the tallest span on the line, in points.
    pub height_pt: f32,
    /// Total width from the leftmost span's x to the rightmost
    /// span's right edge, in points.
    pub width_pt: f32,
}

/// Y-position tolerance (points) for considering two spans on the
/// same line. Source PDFs sometimes emit spans on the same visual
/// line at slightly different y baselines (sub/superscript adjustments,
/// font-metric quirks). 2 pt is permissive enough to catch them
/// without merging spans from genuinely different lines (line-height
/// is typically ≥ 8 pt).
const LINE_Y_TOLERANCE_PT: f32 = 2.0;

/// Group spans into lines. Spans within `LINE_Y_TOLERANCE_PT` of
/// each other on the y axis (PDF coords, y-up) and on the same page
/// are merged into one [`Line`]; each line's spans are sorted by x
/// so the runs emit left-to-right.
///
/// Input spans are taken by value (the caller's vec is consumed) so
/// we can sort + reorder freely. Empty input returns an empty vec.
pub(crate) fn group_spans_into_lines(spans: Vec<TextSpan>) -> Vec<Line> {
    if spans.is_empty() {
        return Vec::new();
    }

    let mut sorted = spans;
    // Sort by descending y (top-of-page first), then by x within a line.
    sorted.sort_by(|a, b| {
        crate::utils::safe_float_cmp(b.bbox.y, a.bbox.y)
            .then_with(|| crate::utils::safe_float_cmp(a.bbox.x, b.bbox.x))
    });

    let mut lines: Vec<Line> = Vec::new();
    for span in sorted {
        // Skip empty/null-only spans early — they confuse line
        // bookkeeping (zero-width bboxes anchor the line at junk
        // x positions).
        let trimmed = span.text.trim_matches('\u{0000}');
        if trimmed.is_empty() {
            continue;
        }

        // Match against the most recently opened line first; PDFs
        // emit spans roughly top-to-bottom so the active line is
        // almost always the right home.
        //
        // Also reject merges when the candidate span's font size
        // differs from the line's existing spans by > 2×. This
        // catches drop caps — a 68-pt "A" wrapped by body text at
        // 8 pt should NOT share a paragraph frame with that body
        // text, even when their bounding boxes vertically overlap.
        // Without this guard, an academic-paper drop cap renders
        // inline with the body text following it as one giant
        // heading-class frame.
        let placed = lines.last_mut().and_then(|line| {
            let baseline = line.y_pt;
            if (span.bbox.y - baseline).abs() > LINE_Y_TOLERANCE_PT {
                return None;
            }
            let line_max_size = line
                .spans
                .iter()
                .map(|s| s.font_size)
                .fold(0.0_f32, f32::max);
            let line_min_size = line
                .spans
                .iter()
                .map(|s| s.font_size)
                .fold(f32::INFINITY, f32::min);
            let combined_max = line_max_size.max(span.font_size);
            let combined_min = line_min_size.min(span.font_size).max(0.1);
            if combined_max / combined_min > 2.0 {
                return None;
            }
            // Reject the merge when the candidate span sits far to
            // the right of the line's current extent — that's a
            // multi-column page (e.g. a 2-column academic paper, a
            // multi-column newspaper) where two columns happen to
            // share a baseline. Threshold is `max_fs * 4` ≈ 36–48 pt
            // for typical body text, well wider than any justified
            // inter-word gap but narrower than the typical 60+ pt
            // column gutter.
            let line_right = line.x_pt + line.width_pt;
            let gap = span.bbox.x - line_right;
            let max_gap = combined_max * 4.0;
            if gap > max_gap {
                return None;
            }
            Some(line)
        });
        if let Some(line) = placed {
            // Update bbox.
            let span_right = span.bbox.x + span.bbox.width;
            let line_right = line.x_pt + line.width_pt;
            let new_left = line.x_pt.min(span.bbox.x);
            let new_right = line_right.max(span_right);
            line.x_pt = new_left;
            line.width_pt = (new_right - new_left).max(0.0);
            line.height_pt = line.height_pt.max(span.bbox.height);
            line.spans.push(span);
        } else {
            let line = Line {
                x_pt: span.bbox.x,
                y_pt: span.bbox.y,
                height_pt: span.bbox.height,
                width_pt: span.bbox.width,
                spans: vec![span],
            };
            lines.push(line);
        }
    }

    // Re-sort each line's spans by x (the placement above appends
    // in sorted-input order, but later lines could pick up an
    // earlier-x span if Y tolerance bridged two lines).
    for line in &mut lines {
        line.spans
            .sort_by(|a, b| crate::utils::safe_float_cmp(a.bbox.x, b.bbox.x));
    }

    lines
}
