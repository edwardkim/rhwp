//! ecrits #148: a BODY paragraph must re-wrap after a length-changing edit
//! (field fill / replaceAll), not run off the right margin.
//!
//! Root cause: `delete_text_at`/`insert_text_at` only SHIFT line_segs.text_start by
//! the char delta; they never recompute the wrap boundaries. `compose_paragraph`
//! slices each composed line strictly from line_segs[i].text_start to
//! line_segs[i+1].text_start, so after a short blank ("------") is replaced by a
//! long value the last composed line absorbs the extra glyphs and overflows the
//! column. The cell edit path already reflows (reflow_cell_paragraph); the body
//! field-fill (`set_field_*`) and `replaceAll` body paths did not. This test asserts
//! that after such an edit NO composed line of the paragraph exceeds the column
//! text-area width (the composer is the source the canvas/SVG renderers lay out).

use rhwp::renderer::composer::{compose_paragraph, estimate_composed_line_width};
use rhwp::renderer::style_resolver::resolve_styles;
use rhwp::wasm_api::HwpDocument;
use std::path::Path;

const DPI: f64 = 96.0;

fn load() -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join("employment_v1.hwp");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read fixture: {e}"));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse fixture: {e}"))
}

/// Column text-area width (px) for a top-level body paragraph.
fn body_col_width_px(doc: &HwpDocument, sec: usize, para: usize) -> f64 {
    use rhwp::renderer::page_layout::PageLayoutInfo;
    let section = &doc.document().sections[sec];
    let page_def = &section.section_def.page_def;
    let column_def = HwpDocument::find_column_def_for_paragraph(&section.paragraphs, para);
    let layout = PageLayoutInfo::from_page_def(page_def, &column_def, DPI);
    layout.column_areas[0].width
}

/// Assert no composed line of (sec,para) overflows the column text area.
fn assert_no_overflow(doc: &HwpDocument, sec: usize, para: usize, ctx: &str) {
    let styles = resolve_styles(&doc.document().doc_info, DPI);
    let p = &doc.document().sections[sec].paragraphs[para];
    let col_w = body_col_width_px(doc, sec, para);
    let para_style = styles.para_styles.get(p.para_shape_id as usize);
    let ml = para_style.map(|s| s.margin_left).unwrap_or(0.0);
    let mr = para_style.map(|s| s.margin_right).unwrap_or(0.0);
    let avail = col_w - ml - mr;
    let composed = compose_paragraph(p);
    eprintln!(
        "[{ctx}] sec={sec} para={para} segs={} avail={avail:.1} lines={}",
        p.line_segs.len(),
        composed.lines.len()
    );
    // The reflow's fill_lines and the composer's estimate_composed_line_width share
    // estimate_text_width but accumulate/round slightly differently, so a correctly
    // wrapped line may measure a few px over here. A genuine wrap FAILURE leaves the
    // whole extra run on one line (hundreds of px over). Allow ~half a CJK glyph of
    // drift (10px); anything beyond that is a real overflow.
    const DRIFT_PX: f64 = 10.0;
    for (i, line) in composed.lines.iter().enumerate() {
        let w = estimate_composed_line_width(line, &styles);
        eprintln!("   line[{i}] width={w:.1} (avail={avail:.1})");
        assert!(
            w <= avail + DRIFT_PX,
            "[{ctx}] composed line {i} width {w:.1}px exceeds column text area {avail:.1}px \
             by more than measurement drift (body paragraph did not re-wrap after edit)"
        );
    }
}

#[test]
fn intro_rewraps_after_replace_all() {
    let mut doc = load();
    let (sec, para) = (0usize, 72usize);

    // Sanity: the clean template fits.
    assert_no_overflow(&doc, sec, para, "clean");

    // Demo-fill via replaceAll: replace the dash blanks with long company names.
    // ("------------" is a substring of "---------------", so replace the longer run
    // first to avoid partial matches.)
    doc.replace_all_native("---------------", "주식회사 클라우드솔루션", false)
        .expect("replace 1st dash run");
    doc.replace_all_native("------------", "주식회사 넥스트AI랩", false)
        .expect("replace 2nd dash run");

    let after = &doc.document().sections[sec].paragraphs[para];
    assert!(
        after.text.contains("클라우드솔루션") && after.text.contains("넥스트AI랩"),
        "fill must have applied, got {:?}",
        after.text
    );
    assert_no_overflow(&doc, sec, para, "after replaceAll");
}

#[test]
fn body_field_fill_rewraps() {
    let mut doc = load();
    // Pick a body field and overfill it with a long value, then assert no overflow.
    let fields = doc.collect_all_fields();
    let body_field = fields
        .iter()
        .find(|f| f.location.nested_path.is_empty())
        .map(|f| {
            (
                f.field.field_id,
                f.location.section_index,
                f.location.para_index,
            )
        })
        .expect("a body field exists");
    let (fid, sec, para) = body_field;
    let long = "주식회사 클라우드솔루션 및 주식회사 넥스트AI랩 사이의 매우 긴 추가 문구를 채워 넣어 본문 폭을 초과하게 만든다";
    doc.set_field_value_by_id(fid, long)
        .expect("set field value");
    let p = &doc.document().sections[sec].paragraphs[para];
    assert!(
        p.text.contains("넥스트AI랩"),
        "field fill applied: {:?}",
        p.text
    );
    assert_no_overflow(&doc, sec, para, "after field fill");
}
