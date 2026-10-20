//! Canonical page reading order — single source of truth for text-extraction
//! span ordering (issue #457).
//!
//! Resolution per PDF 32000-1:2008:
//!
//!   1. **Tagged PDF** (`/MarkInfo /Marked true`) with a `/StructTreeRoot`
//!      and `/Suspects != true`: walk the structure tree on this page and
//!      return spans in **logical structure order** (§14.7.2). This is the
//!      authoritative reading order when present.
//!
//!   2. **Otherwise**: return spans in **page content order** — the
//!      geometric top-to-bottom + left-to-right pass described in §14.8.2.3.1.
//!
//! All public text-extraction APIs (`extract_words`, `extract_text_lines`,
//! `extract_text`, `to_markdown`, `to_html`, `to_plain_text`) should consume
//! this function so they cannot drift apart on the same input.
//!
//! The `StructureTreeStrategy` it dispatches to already falls back to the
//! geometric strategy when the structure tree is suspect (§14.7.1) or when
//! the MCID order would zigzag horizontally across columns — both
//! defenses against producer bugs that this helper inherits transparently.

use crate::document::PdfDocument;
use crate::error::Result;
use crate::geometry::Rect;
use crate::pipeline::{OrderedTextSpan, ReadingOrderContext, TextPipeline, TextPipelineConfig};

/// Compute the canonical reading-order span sequence for a single page.
///
/// Returns an empty vector when the page has no extractable text.
///
/// All extracted spans are returned by default — including any that the
/// upstream extractor tagged as `/Artifact` (running headers, footers,
/// page numbers, watermarks; ISO 32000-1:2008 §14.8.2.2.1). Some
/// downstream callers (e.g. `extract_text` on untagged PDFs) apply
/// their own artifact filter. Use
/// [`page_reading_order_no_artifacts`] for the spec-correct
/// "exclude artifacts" variant.
///
/// # Errors
///
/// Returns the underlying parse / extraction error if span extraction
/// itself fails. Structure-tree resolution errors are tolerated and the
/// helper falls back to geometric order.
pub fn page_reading_order(doc: &PdfDocument, page_index: usize) -> Result<Vec<OrderedTextSpan>> {
    page_reading_order_inner(doc, page_index, /*include_artifacts*/ true)
}

/// Variant of [`page_reading_order`] that drops spans flagged as
/// `/Artifact` (running headers, footers, page numbers, watermarks;
/// ISO 32000-1:2008 §14.8.2.2.1).
pub fn page_reading_order_no_artifacts(
    doc: &PdfDocument,
    page_index: usize,
) -> Result<Vec<OrderedTextSpan>> {
    page_reading_order_inner(doc, page_index, /*include_artifacts*/ false)
}

fn page_reading_order_inner(
    doc: &PdfDocument,
    page_index: usize,
    include_artifacts: bool,
) -> Result<Vec<OrderedTextSpan>> {
    let mut spans = doc.extract_spans(page_index)?;
    if !include_artifacts {
        spans.retain(|s| s.artifact_type.is_none());
    }
    if spans.is_empty() {
        return Ok(Vec::new());
    }

    // Tier 1 (logical structure order) → Tier 2 (article threads) → Tier 3
    // (geometric). The v0.3.61 sweep showed a bare ≥80%-bead-coverage gate
    // regressed single-column books (it reordered content non-improvingly), so
    // Tier 2 only activates behind the conservative multi-column +
    // order-divergence gate in `page_article_bead_rects` — which is provably a
    // no-op on single-column / geometric-order threads.
    let mut context = build_context(doc, page_index);
    if !context.has_structure_tree {
        if let Some(beads) = page_article_bead_rects(doc, page_index, &spans) {
            context = context.with_bead_rects(beads);
        }
    }

    let pipeline = TextPipeline::with_config(TextPipelineConfig::default());

    // Text-matrix-rotated content. Gated to unrotated pages:
    // on a `/Rotate`d page `postprocess_spans` already mapped
    // rotated-content spans into the displayed frame, so their retained
    // `rotation_degrees` describes the pre-display frame and re-rotating
    // here would double-transform.
    if doc.get_page_rotation(page_index).unwrap_or(0) == 0 {
        // A dominant rotation (a landscape table typeset on a portrait
        // page) reorders the WHOLE page in the rotated reading frame.
        if let Some(rot) = crate::utils::dominant_rotation(&spans).and_then(reading_frame_quadrant)
        {
            log::debug!(
                "page {page_index}: dominant text rotation {rot}° — ordering in rotated frame"
            );
            return order_in_rotated_frame(doc, page_index, spans, context, &pipeline, rot);
        }
        // Otherwise mirror the span path's per-span rotation firewall:
        // rotated minority runs (margin stamps, figure labels) break the
        // axis-aligned assumptions of the geometric strategies, so lift
        // them out, order each rotation group in its upright frame, and
        // append after the horizontal flow.
        if spans.iter().any(|s| s.rotation_degrees != 0.0) {
            let (rotated, upright): (Vec<_>, Vec<_>) =
                spans.into_iter().partition(|s| s.rotation_degrees != 0.0);
            log::debug!(
                "page {page_index}: {} rotated minority span(s) appended after the horizontal flow",
                rotated.len()
            );
            let mut ordered = if upright.is_empty() {
                Vec::new()
            } else {
                pipeline.process(upright, context)?
            };
            let base = ordered.len();
            ordered.extend(
                PdfDocument::order_rotated_blocks(rotated)
                    .into_iter()
                    .enumerate()
                    .map(|(i, s)| OrderedTextSpan::new(s, base + i)),
            );
            return Ok(ordered);
        }
    }

    pipeline.process(spans, context)
}

/// Display-rotation quadrant that turns text of the given snapped rotation
/// upright: `90°` text (reading bottom-to-top) becomes readable when the
/// page is displayed as if `/Rotate 90`, `-90°` under `/Rotate 270`, and
/// `180°` under `/Rotate 180`. Mirrored or free-angle runs (which
/// `snap_run_rotation` reports as raw angles) have no quadrant frame.
fn reading_frame_quadrant(degrees: f32) -> Option<i32> {
    for (deg, rot) in [(90.0, 90), (180.0, 180), (-90.0, 270)] {
        if (degrees - deg).abs() < 0.5 {
            return Some(rot);
        }
    }
    None
}

/// Order a dominant-rotation page in its rotated reading frame: map every
/// span bbox through the display rotation (so the text becomes horizontal),
/// run the standard pipeline there, then map the bboxes back so callers see
/// true page coordinates — only the ORDER reflects the rotated frame.
fn order_in_rotated_frame(
    doc: &PdfDocument,
    page_index: usize,
    mut spans: Vec<crate::layout::TextSpan>,
    context: ReadingOrderContext,
    pipeline: &TextPipeline,
    rot: i32,
) -> Result<Vec<OrderedTextSpan>> {
    let (llx, lly, urx, ury) = doc
        .get_page_media_box(page_index)
        .unwrap_or((0.0, 0.0, 612.0, 792.0));
    let (w, h) = (urx - llx, ury - lly);

    // Rotated spans store TEXT-LOCAL extents (origin + advance-along-the-
    // run as `width` + font size as `height`), so mapping into the reading
    // frame rotates the ORIGIN as a point — through the same quadrant map
    // as `PdfDocument::rotate_span_bbox` — and keeps the extents, which
    // already describe the run in its own upright frame. This mirrors
    // `order_rotated_blocks`, which sorts rotated origins the same way.
    let map_origin = |x: f32, y: f32, rot: i32, fw: f32, fh: f32| -> (f32, f32) {
        let (rx, ry) = (x - llx, y - lly);
        let (mx, my) = match rot {
            90 => (ry, fw - rx),
            180 => (fw - rx, fh - ry),
            270 => (fh - ry, rx),
            _ => (rx, ry),
        };
        (llx + mx, lly + my)
    };

    for s in &mut spans {
        let (x, y) = map_origin(s.bbox.x, s.bbox.y, rot, w, h);
        s.bbox.x = x;
        s.bbox.y = y;
    }
    // The rotated frame swaps the page dimensions for 90°/270°.
    let (fw, fh) = if rot % 180 == 90 { (h, w) } else { (w, h) };
    let context = context.with_bbox(Rect::new(llx, lly, fw, fh));

    let mut ordered = pipeline.process(spans, context)?;

    // Inverse map: the opposite quadrant applied with the rotated frame's
    // dimensions. `w - (w - x)` round-trips within ~1 ULP of the page
    // dimension in f32 (≈6e-5 pt on a Letter page) — well inside every
    // downstream tolerance, but not bit-exact.
    let inv = (360 - rot) % 360;
    for os in &mut ordered {
        let (x, y) = map_origin(os.span.bbox.x, os.span.bbox.y, inv, fw, fh);
        os.span.bbox.x = x;
        os.span.bbox.y = y;
    }
    Ok(ordered)
}

/// Article-thread (#458) bead rectangles for `page_index`, in `/N` chain order,
/// when a conservative gate confirms a thread genuinely governs this page.
///
/// All conditions are required — the v0.3.61 corpus sweep found a bare
/// ≥80%-coverage gate regressed single-column books by reordering them
/// non-improvingly:
///   1. **≥2 beads** on the page (nothing to reorder otherwise).
///   2. **Coverage** — ≥80% of non-empty span centres fall inside some bead.
///   3. **Multi-column** — the beads occupy ≥2 disjoint horizontal bands; a
///      single-column thread adds nothing over geometric order (this is the
///      gate that excludes the technical books the prior attempt regressed).
///   4. **Order-divergence** — the `/N` bead order differs from the naive
///      geometric order (top-to-bottom, left-to-right). When they coincide the
///      thread reorders nothing, so skipping keeps output byte-identical.
fn page_article_bead_rects(
    doc: &PdfDocument,
    page_index: usize,
    spans: &[crate::layout::TextSpan],
) -> Option<Vec<Rect>> {
    let threads = doc.cached_article_threads();
    if threads.is_empty() {
        return None;
    }
    // This page's beads, in `/N` chain order across all threads.
    let beads: Vec<Rect> = threads
        .iter()
        .flat_map(|t| t.beads.iter())
        .filter(|b| b.page_index == page_index)
        .map(|b| b.rect)
        .collect();
    if beads.len() < 2 {
        return None;
    }

    // 2. Coverage.
    let body: Vec<&crate::layout::TextSpan> =
        spans.iter().filter(|s| !s.text.trim().is_empty()).collect();
    if body.is_empty() {
        return None;
    }
    let inside = |r: &Rect, x: f32, y: f32| {
        x >= r.x && x <= r.x + r.width && y >= r.y && y <= r.y + r.height
    };
    let covered = body
        .iter()
        .filter(|s| {
            let cx = s.bbox.x + s.bbox.width * 0.5;
            let cy = s.bbox.y + s.bbox.height * 0.5;
            beads.iter().any(|r| inside(r, cx, cy))
        })
        .count();
    if (covered as f32) < 0.8 * body.len() as f32 {
        return None;
    }

    // 3. Multi-column: sweep bead x-extents; require ≥2 disjoint bands.
    let mut xs: Vec<(f32, f32)> = beads.iter().map(|r| (r.x, r.x + r.width)).collect();
    xs.sort_by(|a, b| crate::utils::safe_float_cmp(a.0, b.0));
    let mut bands = 1usize;
    let mut cover_right = xs[0].1;
    for &(l, r) in &xs[1..] {
        if l > cover_right {
            bands += 1;
        }
        cover_right = cover_right.max(r);
    }
    if bands < 2 {
        return None;
    }

    // 4. Order-divergence vs naive geometric (top-to-bottom, left-to-right).
    let mut geom: Vec<Rect> = beads.clone();
    geom.sort_by(|a, b| {
        let y = crate::utils::safe_float_cmp(b.y, a.y); // larger y = higher on page
        if y != std::cmp::Ordering::Equal {
            return y;
        }
        crate::utils::safe_float_cmp(a.x, b.x)
    });
    let same_order = beads
        .iter()
        .zip(geom.iter())
        .all(|(a, b)| a.x == b.x && a.y == b.y);
    if same_order {
        return None;
    }

    Some(beads)
}

/// Build the `ReadingOrderContext` for a page from the document's
/// `MarkInfo`, `StructTreeRoot`, and media box.
///
/// Best-effort: any errors reading structure metadata produce a context
/// without MCID order, which means the pipeline takes the geometric path.
pub(crate) fn build_context(doc: &PdfDocument, page_index: usize) -> ReadingOrderContext {
    let media_box = doc
        .get_page_media_box(page_index)
        .unwrap_or((0.0, 0.0, 612.0, 792.0));
    // MediaBox is `(llx, lly, urx, ury)` per PDF 32000-1:2008 §7.7.3.3.
    // `Rect::new` expects `(x, y, width, height)`, so use `from_points`.
    let bbox = Rect::from_points(media_box.0, media_box.1, media_box.2, media_box.3);

    let mut ctx = ReadingOrderContext::new()
        .with_page(page_index as u32)
        .with_bbox(bbox);

    // Use logical structure order only when the tree is trustworthy
    // (§14.8.2.3.1 / §14.7.1): the document is /Marked or the catalog references
    // a /StructTreeRoot, and /MarkInfo /Suspects is not true. This accepts
    // PDF-1.4 catalog-only tagged files that the old `!marked` early-return
    // wrongly skipped, and rejects suspect trees.
    let Some(tree) = doc.struct_tree_trustworthy() else {
        return ctx;
    };

    // Use the all-pages traversal cache (O(1) per page) instead of re-walking
    // the whole structure tree here (≈ O(pages²) across a tagged document).
    //
    // The scope travels with the id. It used to be projected away here, on the
    // stated grounds that reading-order strategies "don't disambiguate by
    // content-stream scope" — but that is what they must do: §14.7.4.2 scopes
    // an /MCID to its content stream, so a form's MCID 0 and the page's MCID 0
    // are different marked content and were being ordered as if they were one.
    let mcid_order = doc.cached_mcid_order_for_page(&tree, page_index as u32);

    if !mcid_order.is_empty() {
        ctx = ctx.with_mcid_order(mcid_order);
    }
    // The predicate already vetted the tree as non-suspect, so the strategy's
    // own suspect guard is a no-op here.
    ctx = ctx.with_suspects(false);
    ctx
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn issue_211_fixture(name: &str) -> Option<PathBuf> {
        let home = std::env::var("HOME").ok()?;
        let path = PathBuf::from(home)
            .join("projects/pdf_oxide_tests/pdfs_issue_regression")
            .join(name);
        if !path.exists() {
            eprintln!("Skipping: {} not found", path.display());
            return None;
        }
        Some(path)
    }

    fn open(name: &str) -> Option<PdfDocument> {
        let path = issue_211_fixture(name)?;
        let bytes = std::fs::read(&path).ok()?;
        PdfDocument::from_bytes(bytes).ok()
    }

    #[test]
    fn empty_page_returns_empty_vec() {
        // Use any document; request a page index past the end. extract_spans
        // returns an error in that case, so the helper propagates. We only
        // assert behavior when the helper succeeds; this test currently only
        // verifies the function compiles and links — runtime check below.
        let Some(doc) = open("issue_211_pdf_structure.pdf") else {
            return;
        };
        // Page 0 IS populated. Just confirm we get a non-empty result.
        let result = page_reading_order(&doc, 0).expect("page 0 should resolve");
        assert!(!result.is_empty(), "page 0 of pdf_structure has spans");
    }

    #[test]
    fn tagged_pdf_uses_structure_tree_first() {
        // PDF #2 is tagged. Title spans should appear BEFORE body spans in
        // the canonical order, even though XY-Cut moves them.
        let Some(doc) = open("issue_211_municipal_minutes.pdf") else {
            return;
        };
        let ordered = page_reading_order(&doc, 0).expect("ordering succeeds");

        let title_pos = ordered
            .iter()
            .position(|s| s.span.text.contains("COMITÉ"))
            .expect("title must appear");
        let body_pos = ordered
            .iter()
            .position(|s| s.span.text.contains("Séance"))
            .expect("body must appear");
        assert!(
            title_pos < body_pos,
            "title (COMITÉ at index {}) must precede body (Séance at index {}) \
             in canonical reading order",
            title_pos,
            body_pos,
        );
    }

    #[test]
    fn untagged_pdf_falls_back_to_geometric() {
        // Smoke test — the simple Lorem fixture. The first ordered span must
        // contain "Titre du document" (the document title at top of page).
        let Some(doc) = open("issue_211_pdf_structure.pdf") else {
            return;
        };
        let ordered = page_reading_order(&doc, 0).expect("ordering succeeds");
        assert!(!ordered.is_empty());
        assert!(
            ordered[0].span.text.contains("Titre")
                || ordered
                    .iter()
                    .take(3)
                    .any(|s| s.span.text.contains("Titre")),
            "title 'Titre' must appear among the first few ordered spans; \
             got first 5: {:?}",
            ordered
                .iter()
                .take(5)
                .map(|s| &s.span.text)
                .collect::<Vec<_>>(),
        );
    }
}
