//! Region classification: prose / reference / table / form discrimination.
//!
//! # Why this module exists
//!
//! Untagged multi-column scientific papers (PMC/arXiv) carry **no** spec
//! reading order (ISO 32000-1 §14.8.2.3.1), so the geometric reading-order
//! heuristics in [`crate::document`] must reconstruct column order. The single
//! blocker is a **discrimination problem**: the gates that decide "reorder this
//! page column-major" cannot tell a two-column *prose* / *reference* body from a
//! *table* / *form* using thresholds alone. Loosen a gate to admit ragged
//! reference columns and it swallows tables; tighten it to protect tables and it
//! rejects the references. Prior threshold-only attempts were each reverted by
//! the corpus sweep on `google_doc_document.pdf`'s population table.
//!
//! The seed of the right answer already lived in
//! [`crate::pipeline::reading_order::xycut`]'s private `classify_region_kind`
//! (Prose / Table / Mixed via per-line `mean_chars` + narrow/wide line shape).
//! This module promotes that idea to a **first-class, page-level, unit-tested
//! primitive** and adds the two classes that seed was missing:
//!
//! * [`RegionClass::Reference`] — ragged two-column reference lists (numbered or
//!   hanging-indent entries with ragged right edges). Treated like prose by the
//!   reorder gates (reorder column-major), but named distinctly so a future pass
//!   can apply reference-specific entry grouping.
//! * [`RegionClass::Form`] — label / value rows (a large intra-line gap with text
//!   on both sides), the IRS-form shape that a relaxed prose gate would otherwise
//!   mis-read as a prose gutter.
//!
//! # Contract
//!
//! [`classify_region`] is a **pure read** (never mutates spans) and returns
//! [`RegionClass::Mixed`] on any ambiguity. Callers gate on the *class*, so a
//! misclassification degrades gracefully to the pre-existing geometric behaviour
//! rather than corrupting output.

use crate::layout::TextSpan;

/// Coarse structural class of a contiguous block of text spans.
///
/// Reorder gates admit `Prose` / `Reference` (reorder column-major) and reject
/// `Table` / `Form` (leave the existing row-major / cell handling alone).
/// `Mixed` means "not confidently any of the above" → callers fall back to their
/// prior behaviour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionClass {
    /// Tall stack of wide lines, or narrow column lines carrying substantial
    /// prose content per line. Safe to reorder column-major.
    Prose,
    /// Ragged reference / bibliography column: numbered entries (`12.` / `[12]`)
    /// or hanging-indent entries with ragged-right tail lines. Reorder
    /// column-major like prose.
    Reference,
    /// Short cells in a grid (low mean characters per line, narrow-dominant).
    /// The canonical `google_doc_document.pdf` population table. Tight column
    /// reorder here corrupts cell ordering — do NOT treat as prose.
    Table,
    /// Label / value rows (a large intra-line gap with text on both sides), e.g.
    /// an IRS tax form. Do NOT treat as a prose gutter.
    Form,
    /// Too few lines, mixed shapes, or otherwise not confidently classifiable.
    Mixed,
}

impl RegionClass {
    /// True for the classes the column-reorder gates should accept (a 2-column
    /// body whose halves are both prose/reference is read column-major).
    pub fn is_reorderable_column(self) -> bool {
        matches!(self, RegionClass::Prose | RegionClass::Reference)
    }
}

/// Per-line aggregate built during classification.
struct LineStat {
    top: f32,
    left: f32,
    right: f32,
    /// Non-whitespace character count across all spans on the line.
    nonws_chars: usize,
    /// Trimmed text of the leftmost span on the line (for numbered-entry shape).
    lead_text: String,
    /// Left edges of every span on the line (sorted ascending), for intra-line
    /// gap (form) detection.
    span_lefts: Vec<f32>,
    /// Right edges paired with `span_lefts` order, for the same.
    span_rights: Vec<f32>,
}

/// Classify the block formed by `spans[indices]`.
///
/// Pure read. Returns [`RegionClass::Mixed`] whenever the block is too small or
/// the shape is ambiguous, so callers safely fall back to prior behaviour.
pub fn classify_region(spans: &[TextSpan], indices: &[usize]) -> RegionClass {
    // --- cheap shape guards (mirror xycut's classify_region_kind) ---
    if indices.len() < 6 {
        return RegionClass::Mixed;
    }
    let mut x_min = f32::MAX;
    let mut x_max = f32::MIN;
    for &i in indices {
        x_min = x_min.min(spans[i].bbox.left());
        x_max = x_max.max(spans[i].bbox.right());
    }
    let region_width = x_max - x_min;
    if region_width <= 10.0 {
        return RegionClass::Mixed;
    }

    // Median glyph height drives the line-clustering Y tolerance.
    let med_h = median_height(spans, indices).max(1.0);

    let lines = cluster_lines(spans, indices, med_h);
    let line_count = lines.len();
    if line_count < 6 {
        // Headings, captions, single paragraphs — leave to default behaviour.
        return RegionClass::Mixed;
    }

    // --- per-line statistics ---
    let mut total_chars = 0usize;
    let mut wide_lines = 0usize;
    let mut numbered_lines = 0usize;
    let mut form_lines = 0usize;
    let mut left_edges: Vec<f32> = Vec::with_capacity(line_count);
    for l in &lines {
        total_chars += l.nonws_chars;
        let extent = (l.right - l.left).max(0.0);
        if extent >= region_width * 0.6 {
            wide_lines += 1;
        }
        if starts_numbered_entry(&l.lead_text) {
            numbered_lines += 1;
        }
        if line_has_label_value_gap(l, region_width) {
            form_lines += 1;
        }
        left_edges.push(l.left);
    }
    let mean_chars = total_chars as f32 / line_count as f32;
    let mostly_wide = wide_lines * 2 > line_count;
    let numbered_frac = numbered_lines as f32 / line_count as f32;
    let form_frac = form_lines as f32 / line_count as f32;

    // --- decision ladder (specific → general; default Mixed) ---
    //
    // Table and Form both mean "do NOT reorder as prose", so a fuzzy Table/Form
    // boundary is harmless — only the {Prose,Reference} vs {Table,Form} split is
    // load-bearing for the reorder gates.

    // TABLE: short content per line. This is the robust grid signal — a prose or
    // reference column always carries substantial text per line, so it never
    // falls this low, whereas the google_doc_document population table's
    // digit-only cells (≤ 7 chars) and any short-cell grid land here regardless
    // of how wide the row spans.
    if mean_chars < 10.0 {
        return RegionClass::Table;
    }

    // FORM: a large fraction of lines are label … value rows (a wide interior gap
    // with text on both sides). Distinguishes IRS tax forms — whose label text is
    // long enough to otherwise read as prose — from a real prose body.
    if form_frac >= 0.4 {
        return RegionClass::Form;
    }

    // REFERENCE: numbered entries, or a hanging-indent two-level left edge, with
    // enough text per line to exclude table cells. Treated like prose downstream.
    if mean_chars > 12.0 && (numbered_frac >= 0.3 || has_hanging_indent(&left_edges, med_h)) {
        return RegionClass::Reference;
    }

    // PROSE: a tall stack of wide lines with substantial content per line (the
    // xycut `mean_chars > 20` body signal). The `mostly_narrow` half-column verse
    // path xycut also admits is deliberately NOT reproduced here: it risks
    // pulling a wide-number table into the reorder gate, and our academic targets
    // are wide-line column bodies, not short-verse editions (those degrade to
    // Mixed = prior behaviour, which is safe).
    if mean_chars > 20.0 && mostly_wide {
        return RegionClass::Prose;
    }

    RegionClass::Mixed
}

/// Median of the spans' glyph heights (linear-time enough for blocks).
fn median_height(spans: &[TextSpan], indices: &[usize]) -> f32 {
    let mut hs: Vec<f32> = indices
        .iter()
        .map(|&i| spans[i].bbox.height.abs())
        .filter(|h| *h > 0.0)
        .collect();
    if hs.is_empty() {
        return 1.0;
    }
    hs.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
    hs[hs.len() / 2]
}

/// Cluster spans into baseline lines (top→bottom), tolerant of small Y jitter.
fn cluster_lines(spans: &[TextSpan], indices: &[usize], med_h: f32) -> Vec<LineStat> {
    let mut order: Vec<usize> = indices.to_vec();
    order.sort_by(|&a, &b| {
        crate::utils::safe_float_cmp(spans[a].bbox.top(), spans[b].bbox.top())
            .then_with(|| crate::utils::safe_float_cmp(spans[a].bbox.left(), spans[b].bbox.left()))
    });

    let tol = med_h * 0.6;
    let mut lines: Vec<LineStat> = Vec::new();
    for &i in &order {
        let s = &spans[i];
        let nonws = s.text.chars().filter(|c| !c.is_whitespace()).count();
        match lines.last_mut() {
            Some(l) if (s.bbox.top() - l.top).abs() <= tol => {
                l.left = l.left.min(s.bbox.left());
                l.right = l.right.max(s.bbox.right());
                l.nonws_chars += nonws;
                if s.bbox.left() < l.span_lefts[0] {
                    // New leftmost span on this line → it owns the lead text.
                    l.lead_text = s.text.trim_start().to_string();
                }
                l.span_lefts.push(s.bbox.left());
                l.span_rights.push(s.bbox.right());
            },
            _ => lines.push(LineStat {
                top: s.bbox.top(),
                left: s.bbox.left(),
                right: s.bbox.right(),
                nonws_chars: nonws,
                lead_text: s.text.trim_start().to_string(),
                span_lefts: vec![s.bbox.left()],
                span_rights: vec![s.bbox.right()],
            }),
        }
    }
    // Keep `span_lefts`/`span_rights` paired and left-sorted for gap analysis.
    for l in &mut lines {
        let mut paired: Vec<(f32, f32)> = l
            .span_lefts
            .iter()
            .copied()
            .zip(l.span_rights.iter().copied())
            .collect();
        paired.sort_by(|a, b| crate::utils::safe_float_cmp(a.0, b.0));
        l.span_lefts = paired.iter().map(|p| p.0).collect();
        l.span_rights = paired.iter().map(|p| p.1).collect();
    }
    lines
}

/// True when a line's leading text begins a numbered/bracketed reference entry:
/// `12.`, `12)`, `[12]`, `(12)`.
fn starts_numbered_entry(lead: &str) -> bool {
    let b = lead.as_bytes();
    if b.is_empty() {
        return false;
    }
    // `[12]` / `[12` / `(12`
    if (b[0] == b'[' || b[0] == b'(') && b.get(1).is_some_and(u8::is_ascii_digit) {
        return true;
    }
    // `12.` / `12)` — 1..=3 leading digits then a `.` or `)`.
    let digits = b.iter().take(4).take_while(|c| c.is_ascii_digit()).count();
    if (1..=3).contains(&digits) {
        if let Some(&next) = b.get(digits) {
            return next == b'.' || next == b')';
        }
    }
    false
}

/// True when a line is a label … value row: one large interior horizontal gap
/// (≥ 0.25 · region width) with real text on both sides — the IRS-form shape.
fn line_has_label_value_gap(l: &LineStat, region_width: f32) -> bool {
    if l.span_lefts.len() < 2 {
        return false;
    }
    let threshold = region_width * 0.25;
    // Largest gap between the right edge of one span and the left of the next.
    for w in 1..l.span_lefts.len() {
        let gap = l.span_lefts[w] - l.span_rights[w - 1];
        if gap >= threshold {
            return true;
        }
    }
    false
}

/// Detect a hanging-indent two-level left edge: a primary entry-start edge `l0`
/// and a secondary continuation edge `l0 + δ` (δ ∈ ~[0.8, 5]·med_h), both
/// carrying a meaningful share of lines. Reference lists and prose with
/// first-line indents both produce this bimodality — for the reorder gate that
/// distinction does not matter (both reorder column-major).
fn has_hanging_indent(left_edges: &[f32], med_h: f32) -> bool {
    if left_edges.len() < 6 {
        return false;
    }
    let l0 = left_edges.iter().copied().fold(f32::MAX, f32::min);
    let near_tol = med_h * 0.5;
    let lo_band = left_edges
        .iter()
        .filter(|&&x| (x - l0).abs() <= near_tol)
        .count();
    // Continuation band: lines indented δ ∈ [0.8, 5]·med_h past l0.
    let hi_band = left_edges
        .iter()
        .filter(|&&x| {
            let d = x - l0;
            d >= med_h * 0.8 && d <= med_h * 5.0
        })
        .count();
    let n = left_edges.len();
    // Both bands must be substantial (≥ 25% of lines each).
    lo_band * 4 >= n && hi_band * 4 >= n
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;

    /// Build a minimal span at (left, top) with width/height and given text.
    fn span(text: &str, left: f32, top: f32, width: f32, height: f32) -> TextSpan {
        TextSpan {
            text: text.to_string(),
            bbox: Rect::new(left, top, width, height),
            font_size: height,
            ..Default::default()
        }
    }

    /// A line of prose: one wide span ~`chars` characters long at `top`.
    fn prose_line(top: f32, left: f32, chars: usize) -> TextSpan {
        let text: String = "x".repeat(chars);
        span(&text, left, top, chars as f32 * 5.0, 10.0)
    }

    fn classify(spans: &[TextSpan]) -> RegionClass {
        let idx: Vec<usize> = (0..spans.len()).collect();
        classify_region(spans, &idx)
    }

    #[test]
    fn classify_dense_results_is_prose() {
        // 10 wide lines, ~40 chars each → mean_chars > 20, mostly wide.
        let spans: Vec<TextSpan> = (0..10)
            .map(|i| prose_line(i as f32 * 12.0, 0.0, 40))
            .collect();
        assert_eq!(classify(&spans), RegionClass::Prose);
    }

    #[test]
    fn classify_numbered_references_is_reference() {
        // Each line starts "12. Author, Title ..." → numbered entries.
        let spans: Vec<TextSpan> = (0..8)
            .map(|i| {
                let t = format!("{}. Author A, Title of the work, Journal", i + 1);
                span(&t, 0.0, i as f32 * 12.0, 180.0, 10.0)
            })
            .collect();
        assert_eq!(classify(&spans), RegionClass::Reference);
    }

    #[test]
    fn classify_hanging_indent_references_is_reference() {
        // Two-line entries: entry start at x=0, continuation indented to x=15.
        let mut spans = Vec::new();
        for e in 0..4 {
            let base = e as f32 * 24.0;
            spans.push(span(
                "Smith J, Some long reference entry title here",
                0.0,
                base,
                200.0,
                10.0,
            ));
            spans.push(span(
                "continuation of the reference line indented",
                15.0,
                base + 12.0,
                180.0,
                10.0,
            ));
        }
        assert_eq!(classify(&spans), RegionClass::Reference);
    }

    #[test]
    fn classify_table_cells_is_table() {
        // 12 short numeric cells across 6 rows, narrow → mean_chars < 8.
        let mut spans = Vec::new();
        for r in 0..6 {
            spans.push(span("12.3", 0.0, r as f32 * 12.0, 18.0, 10.0));
            spans.push(span("45.6", 60.0, r as f32 * 12.0, 18.0, 10.0));
        }
        assert_eq!(classify(&spans), RegionClass::Table);
    }

    #[test]
    fn classify_form_label_value_is_form() {
        // Label at left, value far right → large interior gap on every line.
        let spans: Vec<TextSpan> = (0..8)
            .map(|i| {
                let mut label =
                    span("Wages, salaries, tips, etc.", 0.0, i as f32 * 12.0, 90.0, 10.0);
                label.text = "Wages, salaries, tips".to_string();
                label
            })
            .collect();
        // Add right-aligned value spans creating the gap.
        let mut all = Vec::new();
        for (i, l) in spans.into_iter().enumerate() {
            all.push(l);
            all.push(span("1,234", 200.0, i as f32 * 12.0, 30.0, 10.0));
        }
        assert_eq!(classify(&all), RegionClass::Form);
    }

    #[test]
    fn classify_single_paragraph_is_mixed() {
        // Only 4 lines → below the 6-line substantiality floor.
        let spans: Vec<TextSpan> = (0..4)
            .map(|i| prose_line(i as f32 * 12.0, 0.0, 40))
            .collect();
        assert_eq!(classify(&spans), RegionClass::Mixed);
    }

    #[test]
    fn classify_empty_or_tiny_is_mixed() {
        assert_eq!(classify(&[]), RegionClass::Mixed);
        let spans: Vec<TextSpan> = (0..3)
            .map(|i| prose_line(i as f32 * 12.0, 0.0, 10))
            .collect();
        assert_eq!(classify(&spans), RegionClass::Mixed);
    }
}
