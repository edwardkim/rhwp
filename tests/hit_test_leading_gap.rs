//! Regression: click → caret hit-test must resolve the click x to the correct
//! on-line character offset even when the click y lands in the line's *leading
//! gap* (a few px above the glyph box top), not only when y is inside the glyph
//! box.
//!
//! Bug (`hit_test_native`): the cell-run / body / closest-line fallbacks resolved
//! the click x to a precise character only when the click y fell *inside* a
//! TextRun's glyph bbox. When the click landed in the line's leading gap — where
//! real clicks routinely land — the fallback snapped the caret to the LINE START
//! (`char_start`) or LINE END (`char_start + char_count`) instead of the
//! character under the click x. Multi-run lines additionally snapped to the line
//! end for any x past the first run.
//!
//! Reproduction document: `samples/exam_social.hwp` (already in the repo).
//! Target: page 0, left column, the body line whose glyph box top is y≈482.7.
//! We sweep the click x across that line at two click y values:
//!   * y = 488.0 — *inside* the glyph box (always worked).
//!   * y = 481.5 — in the *leading gap* just above the box top (the bug).
//!
//! We then assert the caret x tracks the click x within tolerance and increases
//! monotonically — i.e. no snap to a constant line-start/line-end x. The
//! leading-gap case failed before the fix.

use std::path::Path;

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn load() -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/exam_social.hwp");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    HwpDocument::from_bytes(&bytes).expect("parse exam_social.hwp")
}

/// (charOffset, cursorRect.x) for a click.
fn hit(doc: &HwpDocument, page: u32, x: f64, y: f64) -> (u64, f64) {
    let json = doc
        .hit_test_native(page, x, y)
        .unwrap_or_else(|e| panic!("hit_test_native({page},{x},{y}): {e}"));
    let v: Value = serde_json::from_str(&json).unwrap_or_else(|e| panic!("parse `{json}`: {e}"));
    (
        v["charOffset"]
            .as_u64()
            .unwrap_or_else(|| panic!("no charOffset in `{json}`")),
        v["cursorRect"]["x"]
            .as_f64()
            .unwrap_or_else(|| panic!("no cursorRect.x in `{json}`")),
    )
}

// Interior x range of the target line (just inside the line start/end so we are
// never clamping to the line edges).
const X_LEFT: f64 = 200.0;
const X_RIGHT: f64 = 500.0;
// One snapped-to-edge click is off by 100+px (the half-line distance); a correct
// resolution lands within ~one glyph width of the click. The font here is ~11.5px
// wide and the caret snaps to a glyph boundary, so allow a generous 2-glyph slack.
const TOLERANCE_PX: f64 = 24.0;

fn assert_line_tracks_x(doc: &HwpDocument, y: f64) {
    let mut prev_cx = f64::NEG_INFINITY;
    let mut prev_off: Option<u64> = None;
    let mut distinct = std::collections::BTreeSet::new();
    let mut x = X_LEFT;
    while x <= X_RIGHT {
        let (off, cx) = hit(doc, 0, x, y);
        distinct.insert(off);

        // (a) caret x stays near the click x (no snap to a constant line edge).
        assert!(
            (cx - x).abs() <= TOLERANCE_PX,
            "interior click y={y} x={x} → caret x={cx} (Δ={:.1}px) snapped away from \
             the click; expected within {TOLERANCE_PX}px (offset={off})",
            (cx - x).abs()
        );

        // (b) caret x and char offset are (weakly) monotonically increasing.
        assert!(
            cx + 0.5 >= prev_cx,
            "caret x not monotonic at y={y} x={x}: caret x={cx} < previous {prev_cx}"
        );
        if let Some(p) = prev_off {
            assert!(
                off >= p,
                "char offset not monotonic at y={y} x={x}: {off} < previous {p}"
            );
        }
        prev_cx = cx;
        prev_off = Some(off);
        x += 10.0;
    }

    // The sweep must hit many distinct offsets; a snap-to-edge bug collapses the
    // whole interior to one or two offsets.
    assert!(
        distinct.len() >= 8,
        "interior x sweep at y={y} resolved to only {} distinct offsets {distinct:?}; \
         expected the caret to track across the line (snap-to-line-edge regression)",
        distinct.len()
    );
}

#[test]
fn interior_clicks_track_x_inside_glyph_box() {
    let doc = load();
    assert_line_tracks_x(&doc, 488.0);
}

#[test]
fn interior_clicks_track_x_in_leading_gap() {
    // The regression case: y=481.5 is ~1px above the glyph box top (≈482.7),
    // inside the line's leading gap. Before the fix this snapped to the line
    // start/end instead of tracking the click x.
    let doc = load();
    assert_line_tracks_x(&doc, 481.5);
}
