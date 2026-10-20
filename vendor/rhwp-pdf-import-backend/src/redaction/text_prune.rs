//! Text-run pruning for destructive redaction (#231, T5 — guarantees
//! G1 "no recoverable text" and G2 "no width/shift side channel").
//!
//! A `Tj`/`TJ`/`'`/`"` show is a sequence of glyphs. To redact text we
//! must (feature plan §4.3):
//!
//! 1. Drop **every** glyph whose box touches a redaction region — the
//!    conservative "any overlap ⇒ remove" rule (never under-redact).
//! 2. Re-emit each maximal run of *surviving* glyphs with a **fresh
//!    absolute** text matrix and **no inter-glyph offsets**. Discarding
//!    the original `TJ` deltas is what kills the Bland et al. (PETS 2023)
//!    width/shift side channel: a positional delta can otherwise encode
//!    the count/advance of removed neighbours (G2).
//! 3. Record the removed glyph codes so `font_scrub` can strip their
//!    `/Widths` / `/ToUnicode` entries later.
//!
//! This module owns *only* that pure segmentation algorithm. Extracting
//! real per-glyph boxes from a content stream (sharing the metric path
//! with `extractors::text`) and re-serializing is the integration step
//! (plan T4/T11); here the input is the abstract [`Glyph`] so the
//! security-critical logic is exhaustively unit-testable in isolation and
//! cannot itself under-redact (it makes no I/O and is not yet wired in).

use std::collections::HashSet;

use super::region::RegionSet;
use crate::geometry::Rect;

/// One shown glyph, already mapped to page space by the caller.
#[derive(Debug, Clone, PartialEq)]
pub struct Glyph {
    /// Encoded bytes of this glyph within its show string (for re-emit).
    pub bytes: Vec<u8>,
    /// Page-space box (ink ∪ advance) used for the overlap test.
    pub bbox: Rect,
    /// Absolute text-rendering matrix `[a,b,c,d,e,f]` at this glyph — the
    /// `Tm` to emit when a surviving run *starts* here (absolute
    /// re-anchoring, G2).
    pub render_matrix: [f32; 6],
    /// `(resource_id, glyph_code)` — recorded into the removed set when
    /// this glyph is pruned, so fonts can be scrubbed (G2).
    pub code: (u32, u32),
}

/// A surviving run, re-anchored absolutely with no inter-glyph offsets.
#[derive(Debug, Clone, PartialEq)]
pub struct PrunedRun {
    /// Absolute text matrix to emit (`Tm`) before this run's bytes.
    pub anchor: [f32; 6],
    /// Concatenated surviving glyph bytes (emit as a single `Tj`, no
    /// `TJ` deltas — kills the width/shift side channel).
    pub bytes: Vec<u8>,
}

/// Result of pruning one show.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TextPruneResult {
    /// Surviving runs in document order. Empty ⇒ the whole show was
    /// inside a region and is emitted as nothing (no compensating
    /// offset).
    pub runs: Vec<PrunedRun>,
    /// Distinct `(resource_id, glyph_code)` pairs removed (deduped, in
    /// first-seen order) for `font_scrub`.
    pub removed_codes: Vec<(u32, u32)>,
    /// Total glyphs physically removed.
    pub glyphs_removed: usize,
}

/// Prune one glyph sequence against the page's regions.
///
/// Conservative (feature plan §4.3 / §9 risk 6): a glyph is removed if
/// its box intersects *any* edge-padded region — never "only if the
/// centre is inside". Surviving glyphs are grouped into maximal runs,
/// each re-anchored at its first glyph's absolute `render_matrix`; the
/// original inter-glyph spacing is intentionally discarded so no
/// positional delta can encode a removed glyph (G2).
pub fn prune_run(glyphs: &[Glyph], regions: &RegionSet, min_padding: f32) -> TextPruneResult {
    let mut out = TextPruneResult::default();
    let mut cur: Option<PrunedRun> = None;
    // O(1) membership for the dedup; the Vec still carries the public
    // first-seen order (Copilot review, PR #512 — avoids O(n²)).
    let mut seen_codes: HashSet<(u32, u32)> = HashSet::new();

    for g in glyphs {
        let removed = regions.any_intersects(&g.bbox, min_padding);
        if removed {
            // Close any open surviving run (each run is independently,
            // absolutely anchored — no offset bridges the gap).
            if let Some(run) = cur.take() {
                out.runs.push(run);
            }
            out.glyphs_removed += 1;
            if seen_codes.insert(g.code) {
                out.removed_codes.push(g.code);
            }
        } else {
            match &mut cur {
                Some(run) => run.bytes.extend_from_slice(&g.bytes),
                None => {
                    cur = Some(PrunedRun {
                        anchor: g.render_matrix,
                        bytes: g.bytes.clone(),
                    });
                },
            }
        }
    }
    if let Some(run) = cur.take() {
        out.runs.push(run);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::redaction::region::{RedactionRegion, RegionSet, DEFAULT_EDGE_PADDING};

    fn glyph(b: &[u8], x0: f32, y0: f32, x1: f32, y1: f32, code: (u32, u32)) -> Glyph {
        Glyph {
            bytes: b.to_vec(),
            bbox: Rect::from_points(x0, y0, x1, y1),
            render_matrix: [1.0, 0.0, 0.0, 1.0, x0, y0],
            code,
        }
    }

    fn region_at(x0: f32, y0: f32, x1: f32, y1: f32) -> RegionSet {
        let mut rs = RegionSet::new(0);
        rs.push(RedactionRegion::from_rect(x0, y0, x1, y1, None));
        rs
    }

    #[test]
    fn all_outside_is_one_intact_run() {
        let glyphs = vec![
            glyph(b"H", 0.0, 0.0, 10.0, 12.0, (0, 1)),
            glyph(b"i", 10.0, 0.0, 16.0, 12.0, (0, 2)),
        ];
        let r = region_at(500.0, 500.0, 600.0, 600.0);
        let out = prune_run(&glyphs, &r, DEFAULT_EDGE_PADDING);
        assert_eq!(out.glyphs_removed, 0);
        assert!(out.removed_codes.is_empty());
        assert_eq!(out.runs.len(), 1);
        assert_eq!(out.runs[0].bytes, b"Hi");
        assert_eq!(out.runs[0].anchor, [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    }

    #[test]
    fn all_inside_emits_nothing_no_compensating_offset() {
        let glyphs = vec![
            glyph(b"S", 10.0, 10.0, 20.0, 22.0, (0, 1)),
            glyph(b"E", 20.0, 10.0, 30.0, 22.0, (0, 2)),
            glyph(b"C", 30.0, 10.0, 40.0, 22.0, (0, 3)),
        ];
        let r = region_at(0.0, 0.0, 100.0, 100.0);
        let out = prune_run(&glyphs, &r, DEFAULT_EDGE_PADDING);
        assert_eq!(out.glyphs_removed, 3);
        assert_eq!(out.removed_codes, vec![(0, 1), (0, 2), (0, 3)]);
        assert!(out.runs.is_empty(), "no surviving runs, no offset emitted");
    }

    #[test]
    fn straddle_splits_into_two_absolutely_anchored_runs() {
        // "PUBLICsecret" — region covers x∈[60,200] (the "secret" part).
        let glyphs = vec![
            glyph(b"P", 0.0, 0.0, 10.0, 12.0, (0, 1)),
            glyph(b"U", 10.0, 0.0, 20.0, 12.0, (0, 2)),
            glyph(b"B", 20.0, 0.0, 30.0, 12.0, (0, 3)),
            glyph(b"s", 60.0, 0.0, 70.0, 12.0, (0, 9)),
            glyph(b"e", 70.0, 0.0, 80.0, 12.0, (0, 10)),
            // a survivor AFTER the redacted span (e.g. trailing public)
            glyph(b"X", 300.0, 0.0, 310.0, 12.0, (0, 20)),
        ];
        let r = region_at(55.0, -5.0, 200.0, 20.0);
        let out = prune_run(&glyphs, &r, DEFAULT_EDGE_PADDING);
        assert_eq!(out.glyphs_removed, 2);
        assert_eq!(out.removed_codes, vec![(0, 9), (0, 10)]);
        assert_eq!(out.runs.len(), 2);
        // first run = "PUB", anchored at glyph 'P' absolute position
        assert_eq!(out.runs[0].bytes, b"PUB");
        assert_eq!(out.runs[0].anchor, [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
        // second run = "X", independently anchored at its own absolute
        // position (no offset encodes the 2 removed glyphs — G2)
        assert_eq!(out.runs[1].bytes, b"X");
        assert_eq!(out.runs[1].anchor, [1.0, 0.0, 0.0, 1.0, 300.0, 0.0]);
    }

    #[test]
    fn single_interior_glyph_removed_splits_run() {
        let glyphs = vec![
            glyph(b"a", 0.0, 0.0, 10.0, 12.0, (0, 1)),
            glyph(b"X", 100.0, 0.0, 110.0, 12.0, (0, 2)), // in region
            glyph(b"b", 200.0, 0.0, 210.0, 12.0, (0, 3)),
        ];
        let r = region_at(95.0, -5.0, 115.0, 20.0);
        let out = prune_run(&glyphs, &r, DEFAULT_EDGE_PADDING);
        assert_eq!(out.glyphs_removed, 1);
        assert_eq!(out.removed_codes, vec![(0, 2)]);
        assert_eq!(out.runs.len(), 2);
        assert_eq!(out.runs[0].bytes, b"a");
        assert_eq!(out.runs[1].bytes, b"b");
        assert_eq!(out.runs[1].anchor, [1.0, 0.0, 0.0, 1.0, 200.0, 0.0]);
    }

    #[test]
    fn conservative_edge_touch_is_removed() {
        // Glyph sits just outside the region bbox but within the
        // edge-padding margin ⇒ must be removed (G1, never under-redact).
        let glyphs = vec![glyph(b"z", 100.3, 0.0, 100.4, 12.0, (0, 1))];
        let r = region_at(0.0, -5.0, 100.0, 20.0);
        let out = prune_run(&glyphs, &r, DEFAULT_EDGE_PADDING);
        assert_eq!(out.glyphs_removed, 1);
        assert!(out.runs.is_empty());
    }

    #[test]
    fn removed_codes_are_deduped_first_seen_order() {
        let glyphs = vec![
            glyph(b"x", 10.0, 10.0, 20.0, 22.0, (0, 7)),
            glyph(b"x", 20.0, 10.0, 30.0, 22.0, (0, 7)), // same code again
            glyph(b"y", 30.0, 10.0, 40.0, 22.0, (1, 3)),
        ];
        let r = region_at(0.0, 0.0, 100.0, 100.0);
        let out = prune_run(&glyphs, &r, DEFAULT_EDGE_PADDING);
        assert_eq!(out.glyphs_removed, 3);
        assert_eq!(out.removed_codes, vec![(0, 7), (1, 3)]);
    }

    #[test]
    fn empty_input_is_empty_result() {
        let r = region_at(0.0, 0.0, 10.0, 10.0);
        let out = prune_run(&[], &r, DEFAULT_EDGE_PADDING);
        assert_eq!(out, TextPruneResult::default());
    }

    #[test]
    fn no_regions_keeps_everything() {
        let glyphs = vec![
            glyph(b"o", 0.0, 0.0, 10.0, 12.0, (0, 1)),
            glyph(b"k", 10.0, 0.0, 20.0, 12.0, (0, 2)),
        ];
        let empty = RegionSet::new(0);
        let out = prune_run(&glyphs, &empty, DEFAULT_EDGE_PADDING);
        assert_eq!(out.glyphs_removed, 0);
        assert_eq!(out.runs.len(), 1);
        assert_eq!(out.runs[0].bytes, b"ok");
    }
}
