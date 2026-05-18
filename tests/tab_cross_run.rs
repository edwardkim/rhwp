//! Cross-run tab detection regression tests — Task #290.
//!
//! samples/exam_math.hwp page 7 item 18 ("수열" problem) had its first-line text
//! pushed to the right edge of the column because cross-run tab detection used
//! `find_next_tab_stop` only (TabDef path), ignoring `composed.tab_extended`
//! (inline tab data). When the last `\t` of a run was a LEFT inline tab and all
//! TabDef stops were already passed, `auto_tab_right` fallback marked it RIGHT
//! and repositioned the next run to the column's right edge.
//!
//! This file pins the visual fix: "수" glyph must render near column start.

use std::fs;
use std::path::Path;

/// Render exam_math.hwp page 7 and extract the x-coordinate of the first "수" glyph
/// that appears in the left column's main body text (i.e., item 18's first line).
fn first_sukyul_x_in_left_column(svg: &str) -> Option<f64> {
    // Approximate left column range for exam_math (A4 portrait, body width ~884 px, 2 cols):
    // col 0 spans roughly [72, 492] in SVG px.
    // The first "수" in that range (excluding the title "수학 영역" which sits in the
    // page header area around y<150) is item 18's "수" at y~162.
    for line in svg.lines() {
        if !line.contains(">수<") {
            continue;
        }
        let Some(tx_start) = line.find("translate(") else {
            continue;
        };
        let rest = &line[tx_start + "translate(".len()..];
        let Some(comma) = rest.find(',') else {
            continue;
        };
        let Some(x) = rest[..comma].parse::<f64>().ok() else {
            continue;
        };
        let Some(paren) = rest.find(')') else {
            continue;
        };
        let Some(y) = rest[comma + 1..paren].parse::<f64>().ok() else {
            continue;
        };
        // Skip title "수학 영역" (y < 150) and right column (x > 500). Take the first hit
        // in body y (150~200) and left column (x < 300 normal, x > 300 = bug).
        if y >= 150.0 && y < 200.0 && x < 500.0 {
            return Some(x);
        }
    }
    None
}

#[test]
fn task290_exam_math_p7_item18_not_right_aligned() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let bytes = fs::read(Path::new(repo_root).join("samples/exam_math.hwp"))
        .expect("samples/exam_math.hwp present");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse");
    // Page 7 → 0-based index 6
    let svg = doc.render_page_svg_native(6).expect("render page 7");

    let x = first_sukyul_x_in_left_column(&svg)
        .expect("item 18 '수' glyph not found in expected region");
    // Before fix: x ≈ 290.9 (bug — right-aligned to column end ~420)
    // After fix:  x ≈ 109.8 (correct — just after "18." + inline tab widths)
    // Threshold 200 gives wide safety margin vs the 290+ bug.
    assert!(
        x < 200.0,
        "item 18 '수' glyph at x={x}; expected < 200 (near column start). \
         Likely regression of the cross-run tab detection fix (Task #290)."
    );
}
