//! Issue #1133: 중첩 표가 있는 가운데 정렬 셀의 콘텐츠 높이 회귀 가드.
//!
//! 재현 문서의 page 2 `지원서 접수` 오른쪽 셀은 `valign=Center`이고, 마지막 문단에
//! 중첩 1x1 표가 있다. 해당 문단의 LINE_SEG.line_height는 내부 표의 실제 높이를 담지
//! 않으므로, 측정 결과는 `line_seg.vpos + line_height`보다 커야 한다.

use rhwp::model::control::Control;
use rhwp::parser::parse_document;
use rhwp::renderer::composer::compose_paragraph;
use rhwp::renderer::height_measurer::HeightMeasurer;
use rhwp::renderer::style_resolver::resolve_styles_with_variant;
use rhwp::renderer::{hwpunit_to_px, DEFAULT_DPI};
use rhwp::wasm_api::HwpDocument;

fn assert_issue_1133_nested_table_height_contract(path: &str) {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    let doc = parse_document(&bytes).unwrap_or_else(|e| panic!("parse {path}: {e}"));
    let section = doc.sections.first().expect("section 0");
    let para = section.paragraphs.get(29).expect("pi=29");
    let table = match para.controls.first().expect("pi=29 table") {
        Control::Table(t) => t.as_ref(),
        other => panic!("pi=29 first control is not table: {other:?}"),
    };
    let target_cell = table.cells.get(3).expect("outer table cell[3]");
    assert!(
        target_cell
            .paragraphs
            .iter()
            .any(|p| p.controls.iter().any(|c| matches!(c, Control::Table(_)))),
        "target cell must contain a nested table"
    );

    let legacy_line_seg_bottom_px = target_cell
        .paragraphs
        .iter()
        .flat_map(|p| p.line_segs.last())
        .map(|s| hwpunit_to_px(s.vertical_pos + s.line_height, DEFAULT_DPI))
        .fold(0.0f64, f64::max);

    let styles = resolve_styles_with_variant(&doc.doc_info, DEFAULT_DPI, doc.is_hwp3_variant);
    let composed = section
        .paragraphs
        .iter()
        .map(compose_paragraph)
        .collect::<Vec<_>>();
    let measured = HeightMeasurer::new(DEFAULT_DPI)
        .with_hwp3_variant(doc.is_hwp3_variant)
        .measure_section(&section.paragraphs, &composed, &styles, None);
    let measured_table = measured
        .tables
        .iter()
        .find(|t| t.para_index == 29 && t.control_index == 0)
        .expect("measured table pi=29 ci=0");
    let measured_cell = measured_table
        .cells
        .iter()
        .find(|c| c.row == 1 && c.col == 1)
        .expect("measured cell r=1 c=1");

    assert!(
        measured_cell.total_content_height > legacy_line_seg_bottom_px + 100.0,
        "{path}: nested table content height was not reflected enough: measured={:.1}, legacy_line_seg_bottom={:.1}",
        measured_cell.total_content_height,
        legacy_line_seg_bottom_px
    );
}

fn render_debug_svg(path: &str, page_idx: u32) -> String {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    let mut doc = HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {path}: {e}"));
    doc.set_debug_overlay(true);
    doc.render_page_svg_native(page_idx)
        .unwrap_or_else(|e| panic!("render {path} page {}: {e}", page_idx + 1))
}

fn debug_marker_y(svg: &str, marker: &str) -> f64 {
    let start = svg
        .find(marker)
        .unwrap_or_else(|| panic!("debug marker not found: {marker}"));
    let rest = &svg[start..];
    let key = " y=";
    let y_start = rest
        .find(key)
        .unwrap_or_else(|| panic!("debug marker has no y value: {marker}"))
        + key.len();
    let y_end = rest[y_start..]
        .find('<')
        .map(|i| y_start + i)
        .unwrap_or(rest.len());
    rest[y_start..y_end]
        .parse::<f64>()
        .unwrap_or_else(|e| panic!("parse marker y for {marker}: {e}"))
}

#[test]
fn issue_1133_hwp_nested_table_height_exceeds_line_seg_placeholder() {
    assert_issue_1133_nested_table_height_contract("samples/issue_1133.hwp");
}

#[test]
fn issue_1133_hwpx_nested_table_height_exceeds_line_seg_placeholder() {
    assert_issue_1133_nested_table_height_contract("samples/hwpx/issue_1133.hwpx");
}

#[test]
fn issue_1133_hwpx_preserves_gap_between_consecutive_block_tables() {
    let hwp_svg = render_debug_svg("samples/issue_1133.hwp", 1);
    let hwpx_svg = render_debug_svg("samples/hwpx/issue_1133.hwpx", 1);

    let hwp_gap =
        debug_marker_y(&hwp_svg, "s0:pi=29 ci=0") - debug_marker_y(&hwp_svg, "s0:pi=28 ci=0");
    let hwpx_gap =
        debug_marker_y(&hwpx_svg, "s0:pi=29 ci=0") - debug_marker_y(&hwpx_svg, "s0:pi=28 ci=0");

    assert!(
        (hwp_gap - hwpx_gap).abs() < 1.0,
        "HWPX must preserve the same pi=28 -> pi=29 table gap as HWP: hwp={hwp_gap:.1}, hwpx={hwpx_gap:.1}"
    );
}
