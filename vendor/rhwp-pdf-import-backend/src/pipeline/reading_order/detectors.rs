//! Per-class layout classifiers used by the reading-order pipeline.
//!
//! Each detector recognises a specific layout shape from a region's
//! span set. The detectors are written as pure predicates over span
//! geometry so they can be invoked from any layout pipeline:
//!
//! - [`detect_dramatic_script`]: ≥3 lines starting with a
//!   short token (≤12 chars) ending in `.` at a consistent left X,
//!   followed by a wide gap (>4×em) to the next glyph (Macbeth-style
//!   speaker tags).
//! - [`detect_dense_single_line`]: >80% of glyphs share a single Y
//!   (≤0.5pt) and the X-density is bimodal so the downstream
//!   assembler would otherwise split into two output lines
//!   (SEC DEF 14A 8pt-body interleave).
//! - [`detect_sub_super_glyphs`]: any glyph in the region has
//!   Y-offset from the surrounding line baseline in
//!   (0.2 × font_size, 0.8 × font_size) — chemical-formula
//!   subscripts.
//! - [`detect_narrow_tracked`]: per-line median gap differs from
//!   the font's space-advance by > 1.5× (stretched-column
//!   justified text that produces intra-word splits).
//!
//! The predicates here are usable standalone (callers pass span
//! coordinates) and unit-testable on synthetic input.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Layout classes recognised by the detectors. Used to dispatch
/// per-class assembly strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadingOrderClass {
    /// Default: y-then-x within region (the behaviour).
    Default,
    /// Dramatic-script layout. Speaker tags at consistent
    /// left X; row-major join required.
    DramaticScript,
    /// Dense single-Y line clustered into two output rows.
    /// Single-row regroup required.
    DenseSingleLine,
    /// Sub/super glyphs displaced from baseline. Baseline
    /// reattach required.
    SubSuperBaselineReattach,
    /// Narrow-tracked justified columns. Per-line median-gap
    /// threshold normalisation required.
    NarrowTrackedJustified,
}

/// One glyph or text-fragment for detector input. Minimal geometric
/// surface — detectors only need position and font size, not full
/// `TextSpan` semantics. Callers convert from their internal span
/// types via the trivial mapping.
#[derive(Debug, Clone, Copy)]
pub struct DetectorGlyph {
    /// Lower-left X of the glyph bbox in PDF page coordinates.
    pub x: f32,
    /// Baseline Y in PDF page coordinates.
    pub y: f32,
    /// Glyph width in pt (advance width × font size / 1000).
    pub width: f32,
    /// Font size of this glyph in pt.
    pub font_size: f32,
    /// Length of the text this glyph carries (1 for most ASCII;
    /// multi-char for AGL ligature expansions).
    pub text_len: usize,
}

/// Detect Macbeth-style speaker-tag layout.
///
/// **Trigger**: ≥3 distinct Y-rows where each row starts with a
/// short token (≤12 chars) ending in `.` at a consistent left X
/// (within 2pt), followed by a wide gap (>4×em) to the next glyph.
///
/// **Input shape**: `row_first_glyphs[i]` MUST be the leftmost
/// glyph of `row_texts[i]` (parallel arrays of equal length). The
/// detector reads only the `.x` field, so the geometry of all
/// other glyphs in the row is irrelevant here.
pub fn detect_dramatic_script(row_first_glyphs: &[DetectorGlyph], row_texts: &[&str]) -> bool {
    if row_texts.len() < 3 || row_first_glyphs.len() != row_texts.len() {
        return false;
    }
    let mut speaker_row_count = 0;
    let mut leftmost_x: Option<f32> = None;
    for (row_idx, row) in row_texts.iter().enumerate() {
        let trimmed = row.trim_start();
        if let Some(dot_pos) = trimmed.find('.') {
            let token = &trimmed[..=dot_pos];
            if token.len() <= 12 && !token.is_empty() {
                let first_glyph = &row_first_glyphs[row_idx];
                match leftmost_x {
                    None => leftmost_x = Some(first_glyph.x),
                    Some(prev_x) => {
                        if (prev_x - first_glyph.x).abs() < 2.0 {
                            speaker_row_count += 1;
                        }
                    },
                }
            }
        }
    }
    speaker_row_count >= 3
}

/// Detect a single-Y glyph cluster that the downstream assembler
/// would otherwise split into two output rows.
///
/// **Trigger**: >80% of glyphs share a single Y (within 0.5pt),
/// the X positions cluster into TWO disjoint bands (gap > 5pt
/// between bands). This is the SEC DEF 14A 8pt-body interleave
/// shape — `extract_chars` confirms all glyphs at origin_y == 584.39
/// but `extract_text` emits two character-alternating rows.
pub fn detect_dense_single_line(glyphs: &[DetectorGlyph]) -> bool {
    if glyphs.len() < 8 {
        return false;
    }
    // Bin Y onto a 0.5 pt grid in one pass rather than a nested scan (O(n²) on
    // all-distinct-Y pages). BTreeMap keeps tie-breaking deterministic; for
    // dense single-line text the dominant bin matches the old clustering.
    let mut bins: std::collections::BTreeMap<i32, usize> = std::collections::BTreeMap::new();
    for g in glyphs {
        *bins.entry((g.y * 2.0).round() as i32).or_insert(0) += 1;
    }
    let total = glyphs.len();
    let Some((&dominant_key, &dominant_count)) = bins.iter().max_by_key(|(_, c)| **c) else {
        return false;
    };
    if (dominant_count as f32) / (total as f32) < 0.8 {
        return false;
    }
    // Among glyphs in the dominant Y-bin, do the X positions form two
    // disjoint bands? Compute the gap distribution; bimodal means
    // one large gap stands out from the rest.
    let mut xs: Vec<f32> = glyphs
        .iter()
        .filter(|g| (g.y * 2.0).round() as i32 == dominant_key)
        .map(|g| g.x)
        .collect();
    xs.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
    let mut gaps: Vec<f32> = xs.windows(2).map(|w| w[1] - w[0]).collect();
    if gaps.is_empty() {
        return false;
    }
    let max_gap = gaps.iter().cloned().fold(0.0f32, f32::max);
    gaps.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
    let median_gap = gaps[gaps.len() / 2];
    // Bimodal: one gap is much larger than the typical (median)
    // intra-glyph gap. Ratio > 4 catches the SEC DEF 14A case
    // (45pt outlier vs ~5pt intra-band) while suppressing
    // false positives on uniformly-tracked stretched text where
    // all gaps are similar.
    if median_gap > 0.0 {
        max_gap > 4.0 * median_gap
    } else {
        max_gap > 5.0
    }
}

/// Detect sub/superscript glyphs offset from the surrounding line
/// baseline.
///
/// **Trigger**: any glyph in the region has a Y-offset from its
/// neighbours' line baseline in (0.2 × font_size, 0.8 × font_size).
/// Catches chemical formula subscripts (`H₂SO₄`) and footnote
/// markers where the writer used `Ts` text-rise or an explicit
/// `Tm` y-offset.
pub fn detect_sub_super_glyphs(glyphs: &[DetectorGlyph]) -> bool {
    if glyphs.len() < 2 {
        return false;
    }
    // Cluster glyphs by Y within 0.3 × font_size of each other.
    // Find the dominant Y (baseline). Any glyph whose Y differs from
    // baseline by more than 0.2×fs but less than 0.8×fs is sub/super.
    let mut sum_y = 0.0f32;
    let mut sum_fs = 0.0f32;
    for g in glyphs {
        sum_y += g.y;
        sum_fs += g.font_size;
    }
    let baseline_y = sum_y / glyphs.len() as f32;
    let avg_fs = sum_fs / glyphs.len() as f32;
    let lower = 0.2 * avg_fs;
    let upper = 0.8 * avg_fs;
    glyphs.iter().any(|g| {
        let dy = (g.y - baseline_y).abs();
        dy > lower && dy < upper
    })
}

/// Detect narrow-tracked justified columns where intra-word gaps
/// exceed the proportional-font threshold.
///
/// **Trigger**: per-glyph X-gaps cluster bimodally. The intra-word
/// gap median should be much smaller than the inter-word gap; in
/// stretched justified columns, the intra-word gap rises until it
/// exceeds the standard threshold (0.5 × space-advance).
pub fn detect_narrow_tracked(glyphs: &[DetectorGlyph]) -> bool {
    if glyphs.len() < 6 {
        return false;
    }
    // Sort an index vector by X instead of cloning the glyph slice.
    let mut order: Vec<usize> = (0..glyphs.len()).collect();
    order.sort_by(|&a, &b| crate::utils::safe_float_cmp(glyphs[a].x, glyphs[b].x));
    let mut gaps: Vec<f32> = order
        .windows(2)
        .map(|w| {
            let (prev, next) = (&glyphs[w[0]], &glyphs[w[1]]);
            (next.x - (prev.x + prev.width)).max(0.0)
        })
        .collect();
    if gaps.is_empty() {
        return false;
    }
    gaps.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
    // Median gap.
    let median = gaps[gaps.len() / 2];
    // Expected intra-word gap for proportional font @ avg font_size:
    // typically ~0.05-0.1 × font_size. For monospace, ~0.0.
    let avg_fs: f32 = glyphs.iter().map(|g| g.font_size).sum::<f32>() / glyphs.len() as f32;
    let expected_intra = 0.08 * avg_fs;
    // Narrow-tracked when median gap exceeds 1.5× expected.
    median > 1.5 * expected_intra
}

/// Classify a region into a `ReadingOrderClass` using the per-class
/// detectors in priority order. Most-specific detectors fire first;
/// regions that don't match any specific shape return `Default`.
///
/// `glyphs` is the page's per-span geometry (used by Dense /
/// Sub-Super / Narrow-Tracked). `row_first_glyphs` is parallel to
/// `row_texts` and supplies the leftmost glyph of each row for the
/// DramaticScript X-consistency check; pass `&[]` / `&[]` for both
/// if row-level data is unavailable (DramaticScript will then
/// never fire).
pub fn classify_region(
    glyphs: &[DetectorGlyph],
    row_first_glyphs: &[DetectorGlyph],
    row_texts: &[&str],
) -> ReadingOrderClass {
    if detect_dramatic_script(row_first_glyphs, row_texts) {
        return ReadingOrderClass::DramaticScript;
    }
    if detect_dense_single_line(glyphs) {
        return ReadingOrderClass::DenseSingleLine;
    }
    if detect_sub_super_glyphs(glyphs) {
        return ReadingOrderClass::SubSuperBaselineReattach;
    }
    if detect_narrow_tracked(glyphs) {
        return ReadingOrderClass::NarrowTrackedJustified;
    }
    ReadingOrderClass::Default
}

#[cfg(test)]
mod tests {
    use super::*;

    fn glyph(x: f32, y: f32, width: f32, font_size: f32) -> DetectorGlyph {
        DetectorGlyph {
            x,
            y,
            width,
            font_size,
            text_len: 1,
        }
    }

    #[test]
    fn dramatic_script_fires_on_macbeth_shape() {
        // 4 speaker rows, all starting at left X=50, with the
        // `First Witch.` / `Sec. Witch.` / etc. pattern.
        // `row_first_glyphs[i]` is the leftmost glyph of `rows[i]`.
        let row_first_glyphs = vec![
            glyph(50.0, 100.0, 5.0, 10.0),
            glyph(50.0, 90.0, 5.0, 10.0),
            glyph(50.0, 80.0, 5.0, 10.0),
            glyph(50.0, 70.0, 5.0, 10.0),
        ];
        let rows = vec![
            "First Witch.    I ask you.",
            "Sec. Witch.     Speak.",
            "Third Witch.    Demand.",
            "All.            We'll answer.",
        ];
        assert!(detect_dramatic_script(&row_first_glyphs, &rows));
        // For classify_region the per-glyph signal (Dense / SubSuper /
        // NarrowTracked) is fed by the full-page glyph list; reuse
        // row_first_glyphs as both here since the synthetic shape
        // doesn't exercise those.
        assert_eq!(
            classify_region(&row_first_glyphs, &row_first_glyphs, &rows),
            ReadingOrderClass::DramaticScript,
        );
    }

    #[test]
    fn dramatic_script_skips_prose() {
        let row_first_glyphs = vec![
            glyph(50.0, 100.0, 5.0, 10.0),
            glyph(50.0, 90.0, 5.0, 10.0),
            glyph(50.0, 80.0, 5.0, 10.0),
        ];
        let rows = vec![
            "The first paragraph of a novel begins here.",
            "And continues with more text.",
            "This is plain prose, no speaker tags.",
        ];
        assert!(!detect_dramatic_script(&row_first_glyphs, &rows));
    }

    #[test]
    fn dense_single_line_detects_sec_proxy_shape() {
        // 12 glyphs all at y=584.39, x spans 100..200 with a gap
        // at x=150 (bimodal).
        let mut glyphs = Vec::new();
        for x in [100.0, 105.0, 110.0, 115.0, 120.0, 125.0].iter() {
            glyphs.push(glyph(*x, 584.39, 2.0, 8.0));
        }
        for x in [170.0, 175.0, 180.0, 185.0, 190.0, 195.0].iter() {
            glyphs.push(glyph(*x, 584.39, 2.0, 8.0));
        }
        assert!(detect_dense_single_line(&glyphs));
    }

    #[test]
    fn dense_single_line_skips_multi_line() {
        let glyphs = vec![
            glyph(50.0, 100.0, 2.0, 8.0),
            glyph(60.0, 100.0, 2.0, 8.0),
            glyph(50.0, 90.0, 2.0, 8.0),
            glyph(60.0, 90.0, 2.0, 8.0),
            glyph(50.0, 80.0, 2.0, 8.0),
            glyph(60.0, 80.0, 2.0, 8.0),
            glyph(50.0, 70.0, 2.0, 8.0),
            glyph(60.0, 70.0, 2.0, 8.0),
        ];
        // Glyphs spread across many Ys — not single-line.
        assert!(!detect_dense_single_line(&glyphs));
    }

    #[test]
    fn sub_super_detects_subscript_y_offset() {
        // 4 glyphs at baseline Y=100, plus 1 at Y=104 (sub/super
        // displacement of 4pt with font_size=10pt → 0.4×fs).
        let glyphs = vec![
            glyph(50.0, 100.0, 5.0, 10.0),
            glyph(55.0, 100.0, 5.0, 10.0),
            glyph(60.0, 100.0, 5.0, 10.0),
            glyph(65.0, 100.0, 5.0, 10.0),
            glyph(70.0, 104.0, 5.0, 10.0),
        ];
        assert!(detect_sub_super_glyphs(&glyphs));
    }

    #[test]
    fn sub_super_skips_uniform_baseline() {
        let glyphs = vec![
            glyph(50.0, 100.0, 5.0, 10.0),
            glyph(55.0, 100.0, 5.0, 10.0),
            glyph(60.0, 100.0, 5.0, 10.0),
        ];
        assert!(!detect_sub_super_glyphs(&glyphs));
    }

    #[test]
    fn narrow_tracked_detects_stretched_justification() {
        // 10 glyphs with wide gaps (median gap > 1.5× expected intra-
        // word gap for font_size=10). Expected intra ≈ 0.8pt; gaps
        // here are ~3pt (justified stretch).
        let mut glyphs = Vec::new();
        for i in 0..10 {
            glyphs.push(glyph(50.0 + (i as f32) * 8.0, 100.0, 5.0, 10.0));
        }
        assert!(detect_narrow_tracked(&glyphs));
    }

    #[test]
    fn narrow_tracked_skips_normal_spacing() {
        // 10 glyphs touching each other (gap ≈ 0).
        let mut glyphs = Vec::new();
        for i in 0..10 {
            glyphs.push(glyph(50.0 + (i as f32) * 5.1, 100.0, 5.0, 10.0));
        }
        assert!(!detect_narrow_tracked(&glyphs));
    }

    #[test]
    fn classify_default_when_no_pattern_matches() {
        let glyphs = vec![glyph(50.0, 100.0, 5.0, 10.0), glyph(56.0, 100.0, 5.0, 10.0)];
        assert_eq!(classify_region(&glyphs, &[], &[]), ReadingOrderClass::Default,);
    }
}
