//! Regression: `set_cell_text_native` / `set_cell_text_by_path_native` REPLACE
//! of a table cell with MANY short paragraphs must render the paragraphs as
//! distinct, vertically-separated lines — not collapsed onto one y row.
//!
//! Root cause: the multi-paragraph REPLACE built the extra paragraphs via
//! `Paragraph::split_at`, whose new `line_segs` carry `vertical_pos = 0`
//! (only intra-paragraph vpos is accumulated by `reflow_line_segs`). The
//! Top-aligned table-cell renderer (`table_layout.rs`) anchors every cell
//! paragraph at `cell_top + LINE_SEG.vertical_pos`; with all vpos == 0 the N
//! paragraphs overlapped on the same y and rendered as garbage. The fix runs
//! `recalculate_section_vpos` over the cell's paragraph vector so each
//! paragraph's first LINE_SEG gets the cumulative offset (0, lh+ls, 2(lh+ls)…).
//!
//! Proof: render page 0 to SVG, read the glyph y-coordinates of the inserted
//! single-letter runs, and assert they occupy N distinct, monotonically
//! increasing y rows with a line pitch > epsilon.

use std::path::Path;

use rhwp::wasm_api::HwpDocument;

fn load(rel: &str) -> HwpDocument {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let b = std::fs::read(&p).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    HwpDocument::from_bytes(&b).unwrap_or_else(|e| panic!("parse {rel}: {e:?}"))
}

/// (x, y) of every `<text>` run whose body is exactly one of `letters`.
/// Handles both `x=""/y=""` and `transform="translate(x,y)"` glyph encodings.
fn letter_xy(svg: &str, letters: &[char]) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    let mut rest = svg;
    while let Some(o) = rest.find("<text") {
        let after = &rest[o..];
        let te = after.find('>').unwrap_or(after.len());
        let tag = &after[..te];
        let body = &after[te + 1..];
        let Some(ce) = body.find("</text>") else {
            break;
        };
        let b = &body[..ce];
        if b.chars().count() == 1 && letters.contains(&b.chars().next().unwrap()) {
            let attr = |name: &str| -> Option<f64> {
                let key = format!(" {name}=\"");
                let i = tag.find(&key)? + key.len();
                let j = tag[i..].find('"')? + i;
                tag[i..j].parse().ok()
            };
            let xy = if let (Some(x), Some(y)) = (attr("x"), attr("y")) {
                Some((x, y))
            } else if let Some(to) = tag.find("translate(") {
                let t = &tag[to + 10..];
                t.find(',').and_then(|c| {
                    let x: f64 = t[..c].parse().ok()?;
                    let tc = &t[c + 1..];
                    let ye = tc
                        .find(|ch: char| ch == ')' || ch == ' ')
                        .unwrap_or(tc.len());
                    let y: f64 = tc[..ye].parse().ok()?;
                    Some((x, y))
                })
            } else {
                None
            };
            if let Some((x, y)) = xy {
                out.push((round1(x), round1(y)));
            }
        }
        rest = &body[ce + 7..];
    }
    out
}

/// Distinct sorted y's of the inserted letters that fall in the SAME x-column
/// (the edited cell). Groups by x and returns the rows of the largest cluster,
/// so unrelated cells elsewhere on the page with the same letters don't count.
fn distinct_cell_rows(svg: &str, letters: &[char]) -> Vec<f64> {
    let pts = letter_xy(svg, letters);
    // cluster by x (cell column); tolerance 40px covers per-glyph x within a line.
    let mut cols: Vec<(f64, Vec<f64>)> = Vec::new();
    for (x, y) in pts {
        if let Some(c) = cols.iter_mut().find(|(cx, _)| (*cx - x).abs() < 40.0) {
            c.1.push(y);
        } else {
            cols.push((x, vec![y]));
        }
    }
    let mut best = cols
        .into_iter()
        .max_by_key(|(_, ys)| ys.len())
        .map(|(_, ys)| ys)
        .unwrap_or_default();
    best.sort_by(|a, b| a.partial_cmp(b).unwrap());
    best.dedup();
    best
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

/// Assert there is a run of `expected` consecutive rows in `ys` that are
/// uniformly pitched (the edited cell's lines). The collapse bug produces a
/// SINGLE row for the whole cell, so no such run can exist. We require each gap
/// in the run to be > `min_pitch` and within 50% of the median gap (uniform).
fn assert_separated(ys: &[f64], expected: usize, min_pitch: f64, ctx: &str) {
    assert!(
        ys.len() >= expected,
        "{ctx}: need >= {expected} distinct rows, got {} -> {ys:?} \
         (collapse bug = the cell's paragraphs share one y row)",
        ys.len()
    );
    let gaps: Vec<f64> = ys.windows(2).map(|w| w[1] - w[0]).collect();
    // slide a window of (expected-1) gaps; find one that is uniform & > min_pitch.
    let need = expected - 1;
    let ok = gaps.windows(need).any(|win| {
        let mut sorted = win.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let med = sorted[sorted.len() / 2];
        med > min_pitch
            && win
                .iter()
                .all(|&g| g > min_pitch && (g - med).abs() <= med * 0.5)
    });
    assert!(
        ok,
        "{ctx}: expected a run of {expected} uniformly-pitched rows (> {min_pitch}px, \
         within 50% of median) but found none in ys={ys:?} gaps={gaps:?} \
         (collapse bug = paragraphs overlap on one row)"
    );
}

// Fixture: `inner-table-01.hwp` paragraph 0 holds a table (control 2). Cell 10
// is Top-aligned with a non-zero first-line vpos, so the renderer takes the
// `cell_top + LINE_SEG.vertical_pos` per-paragraph anchor path — exactly the
// path the bug lived in. Its content renders in the left column (x ~= 128) on
// page 0. Without the fix, a 6-paragraph REPLACE collapses to a single y row
// there; with the fix, 6 distinct, uniformly-pitched rows appear.
const FIX: &str = "samples/inner-table-01.hwp";
const SEC: usize = 0;
const PPI: usize = 0;
const CTRL: usize = 2;
const CELL: usize = 10;

/// 6 short paragraphs in a Top-anchored table cell must render on 6 distinct,
/// uniformly-pitched y rows (the core regression for the collapse bug).
#[test]
fn set_cell_text_six_paragraphs_render_on_distinct_rows() {
    let mut doc = load(FIX);
    let lines: Vec<String> = ["A", "B", "C", "D", "E", "F"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let res = doc
        .set_cell_text_native(SEC, PPI, CTRL, CELL, &lines)
        .expect("set_cell_text_native");
    assert!(res.contains("\"cellParaCount\":6"), "got {res}");

    let svg = doc.render_page_svg_native(0).expect("render");
    let ys = distinct_cell_rows(&svg, &['A', 'B', 'C', 'D', 'E', 'F']);
    assert_separated(&ys, 6, 10.0, "6-paragraph set_cell");
}

/// 1-line and 2-line REPLACE must still render with correct (non-zero) spacing —
/// the cases that already worked before the fix must not regress.
#[test]
fn set_cell_text_one_and_two_paragraphs_keep_spacing() {
    // 1 line: a single visible row.
    {
        let mut doc = load(FIX);
        doc.set_cell_text_native(SEC, PPI, CTRL, CELL, &["Q".to_string()])
            .expect("set 1 line");
        let svg = doc.render_page_svg_native(0).expect("render");
        let ys = distinct_cell_rows(&svg, &['Q']);
        assert!(
            !ys.is_empty(),
            "single paragraph must render a visible row: {ys:?}"
        );
    }
    // 2 lines: two vertically-separated rows.
    {
        let mut doc = load(FIX);
        doc.set_cell_text_native(SEC, PPI, CTRL, CELL, &["Q".to_string(), "Z".to_string()])
            .expect("set 2 lines");
        let svg = doc.render_page_svg_native(0).expect("render");
        let ys = distinct_cell_rows(&svg, &['Q', 'Z']);
        assert_separated(&ys, 2, 10.0, "2-paragraph set_cell");
    }
}

// The nested `set_cell_text_by_path_native` vpos-cascade assertion lives as an
// in-crate unit test (`document_core::commands::text_editing::tests::
// set_cell_by_path_cascades_vertical_pos`) because it inspects the crate-private
// `line_segs.vertical_pos` of the innermost (non-page-0-rendered) nested cell.
