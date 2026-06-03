//! Issue #1251: `143E433F503322BD33.hwp` has a chart-like OLE object whose
//! nested OLE container exposes only a legacy `Contents` stream.

use std::fs;
use std::path::Path;

use rhwp::model::bin_data::BinDataType;
use rhwp::ole_chart::{
    ole_chart_ir_json, parse_ole_chart_contents, probe_ole_chart_contents,
    render_ole_chart_standalone_svg, render_ole_chart_svg_fragment, OleChartType,
};
#[cfg(all(not(target_arch = "wasm32"), feature = "charming-renderer"))]
use rhwp::ole_chart::{render_ole_chart_charming_svg, render_smoke_chart_svg};
use rhwp::parser::ole_container::parse_ole_container;

fn read_fixture() -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/143E433F503322BD33.hwp");
    fs::read(&path).unwrap_or_else(|e| panic!("read fixture {}: {}", path.display(), e))
}

fn bin_data_2_raw_contents() -> Vec<u8> {
    let doc = rhwp::parse_document(&read_fixture()).expect("parse fixture");
    let bin_data = doc
        .doc_info
        .bin_data_list
        .get(1)
        .expect("DocInfo BinData #2");

    assert_eq!(bin_data.data_type, BinDataType::Storage);
    assert_eq!(bin_data.storage_id, 2);
    assert_eq!(bin_data.extension.as_deref(), Some("OLE"));

    let content = doc
        .bin_data_content
        .iter()
        .find(|content| content.id == 2)
        .expect("loaded BinData #2 content");
    assert_eq!(content.extension, "OLE");
    assert!(content.data.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]));

    let container = parse_ole_container(&content.data).expect("nested OLE container");
    assert!(
        container.ooxml_chart.is_none(),
        "fixture has no OOXMLChartContents"
    );
    assert!(
        container.preview_emf.is_none(),
        "fixture has no OlePres000 EMF preview"
    );
    assert!(
        container.native_image.is_none(),
        "fixture has no native image preview"
    );
    container.raw_contents.expect("fixture Contents stream")
}

#[test]
fn fixture_has_bin_data_2_ole_contents_only() {
    let raw_contents = bin_data_2_raw_contents();

    assert_eq!(raw_contents.len(), 9876);
    assert_eq!(
        &raw_contents[..16],
        &[
            0x00, 0x00, 0x01, 0x00, 0xEC, 0x2E, 0x00, 0x00, 0xEC, 0x2E, 0x00, 0x00, 0x60, 0x00,
            0x00, 0x00
        ]
    );
}

#[test]
fn ole_chart_contents_probe_is_stable() {
    let raw_contents = bin_data_2_raw_contents();
    let probe = probe_ole_chart_contents(&raw_contents).expect("probe contents");

    assert_eq!(probe.len, 9876);
    assert_eq!(
        probe.first_words_le,
        [0x0001_0000, 0x0000_2EEC, 0x0000_2EEC, 0x0000_0060]
    );
    assert!(!probe.has_cfb_magic);
    assert!(!probe.has_ooxml_chart_marker);
    assert_eq!(probe.legacy_chart_object_start, Some(0x60));
    assert!(probe.has_vt_data_grid_marker);
    assert!(probe.has_vt_chart_title_marker);
    assert!(probe.likely_legacy_hwp_chart_contents);
}

#[test]
fn ole_chart_contents_parse_result_is_stable() {
    let raw_contents = bin_data_2_raw_contents();
    let chart = parse_ole_chart_contents(&raw_contents).expect("parse legacy chart contents");

    assert_eq!(chart.chart_type, OleChartType::Unknown);
    assert_eq!(chart.title.as_deref(), Some("연금 재정 전망"));
    assert_eq!(chart.categories, ["2010년", "2020년", "2030년", "2040년"]);
    assert_eq!(chart.series.len(), 3);
    assert_eq!(chart.series[0].name.as_deref(), Some("적립금"));
    assert_eq!(chart.series[0].values, [328.0, 812.0, 1702.0, 1477.0]);
    assert_eq!(chart.series[1].name.as_deref(), Some("수입"));
    assert_eq!(chart.series[1].values, [50.0, 70.0, 189.0, 191.0]);
    assert_eq!(chart.series[2].name.as_deref(), Some("지출"));
    assert_eq!(chart.series[2].values, [11.0, 15.0, 201.0, 289.0]);
}

#[test]
fn ole_chart_contents_exposes_renderer_neutral_ir() {
    let raw_contents = bin_data_2_raw_contents();
    let chart = parse_ole_chart_contents(&raw_contents).expect("parse legacy chart contents");
    let json = ole_chart_ir_json(&chart).expect("serialize chart ir");

    assert!(json.contains("\"schema\":\"rhwp.oleChartIr\""));
    assert!(json.contains("\"chartType\":\"unknown\""));
    assert!(json.contains("연금 재정 전망"));
    assert!(json.contains("적립금"));
}

#[test]
fn ole_chart_contents_renders_rust_svg_fragment() {
    let raw_contents = bin_data_2_raw_contents();
    let chart = parse_ole_chart_contents(&raw_contents).expect("parse legacy chart contents");
    let svg = render_ole_chart_svg_fragment(&chart, 10.0, 20.0, 420.0, 320.0, 2);

    assert!(svg.contains("hwp-ole-chart-rust-svg"));
    assert!(svg.contains("data-rhwp-ole-chart-renderer=\"rust-svg\""));
    assert!(svg.contains("연금 재정 전망"));
    assert!(svg.contains("적립금"));
    assert!(!svg.contains("charming SSR unavailable"));
}

#[test]
fn ole_chart_contents_renders_standalone_rust_svg() {
    let raw_contents = bin_data_2_raw_contents();
    let chart = parse_ole_chart_contents(&raw_contents).expect("parse legacy chart contents");
    let svg = render_ole_chart_standalone_svg(&chart, 420, 320);

    assert!(svg.starts_with("<svg"), "unexpected SVG prefix: {svg}");
    assert!(svg.contains("연금 재정 전망"));
    assert!(svg.contains("적립금"));
    assert!(!svg.contains("charming SSR unavailable"));
}

#[cfg(all(not(target_arch = "wasm32"), feature = "charming-renderer"))]
#[test]
fn ole_chart_contents_renders_charming_svg_adapter() {
    let raw_contents = bin_data_2_raw_contents();
    let chart = parse_ole_chart_contents(&raw_contents).expect("parse legacy chart contents");
    let svg = render_ole_chart_charming_svg(&chart, 420, 320).expect("render parsed chart");

    assert!(
        svg.starts_with("<svg"),
        "unexpected charming SVG prefix: {svg}"
    );
    assert!(
        svg.contains("연금"),
        "rendered SVG should include chart title"
    );
    assert!(
        svg.contains("적립금"),
        "rendered SVG should include series name"
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn issue_1251_svg_uses_legacy_ole_chart_renderer() {
    let mut doc = rhwp::wasm_api::HwpDocument::from_bytes(&read_fixture()).expect("parse fixture");
    let svg = doc.render_page_svg_native(0).expect("render page svg");

    assert!(svg.contains("hwp-ole-chart"));
    assert!(svg.contains("hwp-ole-chart-rust-svg"));
    assert!(svg.contains("연금 재정 전망"));
    assert!(svg.contains("적립금"));
    assert!(!svg.contains("OLE 개체 (BinData #2)"));
    assert!(!svg.contains("OLE 차트 미지원"));
}

#[cfg(all(not(target_arch = "wasm32"), feature = "charming-renderer"))]
#[test]
fn charming_ssr_smoke_renders_svg_string() {
    let svg = render_smoke_chart_svg(420, 320).expect("charming SSR SVG render");

    assert!(
        svg.starts_with("<svg"),
        "unexpected charming SVG prefix: {svg}"
    );
    assert!(
        svg.contains("alpha"),
        "charming SVG should include sample data labels"
    );
}
