//! Caret↔insert offset alignment on `employment_v1.hwp` at the "하도급계" title.
//!
//! Edge case of the table-cell/body caret↔edit offset fix (rhwp_core 6315ea7).
//! The title paragraph (sec 0, para 0) leads with non-inline *structural* controls
//! (SectionDef + ColumnDef at text position 0, NewNumber at the end). Two paths
//! disagreed on the offset→position mapping:
//!
//!   1. The render text-run synthesizer (`synthesize_marker_paragraph`, Task #991)
//!      counted these structural controls as inline-visible and injected leading
//!      `\u{FFFC}` markers. The measurer then advanced past those (non-painted)
//!      markers, shifting every caret stop one glyph to the right of the painted
//!      glyph it should sit on.
//!   2. `cursor_offset_to_text_offset` (insert/delete) counted ALL controls (incl.
//!      the two structural ones) via `control_text_positions()`, landing inserts a
//!      glyph too early.
//!
//! Symptom: clicking placed the caret after "하도급" (before "계", where it
//! renders) but typing inserted the char ~one glyph earlier (after "도").
//!
//! Contract: an insert of one char at offset `o` must place the new glyph exactly
//! where `cursor_rect(o)` renders the caret — the inserted glyph's painted left
//! edge equals the post-insert caret x for offset `o` (and the caret for `o+1`
//! sits one glyph advance to the right). This holds even though the title is
//! center-aligned (insert recenters the whole line — so we compare against the
//! SVG-painted glyph, the ground truth, not the pre-insert caret x).

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;
use std::path::Path;

fn load() -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join("employment_v1.hwp");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read fixture: {e}"));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse fixture: {e}"))
}

fn caret_x(doc: &HwpDocument, sec: usize, para: usize, off: usize) -> f64 {
    let json = doc
        .get_cursor_rect_native(sec, para, off)
        .unwrap_or_else(|e| panic!("cursor_rect({sec},{para},{off}): {e}"));
    serde_json::from_str::<Value>(&json).unwrap()["x"]
        .as_f64()
        .unwrap()
}

fn hit(doc: &HwpDocument, page: u32, x: f64, y: f64) -> (usize, usize, usize) {
    let json = doc.hit_test_native(page, x, y).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    (
        v["sectionIndex"].as_u64().unwrap() as usize,
        v["paragraphIndex"].as_u64().unwrap() as usize,
        v["charOffset"].as_u64().unwrap() as usize,
    )
}

/// x of the first `<text ... x="N" ...>glyph</text>` whose content is exactly `glyph`.
fn glyph_x(svg: &str, glyph: &str) -> Option<f64> {
    let needle = format!(">{}</text>", glyph);
    let gpos = svg.find(&needle)?;
    let tstart = svg[..gpos].rfind("<text")?;
    let attrs = &svg[tstart..gpos];
    let xs = attrs.find("x=\"")? + 3;
    let xr = &attrs[xs..];
    let xe = xr.find('"')?;
    xr[..xe].parse().ok()
}

#[test]
fn caret_and_insert_agree_at_employment_title() {
    let mut doc = load();

    // Click just left of "계" — the "하도급|계" boundary (glyphs render with a
    // 20px CJK advance; clicking at x=400 lands on the 급/계 boundary).
    let (sec, para, off) = hit(&doc, 0, 400.0, 104.0);
    assert_eq!((sec, para), (0, 0), "title is sec0/para0");

    // Insert a distinctive wide CJK glyph at the resolved caret offset.
    doc.insert_text_native(sec, para, off, "캭")
        .expect("insert");

    // Ground truth: where did the inserted glyph actually paint?
    let svg = doc.render_page_svg_native(0).expect("render");
    let inserted_x = glyph_x(&svg, "캭").expect("inserted glyph must render on page 0");

    // The caret for `off` (now immediately before the inserted glyph) must sit at
    // the inserted glyph's painted left edge, and the caret for `off+1` (after it)
    // one full glyph advance (~20px) to the right.
    let caret_at = caret_x(&doc, sec, para, off);
    let caret_after = caret_x(&doc, sec, para, off + 1);
    let advance = caret_after - caret_at;

    eprintln!(
        "off={off} inserted_glyph_x={inserted_x:.1} caret(off)={caret_at:.1} \
         caret(off+1)={caret_after:.1} advance={advance:.1}"
    );

    // (a) the inserted glyph sits AT the caret for `off` (caret ↔ insert agree).
    assert!(
        (inserted_x - caret_at).abs() < 1.5,
        "inserted glyph (x={inserted_x:.1}) must render AT the caret for offset {off} \
         (x={caret_at:.1})"
    );

    // (b) caret(off) and caret(off+1) bracket exactly one glyph advance (~20px),
    //     i.e. the inserted glyph lives between them — not one slot away.
    assert!(
        advance > 15.0 && advance < 25.0,
        "caret(off) and caret(off+1) must differ by one glyph advance (~20px), \
         got {advance:.1}px — caret and insert use different offset mappings"
    );
}
