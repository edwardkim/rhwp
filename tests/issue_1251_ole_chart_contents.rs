//! Issue #1251: `143E433F503322BD33.hwp` has a chart-like OLE object whose
//! nested OLE container exposes only a legacy `Contents` stream.

use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::bin_data::BinDataType;
use rhwp::model::control::Control;
use rhwp::model::shape::ShapeObject;
use rhwp::ole_chart::{
    ole_chart_ir_json, parse_ole_chart_contents, probe_ole_chart_contents,
    render_ole_chart_standalone_svg, render_ole_chart_svg_fragment, OleChartType,
};
use rhwp::parser::{cfb_reader::decompress_stream, ole_container::parse_ole_container};
use rhwp::parser::{record::Record, tags};

fn read_fixture() -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/143E433F503322BD33.hwp");
    fs::read(&path).unwrap_or_else(|e| panic!("read fixture {}: {}", path.display(), e))
}

fn read_hwpx_fixture() -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwpx/143E433F503322BD33.hwpx");
    fs::read(&path).unwrap_or_else(|e| panic!("read fixture {}: {}", path.display(), e))
}

fn read_cfb_stream(bytes: &[u8], path: &str) -> Vec<u8> {
    let cursor = Cursor::new(bytes);
    let mut cfb = cfb::CompoundFile::open(cursor).expect("open CFB");
    let mut stream = cfb
        .open_stream(path)
        .unwrap_or_else(|e| panic!("open stream {path}: {e}"));
    let mut data = Vec::new();
    stream
        .read_to_end(&mut data)
        .unwrap_or_else(|e| panic!("read stream {path}: {e}"));
    data
}

fn section0_records(bytes: &[u8]) -> Vec<Record> {
    let raw = read_cfb_stream(bytes, "/BodyText/Section0");
    let section = decompress_stream(&raw).unwrap_or(raw);
    Record::read_all(&section).expect("parse Section0 records")
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

fn hwpx_bin_data_3_raw_contents() -> Vec<u8> {
    let doc = rhwp::parse_document(&read_hwpx_fixture()).expect("parse hwpx fixture");
    let bin_data = doc
        .doc_info
        .bin_data_list
        .get(2)
        .expect("DocInfo BinData #3");

    assert_eq!(bin_data.data_type, BinDataType::Storage);
    assert_eq!(bin_data.storage_id, 3);
    assert_eq!(bin_data.extension.as_deref(), Some("OLE"));

    let content = doc
        .bin_data_content
        .iter()
        .find(|content| content.id == 3)
        .expect("loaded HWPX BinData #3 content");
    assert_eq!(content.extension, "OLE");
    assert!(content.data.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]));

    let container = parse_ole_container(&content.data).expect("nested HWPX OLE container");
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

#[test]
fn issue_1283_hwpx_internal_ole_is_loaded_even_when_isembeded_zero() {
    let raw_contents = hwpx_bin_data_3_raw_contents();
    assert_eq!(raw_contents.len(), 9876);

    let chart = parse_ole_chart_contents(&raw_contents).expect("parse HWPX legacy chart contents");
    assert_eq!(chart.title.as_deref(), Some("연금 재정 전망"));
    assert_eq!(chart.series.len(), 3);
    assert_eq!(chart.series[0].name.as_deref(), Some("적립금"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn issue_1283_hwpx_svg_uses_legacy_ole_chart_renderer() {
    let mut doc =
        rhwp::wasm_api::HwpDocument::from_bytes(&read_hwpx_fixture()).expect("parse hwpx fixture");
    let svg = doc.render_page_svg_native(0).expect("render HWPX page svg");

    assert!(svg.contains("hwp-ole-chart"));
    assert!(svg.contains("hwp-ole-chart-rust-svg"));
    assert!(svg.contains("연금 재정 전망"));
    assert!(svg.contains("적립금"));
    assert!(!svg.contains("OLE 개체 (BinData #3)"));
    assert!(!svg.contains("OLE 차트 미지원"));
}

#[test]
fn issue_1283_hwpx_to_hwp_save_keeps_ole_as_storage() {
    let mut core = DocumentCore::from_bytes(&read_hwpx_fixture()).expect("parse hwpx fixture");
    let hwp = core.export_hwp_with_adapter().expect("export HWP");
    let reparsed = rhwp::parse_document(&hwp).expect("reparse exported HWP");

    let bin_data = reparsed
        .doc_info
        .bin_data_list
        .get(1)
        .expect("exported DocInfo BinData #2");
    assert_eq!(bin_data.data_type, BinDataType::Storage);
    assert_eq!(bin_data.attr, 0x0002);
    assert_eq!(bin_data.storage_id, 2);
    assert_eq!(bin_data.extension.as_deref(), Some("OLE"));

    let exported_ole = reparsed.sections[0]
        .paragraphs
        .iter()
        .flat_map(|para| para.controls.iter())
        .find_map(|control| match control {
            Control::Shape(shape) => match shape.as_ref() {
                ShapeObject::Ole(ole) => Some(ole),
                _ => None,
            },
            _ => None,
        })
        .expect("exported OLE control");
    assert_eq!(exported_ole.bin_data_id, 2);
    assert_eq!(exported_ole.extent_x, 7200);
    assert_eq!(exported_ole.extent_y, 7200);
    assert_eq!(exported_ole.raw_tag_data.len(), 26);
    assert_eq!(
        exported_ole.common.attr, 0x143A_2610,
        "HWPX OLE GenShape CTRL_HEADER attr must preserve Hancom flow/protect/numbering bits"
    );
    assert!(exported_ole.common.flow_with_text);
    assert!(exported_ole.common.size_protect);
    assert!(exported_ole.common.hwp5_gen_shape_attr_bit26);
    assert!(exported_ole.common.hwp5_gen_shape_attr_bit28);
    assert_eq!(
        exported_ole.drawing.shape_attr.ctrl_id,
        rhwp::parser::tags::SHAPE_OLE_ID
    );
    assert!(exported_ole.drawing.shape_attr.is_two_ctrl_id);
    assert_eq!(exported_ole.drawing.shape_attr.local_file_version, 1);
    assert_eq!(exported_ole.drawing.shape_attr.original_width, 7200);
    assert_eq!(exported_ole.drawing.shape_attr.original_height, 7200);
    assert_eq!(exported_ole.drawing.shape_attr.current_width, 7200);
    assert_eq!(exported_ole.drawing.shape_attr.current_height, 7200);

    let content = reparsed
        .bin_data_content
        .iter()
        .find(|content| content.id == 2)
        .expect("exported OLE BinData #2 content");
    assert_eq!(content.extension, "OLE");
    assert!(content.data.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]));

    let ole_stream = read_cfb_stream(&hwp, "/BinData/BIN0002.OLE");
    let ole_payload =
        decompress_stream(&ole_stream).expect("HWP OLE Storage stream must be deflated");
    assert_eq!(
        &ole_payload[..4],
        &(content.data.len() as u32).to_le_bytes(),
        "HWP OLE Storage payload must begin with a size prefix after decompression"
    );
    assert_eq!(
        &ole_payload[4..12],
        &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1],
        "HWP OLE Storage payload must store the nested CFB after the size prefix"
    );

    let records = section0_records(&hwp);
    let hwpx_shape_ctrl_data_contract = [
        0x1b, 0x02, 0x01, 0x00, 0x00, 0x00, 0x03, 0x30, 0x00, 0x80, 0x03, 0x30, 0x01, 0x00, 0x00,
        0x00, 0x01, 0x70, 0x09, 0x00, 0x01, 0x00, 0x00, 0x00,
    ];
    let matching_ctrl_data_count = records
        .iter()
        .filter(|record| {
            record.tag_id == tags::HWPTAG_CTRL_DATA && record.data == hwpx_shape_ctrl_data_contract
        })
        .count();
    assert_eq!(
        matching_ctrl_data_count, 2,
        "HWPX dropcap-like text-box shape must materialize Hancom's duplicated CTRL_DATA contract"
    );
    let bookmark_ctrl_data_contract = [
        0x1b, 0x02, 0x01, 0x00, 0x00, 0x00, 0x00, 0x40, 0x01, 0x00, 0x02, 0x00, 0x38, 0xcc, 0x70,
        0xc8,
    ];
    assert_eq!(
        records
            .iter()
            .filter(|record| {
                record.tag_id == tags::HWPTAG_CTRL_DATA
                    && record.data == bookmark_ctrl_data_contract
            })
            .count(),
        1,
        "HWPX bookmark name must materialize as Hancom CTRL_DATA, not inline CTRL_HEADER bytes"
    );
    let bookmark_header = records
        .iter()
        .find(|record| {
            record.tag_id == tags::HWPTAG_CTRL_HEADER
                && record.data.len() >= 4
                && record.data[..4] == tags::CTRL_BOOKMARK.to_le_bytes()
        })
        .expect("bookmark CTRL_HEADER");
    assert_eq!(
        bookmark_header.data.len(),
        4,
        "bookmark CTRL_HEADER must contain only ctrl_id when CTRL_DATA carries the name"
    );

    let ole_pos = records
        .iter()
        .position(|record| record.tag_id == tags::HWPTAG_SHAPE_COMPONENT_OLE)
        .expect("SHAPE_COMPONENT_OLE record");
    let shape_component = &records[ole_pos - 1];
    assert_eq!(shape_component.tag_id, tags::HWPTAG_SHAPE_COMPONENT);
    assert_eq!(shape_component.size, 196);
    let ole_id = tags::SHAPE_OLE_ID.to_le_bytes();
    assert_eq!(&shape_component.data[..4], &ole_id);
    assert_eq!(&shape_component.data[4..8], &ole_id);
    assert_eq!(&shape_component.data[18..20], &1u16.to_le_bytes());
    assert_eq!(&shape_component.data[20..24], &7200u32.to_le_bytes());
    assert_eq!(&shape_component.data[24..28], &7200u32.to_le_bytes());
    assert_eq!(&shape_component.data[28..32], &7200u32.to_le_bytes());
    assert_eq!(&shape_component.data[32..36], &7200u32.to_le_bytes());

    let container = parse_ole_container(&content.data).expect("exported nested OLE container");
    let raw_contents = container.raw_contents.expect("exported Contents stream");
    let chart = parse_ole_chart_contents(&raw_contents).expect("parse exported chart contents");
    assert_eq!(chart.title.as_deref(), Some("연금 재정 전망"));
}
