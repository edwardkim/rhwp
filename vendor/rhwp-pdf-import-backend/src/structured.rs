//! Structured per-page extraction (`extract_structured`) — issue #536.
//!
//! `PdfDocument::extract_structured(page)` returns a [`StructuredPage`]: the
//! page's text grouped into typed [`StructuredRegion`]s (body blocks, headings,
//! header/footer/page-number chrome, marginal labels) in reading order, with a
//! best-effort `column_index` for multi-column bodies.
//!
//! This is an **additive aggregation layer** over signals the extractor already
//! attaches to every [`TextSpan`](crate::layout::TextSpan):
//!
//! * `artifact_type` ([`crate::extractors::text::ArtifactType`]) →
//!   header / footer / page-number / artifact roles, per ISO 32000-1:2008
//!   §14.8.2.2 ("Real Content and Artifacts"). For a tagged PDF these come from
//!   the `/Artifact` marked-content sequences (§14.6.2); they are honoured
//!   for free.
//! * `heading_level` → [`RegionRole::StructuralHeading`]. Populated from the
//!   structure tree (`H1`..`H6`, §14.7.2) when the PDF is tagged, or from a
//!   font-size heuristic when it is not.
//! * span geometry → column assignment per §14.8.2.3.1 ("Page Content Order":
//!   multi-column layouts read column to column).
//!
//! Because the role signals already ride on the spans, a trustworthy
//! `/StructTreeRoot` (see [`crate::document::PdfDocument::prefers_structure_reading_order`])
//! drives the region roles automatically; untagged PDFs fall back to the
//! geometric/heuristic signals.

use crate::extractors::text::{ArtifactType, PaginationSubtype};
use crate::geometry::Rect;
use crate::layout::TextSpan;

/// A single page decomposed into typed regions in reading order.
#[derive(Debug, Clone, serde::Serialize)]
#[cfg_attr(feature = "wasm", serde(rename_all = "camelCase"))]
pub struct StructuredPage {
    /// Zero-based page index.
    pub page_index: usize,
    /// Page width in PDF points.
    pub page_width: f32,
    /// Page height in PDF points.
    pub page_height: f32,
    /// Regions in reading order (column-by-column per ISO 32000-1 §14.8.2.3.1).
    pub regions: Vec<StructuredRegion>,
}

/// A contiguous run of same-role spans, optionally tagged with a column index.
#[derive(Debug, Clone, serde::Serialize)]
#[cfg_attr(feature = "wasm", serde(rename_all = "camelCase"))]
pub struct StructuredRegion {
    /// The semantic role of this region.
    pub kind: RegionRole,
    /// The region's text (spans joined with single spaces / newlines).
    pub text: String,
    /// Union bounding box of the region's spans.
    pub bbox: Rect,
    /// The underlying spans that make up this region.
    pub spans: Vec<TextSpan>,
    /// Column index for multi-column bodies: `Some(0)` = leftmost column,
    /// `Some(1)` = next column, … `None` for full-width content, headings,
    /// or chrome.
    pub column_index: Option<usize>,
    /// Logical section index from the nearest `Sect`/`Art`/`Part` structure
    /// ancestor (ISO 32000-1:2008 §14.8.4.2) — the spec-authoritative,
    /// page-independent grouping, so a chapter that continues across pages
    /// keeps one `section_id` (#734 §5/§6). `None` for untagged PDFs or content
    /// outside any section.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub section_id: Option<usize>,
}

/// Per-MCID structure-tree facts that `extract_structured` surfaces into region
/// roles (ISO 32000-1:2008 §14.8.4) — empty for untagged PDFs. Keyed by MCID
/// (page-scoped, which is unique within a single page's structured extraction).
#[derive(Debug, Default, Clone)]
pub(crate) struct McidStructInfo {
    /// MCIDs whose nearest structure ancestor is `Lbl` (§14.8.4.3.3) — a
    /// label/numeral that distinguishes an item (a verse / list number).
    pub lbl: std::collections::HashSet<u32>,
    /// MCID → logical section index (nearest `Sect`/`Art`/`Part` ancestor).
    pub section: std::collections::HashMap<u32, usize>,
}

/// The semantic role of a [`StructuredRegion`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[cfg_attr(feature = "wasm", serde(rename_all = "camelCase"))]
pub enum RegionRole {
    /// Ordinary body text.
    BodyBlock,
    /// A document heading (ISO 32000-1 §14.7.2 `H1`..`H6`).
    StructuralHeading {
        /// Heading level, 1–6.
        level: u8,
    },
    /// A short verse / section numeral sitting in a narrow column indent.
    MarginalLabel,
    /// Running header (§14.8.2.2 Pagination / Header).
    Header,
    /// Running footer (§14.8.2.2 Pagination / Footer).
    Footer,
    /// Page-number folio (§14.8.2.2 Pagination / page number).
    PageNumber,
    /// Any other artifact (watermark, layout, background; §14.8.2.2).
    Artifact,
}

/// Map a span's `artifact_type` / `heading_level` to a [`RegionRole`].
fn role_for_span(span: &TextSpan) -> RegionRole {
    if let Some(at) = &span.artifact_type {
        return match at {
            ArtifactType::Pagination(PaginationSubtype::Header) => RegionRole::Header,
            ArtifactType::Pagination(PaginationSubtype::Footer) => RegionRole::Footer,
            ArtifactType::Pagination(PaginationSubtype::PageNumber) => RegionRole::PageNumber,
            // Watermark / Other pagination, plus Layout / Page / Background.
            _ => RegionRole::Artifact,
        };
    }
    if let Some(level) = span.heading_level {
        return RegionRole::StructuralHeading { level };
    }
    if is_marginal_label(&span.text) {
        return RegionRole::MarginalLabel;
    }
    RegionRole::BodyBlock
}

/// A conservative marginal-label test: a short, standalone numeric or
/// lowercase-roman token (a verse / section numeral). When unsure we return
/// `false` so the span folds into the adjacent body block — reading order is
/// correct either way.
fn is_marginal_label(text: &str) -> bool {
    let t = text.trim();
    if t.is_empty() || t.chars().count() > 4 {
        return false;
    }
    let is_arabic = t.chars().all(|c| c.is_ascii_digit());
    let is_roman = !t.is_empty()
        && t.chars()
            .all(|c| matches!(c, 'i' | 'v' | 'x' | 'l' | 'c' | 'd' | 'm'));
    is_arabic || is_roman
}

/// Union of two rectangles (corner-based).
fn rect_union(a: &Rect, b: &Rect) -> Rect {
    let x0 = a.x.min(b.x);
    let y0 = a.y.min(b.y);
    let x1 = (a.x + a.width).max(b.x + b.width);
    let y1 = (a.y + a.height).max(b.y + b.height);
    Rect::new(x0, y0, x1 - x0, y1 - y0)
}

/// A column-body line spans at most ≈ half the content width (one column minus
/// the gutter); anything wider is cross-column chrome. The threshold is set a
/// little above half to tolerate uneven columns and ragged right edges.
const COLUMN_BRIDGE_FRACTION: f32 = 0.6;

/// Best-effort single-gutter detector for body spans.
///
/// Returns the gutter X (page coordinate) when the body spans split into two
/// columns separated by a vertical whitespace corridor that **no span crosses**,
/// else `None`. Conservative by design: a page with no clear two-column body
/// yields `None` and every body region gets `column_index == None`.
///
/// Detection is **edge-based** (a valley in the horizontal projection of span
/// extents), not center-of-mass based. A span-center histogram collapses on the
/// layouts that need column routing most — short, ragged lines (Bible verses,
/// reference editions) and word-level spans pack centres densely across each
/// column, leaving no single wide centre-gap even though the columns are
/// visually obvious. The empty corridor between the left column's right edge and
/// the right column's left edge survives all of that: inter-word gaps inside a
/// line are crossed by other lines at different y, so only a true column gutter
/// forms a page-spanning empty band. Because a real gutter can be narrow
/// (≈8–20pt) the width gate is an absolute minimum, not a fraction of the page —
/// the "no span crosses it" + "substantial body on both sides" + "near the page
/// middle" conditions are what guard against false positives.
///
/// Spans wider than [`COLUMN_BRIDGE_FRACTION`] of the body content width are
/// dropped before the sweep: a full-width book title, running head, or
/// horizontal rule literally spans both columns and the gutter, so its extent
/// bridges the corridor in the 1-D projection and hides an otherwise obvious
/// split (issue #734 — KJF reference Bible, where the merged book-title line
/// `"Le Troisième Livre de Moïse Appelé GENÈSE"` collapsed the gutter to
/// nothing). A column-body line never exceeds roughly half the content width;
/// only cross-column chrome does, so excluding it is safe and is what lets the
/// edge corridor survive for short, ragged verse columns.
fn detect_gutter_x(body: &[&TextSpan], page_width: f32) -> Option<f32> {
    /// A column gutter is a true empty channel; ordinary inter-word/-glyph gaps
    /// are both narrower and crossed by other lines, so they never survive.
    const MIN_GUTTER_PT: f32 = 8.0;
    /// Each side must hold at least a small line or two — a single off-margin
    /// token is not a column. The empty-corridor + near-middle gates do the real
    /// false-positive rejection.
    const MIN_SIDE_SPANS: usize = 2;

    if body.len() < 4 || page_width <= 0.0 {
        return None;
    }
    // Horizontal extents (left, right) of every finite, non-empty body span.
    let all_extents: Vec<(f32, f32)> = body
        .iter()
        .filter(|s| {
            s.bbox.width > 0.0
                && s.bbox.x.is_finite()
                && s.bbox.width.is_finite()
                && !s.text.trim().is_empty()
        })
        .map(|s| (s.bbox.x, s.bbox.x + s.bbox.width))
        .collect();
    if all_extents.len() < 4 {
        return None;
    }

    let content_min = all_extents
        .iter()
        .map(|b| b.0)
        .fold(f32::INFINITY, f32::min);
    let content_max = all_extents
        .iter()
        .map(|b| b.1)
        .fold(f32::NEG_INFINITY, f32::max);
    let content_w = content_max - content_min;
    if content_w < page_width * 0.25 {
        return None; // body too narrow to hold two columns
    }

    // Drop cross-column chrome (full-width titles/rules) so it cannot bridge the
    // gutter in projection; a real column-body line stays well under half-width.
    let bridge_w = content_w * COLUMN_BRIDGE_FRACTION;
    let mut boxes: Vec<(f32, f32)> = all_extents
        .into_iter()
        .filter(|(l, r)| r - l <= bridge_w)
        .collect();
    if boxes.len() < 4 {
        return None;
    }
    boxes.sort_by(|a, b| crate::utils::safe_float_cmp(a.0, b.0));

    // Sweep-merge the extents left-to-right; the widest forward jump between the
    // running right edge and the next span's left edge is the widest empty
    // corridor no span crosses. Sorted by left edge, every span entirely left of
    // a clean corridor precedes every span entirely right of it, so the span
    // index at the jump is the left-column count.
    let mut cover_right = boxes[0].1;
    let mut best_gap = 0.0_f32;
    let mut best_mid = 0.0_f32;
    let mut left_count = 0usize;
    for i in 1..boxes.len() {
        let gap = boxes[i].0 - cover_right;
        if gap > best_gap {
            best_gap = gap;
            best_mid = (cover_right + boxes[i].0) * 0.5;
            left_count = i;
        }
        cover_right = cover_right.max(boxes[i].1);
    }

    let rel = best_mid / page_width;
    let right_count = boxes.len() - left_count;
    if best_gap >= MIN_GUTTER_PT
        && (0.3..=0.7).contains(&rel)
        && left_count >= MIN_SIDE_SPANS
        && right_count >= MIN_SIDE_SPANS
    {
        Some(best_mid)
    } else {
        None
    }
}

/// Build a [`StructuredPage`] from reading-order spans + page dimensions.
/// Column-detection mode for `build_structured_page_with_mode` (issue #734
/// Fix 3). The geometric column split is heuristic — ISO 32000-1:2008 defines
/// no reading order for untagged content (§14.8.2.3) — so a consumer who knows
/// their layout may override the heuristic. This override applies only to the
/// geometric (Tier-3) path; it never overrides a trustworthy structure tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "wasm", derive(serde::Serialize, serde::Deserialize))]
pub enum ColumnMode {
    /// Detect two-column bodies heuristically (the default).
    #[default]
    Auto,
    /// Force a two-column split: use the detected gutter, or the page midpoint
    /// when no clean gutter is found. For consumers who know the page is
    /// two-column (e.g. reference editions whose short, ragged lines the
    /// heuristic is conservative about).
    Two,
    /// Treat the whole page as a single column — suppress all column detection
    /// (`column_index` stays `None`).
    Single,
}

/// Build a [`StructuredPage`], detecting columns per [`ColumnMode::Auto`].
///
/// Test-only convenience over [`build_structured_page_full`]; production callers
/// go through [`crate::document::PdfDocument::extract_structured`], which
/// supplies tagged-structure facts.
#[cfg(test)]
pub(crate) fn build_structured_page(
    page_index: usize,
    page_width: f32,
    page_height: f32,
    spans: Vec<TextSpan>,
) -> StructuredPage {
    build_structured_page_full(
        page_index,
        page_width,
        page_height,
        spans,
        ColumnMode::Auto,
        &McidStructInfo::default(),
    )
}

/// Build a [`StructuredPage`] with an explicit column-detection [`ColumnMode`]
/// (issue #734 Fix 3). `Auto` runs the gutter heuristic; `Two` forces a split
/// (detected gutter, else page midpoint); `Single` suppresses columns.
///
/// Test-only convenience over [`build_structured_page_full`].
#[cfg(test)]
pub(crate) fn build_structured_page_with_mode(
    page_index: usize,
    page_width: f32,
    page_height: f32,
    spans: Vec<TextSpan>,
    column_mode: ColumnMode,
) -> StructuredPage {
    build_structured_page_full(
        page_index,
        page_width,
        page_height,
        spans,
        column_mode,
        &McidStructInfo::default(),
    )
}

/// Build a [`StructuredPage`] with both a column [`ColumnMode`] and tagged
/// structure facts ([`McidStructInfo`]): `Lbl` MCIDs become `MarginalLabel`
/// regions and each region carries its `Sect`/`Art`/`Part` section index
/// (ISO 32000-1:2008 §14.8.4 — issue #734 §4/§5/§6). For untagged PDFs the
/// info is empty and this behaves exactly like the geometric path.
pub(crate) fn build_structured_page_full(
    page_index: usize,
    page_width: f32,
    page_height: f32,
    spans: Vec<TextSpan>,
    column_mode: ColumnMode,
    struct_info: &McidStructInfo,
) -> StructuredPage {
    // Column assignment is computed over body spans only (chrome/headings are
    // full-width by convention).
    let body_refs: Vec<&TextSpan> = spans
        .iter()
        .filter(|s| matches!(role_for_span(s), RegionRole::BodyBlock | RegionRole::MarginalLabel))
        .collect();
    let gutter = match column_mode {
        ColumnMode::Auto => detect_gutter_x(&body_refs, page_width),
        // Forced two-column: prefer the detected gutter, else the page midpoint.
        ColumnMode::Two => detect_gutter_x(&body_refs, page_width).or(Some(page_width * 0.5)),
        // Forced single column: never split.
        ColumnMode::Single => None,
    };

    // Body content width over the same finite, non-empty spans the detector
    // uses, so a full-width title/rule (a gutter-bridging span) is assigned no
    // column rather than being forced to one side by its centre (#734).
    let (content_min, content_max) = body_refs
        .iter()
        .filter(|s| {
            s.bbox.width > 0.0
                && s.bbox.x.is_finite()
                && s.bbox.width.is_finite()
                && !s.text.trim().is_empty()
        })
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), s| {
            (lo.min(s.bbox.x), hi.max(s.bbox.x + s.bbox.width))
        });
    let bridge_w = (content_max - content_min) * COLUMN_BRIDGE_FRACTION;

    let column_of = |span: &TextSpan| -> Option<usize> {
        let g = gutter?;
        if span.bbox.width > bridge_w {
            return None; // cross-column chrome spans both columns
        }
        let center = span.bbox.x + span.bbox.width * 0.5;
        Some(if center < g { 0 } else { 1 })
    };

    let mut regions: Vec<StructuredRegion> = Vec::new();
    for span in spans {
        if span.text.trim().is_empty() {
            continue;
        }
        // Tagged-structure roles (ISO 32000-1:2008 §14.8.4) take precedence over
        // the geometric/textual heuristics: an `Lbl` marked-content sequence is
        // authoritatively a label/numeral (§14.8.4.3.3 — #734 §4).
        let mcid = span.mcid;
        let is_lbl = mcid.is_some_and(|m| struct_info.lbl.contains(&m));
        let kind = if is_lbl {
            RegionRole::MarginalLabel
        } else {
            role_for_span(&span)
        };
        let col = match kind {
            RegionRole::BodyBlock | RegionRole::MarginalLabel => column_of(&span),
            _ => None,
        };
        // Nearest Sect/Art/Part section (§14.8.4.2 — #734 §5/§6).
        let section = mcid.and_then(|m| struct_info.section.get(&m).copied());

        // Region grouping:
        //  * A column-tagged body span (`col` is `Some`) merges into the FIRST
        //    region of the same role + column (+ section) so a two-column page
        //    yields one region per column — left column whole, then right —
        //    instead of interleaved per-line regions (#734 Fix 1). The per-column
        //    spans arrive in reading (y) order, so each region's text is the
        //    column read top-to-bottom. A section change forces a new region so
        //    chapters stay separate.
        //  * Full-width / untagged content keeps adjacent-only coalescing, so
        //    distinct blocks (separate headings, paragraphs) stay separate.
        let merge_idx = if col.is_some() {
            regions
                .iter()
                .position(|r| r.kind == kind && r.column_index == col && r.section_id == section)
        } else {
            match regions.last() {
                Some(r) if r.kind == kind && r.column_index == col && r.section_id == section => {
                    Some(regions.len() - 1)
                },
                _ => None,
            }
        };
        if let Some(i) = merge_idx {
            let r = &mut regions[i];
            r.text.push(' ');
            r.text.push_str(span.text.trim());
            r.bbox = rect_union(&r.bbox, &span.bbox);
            r.spans.push(span);
            continue;
        }
        regions.push(StructuredRegion {
            kind,
            text: span.text.trim().to_string(),
            bbox: span.bbox,
            column_index: col,
            section_id: section,
            spans: vec![span],
        });
    }

    StructuredPage {
        page_index,
        page_width,
        page_height,
        regions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(text: &str, x: f32, y: f32, w: f32) -> TextSpan {
        TextSpan {
            text: text.to_string(),
            bbox: Rect::new(x, y, w, 12.0),
            ..Default::default()
        }
    }

    /// Body span carrying a marked-content id (one logical unit, e.g. a verse).
    fn span_mcid(text: &str, x: f32, y: f32, w: f32, mcid: u32) -> TextSpan {
        TextSpan {
            mcid: Some(mcid),
            ..span(text, x, y, w)
        }
    }

    /// Faithful reproduction of issue #734 — KJF reference-Bible page 11
    /// (Genesis 1), 432 pt wide. Left column = verses 1–13 at x≈57.6, right
    /// column = verses 14–25 at x≈230.5, with a ≈30 pt gutter. Distinguishing
    /// features that the existing word-level synthetic test does NOT have, taken
    /// from the issue's own span dump:
    ///   * spans are *line-level* (one wide span per wrapped verse line), and
    ///   * a *full-width* book-title line ("Le Troisième Livre…") and a
    ///     gutter-centred page number get merged into the body — either of
    ///     which bridges the gutter in a 1-D x-projection and defeats
    ///     `detect_gutter_x`.
    ///
    /// Per-verse MCIDs are present (verse 1 = mcid 5, verse 14 = mcid 18, …),
    /// which is the signal the spec-aligned fix uses to vote per logical unit.
    #[test]
    fn issue_734_kjf_reference_columns_get_indices() {
        let pw = 432.0;
        // Left column occupies x∈[57.6,197]; right column x∈[230.5,396];
        // gutter ≈ [197,230]. Line-level spans (≈140 pt wide left lines).
        let mut spans = Vec::new();
        // Full-width book title + running header merged into body flow.
        spans.push(span("Le Troisième Livre de Moïse Appelé GENÈSE", 57.6, 720.0, 339.0));
        // Gutter-centred page number that escaped artifact classification.
        spans.push(span("1", 224.6, 706.0, 8.0));

        let mut y = 690.0;
        // Left column: verses 1..=6, two wrapped line-spans each, per-verse MCID.
        for v in 1..=6u32 {
            spans.push(span_mcid(
                &format!("{v} Au commencement Dieu créa le ciel et la"),
                57.6,
                y,
                139.0,
                4 + v, // verse 1 → mcid 5
            ));
            spans.push(span_mcid("terre.", 57.6, y - 14.0, 28.0, 4 + v));
            y -= 30.0;
        }
        // Right column: verses 14..=19, two wrapped line-spans each, per-verse MCID.
        y = 690.0;
        for v in 14..=19u32 {
            spans.push(span_mcid(
                &format!("{v} ¶ Et Dieu dit, Qu'il y ait des lumières dans le"),
                230.5,
                y,
                165.0,
                4 + v, // verse 14 → mcid 18
            ));
            spans.push(span_mcid(
                "firmament du ciel pour séparer le jour.",
                230.5,
                y - 14.0,
                160.0,
                4 + v,
            ));
            y -= 30.0;
        }

        let page = build_structured_page(0, pw, 792.0, spans);
        let cols: Vec<Option<usize>> = page.regions.iter().map(|r| r.column_index).collect();
        assert!(
            cols.contains(&Some(0)),
            "left column (0) must be assigned for the KJF layout: {cols:?}"
        );
        assert!(
            cols.contains(&Some(1)),
            "right column (1) must be assigned for the KJF layout: {cols:?}"
        );

        // The full-width book-title line spans both columns: it must carry no
        // column index, not be forced to one side by its centre.
        let title = page
            .regions
            .iter()
            .find(|r| r.text.contains("Troisième Livre"))
            .expect("title region present");
        assert_eq!(title.column_index, None, "gutter-bridging title must not be assigned a column");
    }

    #[test]
    fn marginal_label_detects_short_numerals() {
        assert!(is_marginal_label("12"));
        assert!(is_marginal_label("iv"));
        assert!(!is_marginal_label("Genesis"));
        assert!(!is_marginal_label("12345")); // too long
    }

    #[test]
    fn heading_and_body_roles_assigned() {
        let mut h = span("Title", 100.0, 700.0, 80.0);
        h.heading_level = Some(1);
        let b = span("Body text here", 100.0, 680.0, 120.0);
        let page = build_structured_page(0, 612.0, 792.0, vec![h, b]);
        assert_eq!(page.regions.len(), 2);
        assert_eq!(page.regions[0].kind, RegionRole::StructuralHeading { level: 1 });
        assert_eq!(page.regions[1].kind, RegionRole::BodyBlock);
    }

    #[test]
    fn two_column_body_gets_column_indices() {
        // Left column at x≈60, right column at x≈360 on a 612-wide page.
        let spans = vec![
            span("left one", 60.0, 700.0, 120.0),
            span("left two", 60.0, 680.0, 120.0),
            span("right one", 360.0, 700.0, 120.0),
            span("right two", 360.0, 680.0, 120.0),
        ];
        let page = build_structured_page(0, 612.0, 792.0, spans);
        let cols: Vec<Option<usize>> = page.regions.iter().map(|r| r.column_index).collect();
        assert!(cols.contains(&Some(0)), "a left column (0) must be assigned: {cols:?}");
        assert!(cols.contains(&Some(1)), "a right column (1) must be assigned: {cols:?}");
    }

    /// A reference-edition layout (Bible verses): two narrow columns with a
    /// narrow gutter, short ragged verse lines, word-level spans, and marginal
    /// verse numerals at each column's left edge. The old center-of-mass gap
    /// detector returned `None` here (the gutter is far under 12 % of the page
    /// width and word centres pack each column densely); the edge-based corridor
    /// detector must still split the columns.
    #[test]
    fn narrow_gutter_short_line_columns_get_indices() {
        // Page 432pt wide. Left column x∈[36,206], right column x∈[226,396],
        // gutter ≈ [206,226] (20pt, ~4.6 % of width). Word-level spans.
        let pw = 432.0;
        let mut spans = Vec::new();
        let mut y = 700.0;
        for row in 0..6 {
            // Left column: a marginal verse numeral then two short words.
            spans.push(span(&format!("{}", row + 1), 36.0, y, 8.0));
            spans.push(span("Au", 52.0, y, 26.0));
            spans.push(span("commencement", 84.0, y, 110.0));
            // Right column: marginal numeral then two short words.
            spans.push(span(&format!("{}", row + 14), 226.0, y, 12.0));
            spans.push(span("Et", 244.0, y, 22.0));
            spans.push(span("Dieu", 272.0, y, 40.0));
            y -= 14.0;
        }
        let page = build_structured_page(0, pw, 792.0, spans);
        let cols: Vec<Option<usize>> = page.regions.iter().map(|r| r.column_index).collect();
        assert!(cols.contains(&Some(0)), "left column (0) not assigned: {cols:?}");
        assert!(cols.contains(&Some(1)), "right column (1) not assigned: {cols:?}");
    }

    /// `ColumnMode::Two` forces a two-column split even on a layout the
    /// conservative `Auto` heuristic rejects (here: too few spans to detect a
    /// gutter). The split falls back to the page midpoint (#734 Fix 3).
    #[test]
    fn column_mode_two_forces_split_when_auto_rejects() {
        let pw = 432.0;
        // Two spans only — below the detector's `body.len() < 4` floor, so Auto
        // yields no gutter; Two must still split at the page midpoint (216).
        let spans = vec![
            span("left body", 40.0, 700.0, 100.0), // centre 90  < 216 → col 0
            span("right body", 250.0, 700.0, 100.0), // centre 300 > 216 → col 1
        ];

        let auto = build_structured_page_with_mode(0, pw, 792.0, spans.clone(), ColumnMode::Auto);
        assert!(
            auto.regions.iter().all(|r| r.column_index.is_none()),
            "Auto must not split this sparse layout: {:?}",
            auto.regions
                .iter()
                .map(|r| r.column_index)
                .collect::<Vec<_>>()
        );

        let two = build_structured_page_with_mode(0, pw, 792.0, spans, ColumnMode::Two);
        let cols: Vec<Option<usize>> = two.regions.iter().map(|r| r.column_index).collect();
        assert!(cols.contains(&Some(0)), "Two must assign a left column: {cols:?}");
        assert!(cols.contains(&Some(1)), "Two must assign a right column: {cols:?}");
    }

    /// `ColumnMode::Single` suppresses column detection on a layout `Auto` would
    /// split — every region gets `column_index == None` (#734 Fix 3).
    #[test]
    fn column_mode_single_suppresses_clear_columns() {
        let pw = 612.0;
        let spans = vec![
            span("left one", 60.0, 700.0, 120.0),
            span("left two", 60.0, 680.0, 120.0),
            span("right one", 360.0, 700.0, 120.0),
            span("right two", 360.0, 680.0, 120.0),
        ];
        // Sanity: Auto splits this layout (mirrors two_column_body_gets_column_indices).
        let auto = build_structured_page_with_mode(0, pw, 792.0, spans.clone(), ColumnMode::Auto);
        assert!(auto.regions.iter().any(|r| r.column_index == Some(1)));

        let single = build_structured_page_with_mode(0, pw, 792.0, spans, ColumnMode::Single);
        assert!(
            single.regions.iter().all(|r| r.column_index.is_none()),
            "Single must suppress all columns: {:?}",
            single
                .regions
                .iter()
                .map(|r| r.column_index)
                .collect::<Vec<_>>()
        );
    }

    /// A single-column page of ordinary prose (lines spanning most of the body
    /// width at varying right edges) must NOT be split into columns — there is
    /// no empty corridor for any line to leave clear.
    #[test]
    fn single_column_prose_has_no_gutter() {
        let pw = 612.0;
        let mut spans = Vec::new();
        let mut y = 700.0;
        let widths = [430.0, 460.0, 410.0, 470.0, 440.0, 455.0, 425.0, 465.0];
        for w in widths {
            spans.push(span("a single column prose line of body text", 80.0, y, w));
            y -= 14.0;
        }
        let page = build_structured_page(0, pw, 792.0, spans);
        let cols: Vec<Option<usize>> = page.regions.iter().map(|r| r.column_index).collect();
        assert!(
            cols.iter().all(|c| c.is_none()),
            "single-column prose wrongly split into columns: {cols:?}"
        );
    }
}
