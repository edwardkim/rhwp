//! Issue #493: 셀 보호, 셀 필드 이름, 양식 모드 편집 가능 속성 회귀 가드.

use std::fs;
use std::io::Read;
use std::path::Path;

use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::parser::hwpx::parse_hwpx;
use rhwp::serializer::hwpx::serialize_hwpx;
use rhwp::{parse_document, wasm_api::HwpDocument};
use serde_json::Value;

#[derive(Clone, Copy)]
struct TablePos {
    section: usize,
    para: usize,
    control: usize,
}

fn sample_bytes(rel: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}

fn find_first_table(doc: &Document) -> TablePos {
    for (section, section_model) in doc.sections.iter().enumerate() {
        for (para, paragraph) in section_model.paragraphs.iter().enumerate() {
            for (control, ctrl) in paragraph.controls.iter().enumerate() {
                if matches!(ctrl, Control::Table(_)) {
                    return TablePos {
                        section,
                        para,
                        control,
                    };
                }
            }
        }
    }
    panic!("sample should contain a table");
}

fn assert_cell_attrs(doc: &Document, pos: TablePos) {
    let Control::Table(table) =
        &doc.sections[pos.section].paragraphs[pos.para].controls[pos.control]
    else {
        panic!("expected table control");
    };
    assert!(table.cells.len() >= 3, "sample should contain >= 3 cells");

    assert!(table.cells[0].cell_protect(), "0번 셀은 보호 상태");
    assert!(table.cells[1].cell_protect(), "1번 셀은 보호 상태");
    assert!(table.cells[2].cell_protect(), "2번 셀은 보호 상태");
    assert!(
        table.cells.iter().all(|cell| !cell.apply_inner_margin),
        "셀보호 샘플은 모든 셀의 안 여백 지정이 꺼져 있어야 함"
    );

    assert_eq!(table.cells[2].field_name.as_deref(), Some("name"));
    assert!(
        table.cells[2].editable_in_form(),
        "필드 셀은 양식 모드 편집 가능"
    );
}

fn assert_api_attrs(bytes: &[u8], pos: TablePos) {
    let doc = HwpDocument::from_bytes(bytes).expect("load HwpDocument");
    let json = doc
        .get_cell_properties(pos.section as u32, pos.para as u32, pos.control as u32, 2)
        .expect("getCellProperties");
    let props: Value = serde_json::from_str(&json).expect("parse cell properties");
    assert_eq!(props["cellProtect"].as_bool(), Some(true), "{json}");
    assert_eq!(props["fieldName"].as_str(), Some("name"), "{json}");
    assert_eq!(props["editableInForm"].as_bool(), Some(true), "{json}");
    assert_eq!(props["applyInnerMargin"].as_bool(), Some(false), "{json}");

    let fields: Value = serde_json::from_str(&doc.get_field_list()).expect("parse getFieldList");
    let field = fields
        .as_array()
        .expect("field list array")
        .iter()
        .find(|field| field["name"].as_str() == Some("name"))
        .expect("cell field in getFieldList");
    assert_eq!(field["value"].as_str(), Some("12334"), "{fields}");
    assert_eq!(field["editableInForm"].as_bool(), Some(true), "{fields}");
}

fn assert_inner_margin_sample_attrs(doc: &Document, pos: TablePos) {
    let Control::Table(table) =
        &doc.sections[pos.section].paragraphs[pos.para].controls[pos.control]
    else {
        panic!("expected table control");
    };
    assert_eq!(table.cells.len(), 25, "셀보호2 샘플은 25개 셀");
    let explicit_cells: Vec<_> = table
        .cells
        .iter()
        .enumerate()
        .filter(|(_, cell)| cell.apply_inner_margin)
        .collect();
    assert_eq!(
        explicit_cells.len(),
        1,
        "셀보호2 샘플은 한컴 기준 1개 셀만 안 여백 지정 상태"
    );

    let cell = &table.cells[20];
    assert!(
        cell.apply_inner_margin,
        "마지막 행 첫 셀은 안 여백 지정 상태"
    );
    assert_eq!(cell.padding.left, 2834, "좌측 안 여백은 10mm");
    assert_eq!(cell.padding.right, 2834, "우측 안 여백은 10mm");
    assert_eq!(cell.padding.top, 0, "상단 안 여백은 0mm");
    assert_eq!(cell.padding.bottom, 0, "하단 안 여백은 0mm");
    assert_eq!(cell.paragraphs[0].text, "12345");

    let text_starts: Vec<_> = cell.paragraphs[0]
        .line_segs
        .iter()
        .map(|seg| seg.text_start)
        .collect();
    assert_eq!(
        text_starts,
        vec![0, 2, 4],
        "한컴은 좌우 10mm 안 여백 셀을 12/34/5 세 줄로 저장한다"
    );
}

fn table_common_offsets(doc: &HwpDocument, pos: TablePos) -> (i32, i32) {
    let Control::Table(table) =
        &doc.document().sections[pos.section].paragraphs[pos.para].controls[pos.control]
    else {
        panic!("expected table control");
    };
    (
        table.common.horizontal_offset as i32,
        table.common.vertical_offset as i32,
    )
}

fn table_cell_bbox_value(doc: &HwpDocument, pos: TablePos, cell_idx: u64, key: &str) -> f64 {
    let json = doc
        .get_table_cell_bboxes(
            pos.section as u32,
            pos.para as u32,
            pos.control as u32,
            Some(0),
        )
        .expect("get table cell bboxes");
    let bboxes: Value = serde_json::from_str(&json).expect("parse table cell bboxes");
    bboxes
        .as_array()
        .expect("bbox array")
        .iter()
        .find(|bbox| bbox["cellIdx"].as_u64() == Some(cell_idx))
        .and_then(|bbox| bbox[key].as_f64())
        .unwrap_or_else(|| panic!("cell {cell_idx} bbox key {key} not found: {json}"))
}

fn table_cell_property_i64(doc: &HwpDocument, pos: TablePos, cell_idx: u32, key: &str) -> i64 {
    let json = doc
        .get_cell_properties(
            pos.section as u32,
            pos.para as u32,
            pos.control as u32,
            cell_idx,
        )
        .expect("get cell properties");
    let props: Value = serde_json::from_str(&json).expect("parse cell properties");
    props[key]
        .as_i64()
        .unwrap_or_else(|| panic!("cell {cell_idx} property {key} not found: {json}"))
}

fn table_cell_render_width_hu(doc: &HwpDocument, pos: TablePos, cell_idx: u32) -> i64 {
    (table_cell_bbox_value(doc, pos, cell_idx as u64, "w") * 75.0).round() as i64
}

fn table_cell_render_height_hu(doc: &HwpDocument, pos: TablePos, cell_idx: u32) -> i64 {
    (table_cell_bbox_value(doc, pos, cell_idx as u64, "h") * 75.0).round() as i64
}

fn resize_cells_to_render_widths(doc: &mut HwpDocument, pos: TablePos, widths: &[(u32, i64)]) {
    let updates = widths
        .iter()
        .map(|(idx, render_width)| {
            let model_width = table_cell_property_i64(doc, pos, *idx, "width");
            format!(
                r#"{{"cellIdx":{idx},"widthDelta":{},"localResize":true,"renderWidth":{render_width}}}"#,
                render_width - model_width
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    doc.resize_table_cells(
        pos.section as u32,
        pos.para as u32,
        pos.control as u32,
        &format!("[{updates}]"),
    )
    .expect("resize cells to render widths");
}

fn resize_cells_to_render_heights(doc: &mut HwpDocument, pos: TablePos, heights: &[(u32, i64)]) {
    let updates = heights
        .iter()
        .map(|(idx, render_height)| {
            format!(
                r#"{{"cellIdx":{idx},"heightDelta":0,"localResize":true,"renderHeight":{render_height}}}"#,
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    doc.resize_table_cells(
        pos.section as u32,
        pos.para as u32,
        pos.control as u32,
        &format!("[{updates}]"),
    )
    .expect("resize cells to render heights");
}

fn hwpx_section0_xml(bytes: &[u8]) -> String {
    let reader = std::io::Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(reader).expect("open hwpx zip");
    for index in 0..zip.len() {
        let mut file = zip.by_index(index).expect("zip entry");
        if file.name().contains("section0.xml") {
            let mut xml = String::new();
            file.read_to_string(&mut xml).expect("read section0.xml");
            return xml;
        }
    }
    panic!("section0.xml not found");
}

fn named_cell_opening_tag(xml: &str) -> &str {
    let start = xml
        .find(r#"<hp:tc name="name""#)
        .expect("named cell opening tag");
    let end = xml[start..].find('>').expect("opening tag end") + start;
    &xml[start..=end]
}

#[test]
fn cell_protect_field_name_and_form_editable_are_parsed_from_hwp_and_hwpx() {
    for rel in ["samples/셀보호.hwp", "samples/셀보호.hwpx"] {
        let bytes = sample_bytes(rel);
        let doc = parse_document(&bytes).unwrap_or_else(|e| panic!("parse {rel}: {e:?}"));
        let pos = find_first_table(&doc);
        assert_cell_attrs(&doc, pos);
        assert_api_attrs(&bytes, pos);
    }
}

#[test]
fn table_properties_for_cellprotect2_match_hancom_common_object_attrs() {
    let bytes = sample_bytes("samples/셀보호2.hwp");
    let parsed = parse_document(&bytes).expect("parse 셀보호2.hwp");
    let pos = find_first_table(&parsed);
    let doc = HwpDocument::from_bytes(&bytes).expect("load HwpDocument");

    let json = doc
        .get_table_properties(pos.section as u32, pos.para as u32, pos.control as u32)
        .expect("get table properties");
    let props: Value = serde_json::from_str(&json).expect("parse table properties");

    assert_eq!(props["tableWidth"].as_u64(), Some(43190), "{json}");
    assert_eq!(props["tableHeight"].as_u64(), Some(17932), "{json}");
    assert_eq!(props["horzOffset"].as_i64(), Some(4), "{json}");
    assert_eq!(props["vertOffset"].as_i64(), Some(22133), "{json}");
    assert_eq!(props["horzRelTo"].as_str(), Some("Column"), "{json}");
    assert_eq!(props["vertRelTo"].as_str(), Some("Para"), "{json}");
    assert_eq!(props["textWrap"].as_str(), Some("TopAndBottom"), "{json}");
    assert_eq!(props["treatAsChar"].as_bool(), Some(false), "{json}");
}

#[test]
fn table_move_keeps_raw_and_common_offsets_in_sync() {
    let bytes = sample_bytes("samples/셀보호2.hwp");
    let parsed = parse_document(&bytes).expect("parse 셀보호2.hwp");
    let pos = find_first_table(&parsed);
    let mut doc = HwpDocument::from_bytes(&bytes).expect("load HwpDocument");

    let before_json = doc
        .get_table_properties(pos.section as u32, pos.para as u32, pos.control as u32)
        .expect("get before table properties");
    let before: Value = serde_json::from_str(&before_json).expect("parse before table properties");
    let before_horz = before["horzOffset"].as_i64().expect("before horzOffset") as i32;
    let before_vert = before["vertOffset"].as_i64().expect("before vertOffset") as i32;

    doc.move_table_offset(
        pos.section as u32,
        pos.para as u32,
        pos.control as u32,
        750,
        -375,
    )
    .expect("move table offset");

    let after_json = doc
        .get_table_properties(pos.section as u32, pos.para as u32, pos.control as u32)
        .expect("get after table properties");
    let after: Value = serde_json::from_str(&after_json).expect("parse after table properties");
    let expected_horz = before_horz + 750;
    let expected_vert = before_vert - 375;

    assert_eq!(
        after["horzOffset"].as_i64(),
        Some(expected_horz as i64),
        "{after_json}"
    );
    assert_eq!(
        after["vertOffset"].as_i64(),
        Some(expected_vert as i64),
        "{after_json}"
    );
    assert_eq!(
        table_common_offsets(&doc, pos),
        (expected_horz, expected_vert),
        "렌더링이 참조하는 table.common 위치도 raw_ctrl_data와 동기화되어야 함"
    );
}

#[test]
fn compensated_cell_resize_keeps_cellprotect2_table_common_size() {
    let bytes = sample_bytes("samples/셀보호2.hwp");
    let parsed = parse_document(&bytes).expect("parse 셀보호2.hwp");
    let pos = find_first_table(&parsed);
    let mut doc = HwpDocument::from_bytes(&bytes).expect("load HwpDocument");

    let before_json = doc
        .get_table_properties(pos.section as u32, pos.para as u32, pos.control as u32)
        .expect("get before table properties");
    let before: Value = serde_json::from_str(&before_json).expect("parse before table properties");
    let before_width = before["tableWidth"].as_u64().expect("before tableWidth");
    let before_height = before["tableHeight"].as_u64().expect("before tableHeight");
    let before_cell22_x = table_cell_bbox_value(&doc, pos, 22, "x");

    doc.resize_table_cells(
        pos.section as u32,
        pos.para as u32,
        pos.control as u32,
        r#"[
            {"cellIdx":20,"widthDelta":1200},
            {"cellIdx":21,"widthDelta":-1200},
            {"cellIdx":20,"heightDelta":300},
            {"cellIdx":21,"heightDelta":-300}
        ]"#,
    )
    .expect("compensated resize table cells");

    let after_json = doc
        .get_table_properties(pos.section as u32, pos.para as u32, pos.control as u32)
        .expect("get after table properties");
    let after: Value = serde_json::from_str(&after_json).expect("parse after table properties");
    let after_cell20_w = table_cell_bbox_value(&doc, pos, 20, "w");
    let after_cell22_x = table_cell_bbox_value(&doc, pos, 22, "x");
    assert_eq!(
        after["tableWidth"].as_u64(),
        Some(before_width),
        "보상 셀 너비 조절은 표 common width를 바꾸면 안 됨: {after_json}"
    );
    assert_eq!(
        after["tableHeight"].as_u64(),
        Some(before_height),
        "보상 셀 높이 조절은 표 common height를 바꾸면 안 됨: {after_json}"
    );
    assert!(
        after_cell20_w > 90.0,
        "대상 셀은 실제 렌더 bbox 폭이 커져야 함: {after_cell20_w}"
    );
    assert!(
        (after_cell22_x - before_cell22_x).abs() <= 0.2,
        "보상 셀 너비 조절은 뒤쪽 셀 x를 밀면 안 됨: before={before_cell22_x}, after={after_cell22_x}"
    );
}

#[test]
fn compensated_colspan_cell_resize_does_not_leak_to_other_rows() {
    let bytes = sample_bytes("samples/셀보호2.hwp");
    let parsed = parse_document(&bytes).expect("parse 셀보호2.hwp");
    let pos = find_first_table(&parsed);
    let mut doc = HwpDocument::from_bytes(&bytes).expect("load HwpDocument");

    let before_cell0_w = table_cell_bbox_value(&doc, pos, 0, "w");
    let before_cell5_w = table_cell_bbox_value(&doc, pos, 5, "w");
    let before_cell6_w = table_cell_bbox_value(&doc, pos, 6, "w");
    let before_cell7_x = table_cell_bbox_value(&doc, pos, 7, "x");
    let before_cell10_w = table_cell_bbox_value(&doc, pos, 10, "w");
    let drag_delta_hu = 1200_i64;
    let cell5_model_w = table_cell_property_i64(&doc, pos, 5, "width");
    let cell6_model_w = table_cell_property_i64(&doc, pos, 6, "width");
    let cell7_render_w = (table_cell_bbox_value(&doc, pos, 7, "w") * 75.0).round() as i64;
    let cell8_render_w = (table_cell_bbox_value(&doc, pos, 8, "w") * 75.0).round() as i64;
    let cell9_render_w = (table_cell_bbox_value(&doc, pos, 9, "w") * 75.0).round() as i64;
    let cell5_render_w = (before_cell5_w * 75.0).round() as i64 + drag_delta_hu;
    let cell6_render_w = (before_cell6_w * 75.0).round() as i64 - drag_delta_hu;
    let cell5_delta = cell5_render_w - cell5_model_w;
    let cell6_delta = cell6_render_w - cell6_model_w;

    doc.resize_table_cells(
        pos.section as u32,
        pos.para as u32,
        pos.control as u32,
        &format!(
            r#"[
            {{"cellIdx":5,"widthDelta":{cell5_delta},"localResize":true,"renderWidth":{cell5_render_w}}},
            {{"cellIdx":6,"widthDelta":{cell6_delta},"localResize":true,"renderWidth":{cell6_render_w}}},
            {{"cellIdx":7,"widthDelta":0,"localResize":true,"renderWidth":{cell7_render_w}}},
            {{"cellIdx":8,"widthDelta":0,"localResize":true,"renderWidth":{cell8_render_w}}},
            {{"cellIdx":9,"widthDelta":0,"localResize":true,"renderWidth":{cell9_render_w}}}
        ]"#
        ),
    )
    .expect("compensated resize merged row cells");

    let after_cell0_w = table_cell_bbox_value(&doc, pos, 0, "w");
    let after_cell5_w = table_cell_bbox_value(&doc, pos, 5, "w");
    let after_cell7_x = table_cell_bbox_value(&doc, pos, 7, "x");
    let after_cell10_w = table_cell_bbox_value(&doc, pos, 10, "w");

    assert!(
        after_cell5_w > before_cell5_w + 10.0,
        "대상 병합 셀 폭은 커져야 함: before={before_cell5_w}, after={after_cell5_w}"
    );
    assert!(
        (after_cell7_x - before_cell7_x).abs() <= 0.2,
        "보상 셀 너비 조절은 같은 행의 뒤쪽 셀 x를 밀면 안 됨: before={before_cell7_x}, after={after_cell7_x}"
    );
    assert!(
        (after_cell0_w - before_cell0_w).abs() <= 0.2,
        "보상 셀 너비 조절은 위 행 폭을 바꾸면 안 됨: before={before_cell0_w}, after={after_cell0_w}"
    );
    assert!(
        (after_cell10_w - before_cell10_w).abs() <= 0.2,
        "보상 셀 너비 조절은 아래 행 폭을 바꾸면 안 됨: before={before_cell10_w}, after={after_cell10_w}"
    );
}

#[test]
fn local_resize_render_width_keeps_untouched_bottom_cells_stable() {
    let bytes = sample_bytes("samples/셀보호2.hwp");
    let parsed = parse_document(&bytes).expect("parse 셀보호2.hwp");
    let pos = find_first_table(&parsed);
    let mut doc = HwpDocument::from_bytes(&bytes).expect("load HwpDocument");

    let before_cell20_w = table_cell_bbox_value(&doc, pos, 20, "w");
    let before_cell21_w = table_cell_bbox_value(&doc, pos, 21, "w");
    let before_cell22_x = table_cell_bbox_value(&doc, pos, 22, "x");
    let before_cell23_x = table_cell_bbox_value(&doc, pos, 23, "x");
    let before_cell23_w = table_cell_bbox_value(&doc, pos, 23, "w");
    let before_cell24_x = table_cell_bbox_value(&doc, pos, 24, "x");
    let before_cell22_w = table_cell_bbox_value(&doc, pos, 22, "w");
    let before_cell24_w = table_cell_bbox_value(&doc, pos, 24, "w");
    let drag_delta_hu = 2400_i64;
    let cell20_model_w = table_cell_property_i64(&doc, pos, 20, "width");
    let cell21_model_w = table_cell_property_i64(&doc, pos, 21, "width");
    let cell20_render_w = (before_cell20_w * 75.0).round() as i64 + drag_delta_hu;
    let cell21_render_w = (before_cell21_w * 75.0).round() as i64 - drag_delta_hu;
    let cell22_render_w = (before_cell22_w * 75.0).round() as i64;
    let cell23_render_w = (before_cell23_w * 75.0).round() as i64;
    let cell24_render_w = (before_cell24_w * 75.0).round() as i64;
    let cell20_delta = cell20_render_w - cell20_model_w;
    let cell21_delta = cell21_render_w - cell21_model_w;

    doc.resize_table_cells(
        pos.section as u32,
        pos.para as u32,
        pos.control as u32,
        &format!(
            r#"[
            {{"cellIdx":20,"widthDelta":{cell20_delta},"localResize":true,"renderWidth":{cell20_render_w}}},
            {{"cellIdx":21,"widthDelta":{cell21_delta},"localResize":true,"renderWidth":{cell21_render_w}}},
            {{"cellIdx":22,"widthDelta":0,"localResize":true,"renderWidth":{cell22_render_w}}},
            {{"cellIdx":23,"widthDelta":0,"localResize":true,"renderWidth":{cell23_render_w}}},
            {{"cellIdx":24,"widthDelta":0,"localResize":true,"renderWidth":{cell24_render_w}}}
        ]"#
        ),
    )
    .expect("local resize bottom row cells");

    let after_cell20_w = table_cell_bbox_value(&doc, pos, 20, "w");
    let after_cell21_w = table_cell_bbox_value(&doc, pos, 21, "w");
    let after_cell22_x = table_cell_bbox_value(&doc, pos, 22, "x");
    let after_cell23_x = table_cell_bbox_value(&doc, pos, 23, "x");
    let after_cell23_w = table_cell_bbox_value(&doc, pos, 23, "w");
    let after_cell24_x = table_cell_bbox_value(&doc, pos, 24, "x");

    assert!(
        after_cell20_w > before_cell20_w + 20.0,
        "대상 셀은 커져야 함: before={before_cell20_w}, after={after_cell20_w}"
    );
    assert!(
        after_cell21_w < before_cell21_w - 20.0,
        "이웃 셀은 줄어야 함: before={before_cell21_w}, after={after_cell21_w}"
    );
    assert!(
        (after_cell22_x - before_cell22_x).abs() <= 0.2,
        "뒤쪽 셀 x는 유지되어야 함: before={before_cell22_x}, after={after_cell22_x}"
    );
    assert!(
        (after_cell23_x - before_cell23_x).abs() <= 0.2
            && (after_cell23_w - before_cell23_w).abs() <= 0.2
            && (after_cell24_x - before_cell24_x).abs() <= 0.2,
        "건드리지 않은 뒤쪽 셀은 유지되어야 함: 23x {before_cell23_x}->{after_cell23_x}, 23w {before_cell23_w}->{after_cell23_w}, 24x {before_cell24_x}->{after_cell24_x}"
    );
}

#[test]
fn cell_width_equal_local_hints_keep_selected_row_independent() {
    let bytes = sample_bytes("samples/셀보호2.hwp");
    let parsed = parse_document(&bytes).expect("parse 셀보호2.hwp");
    let pos = find_first_table(&parsed);
    let mut doc = HwpDocument::from_bytes(&bytes).expect("load HwpDocument");

    let (selected_row, selected_cells) = {
        let Control::Table(table) =
            &doc.document().sections[pos.section].paragraphs[pos.para].controls[pos.control]
        else {
            panic!("expected table control");
        };
        let mut rows = std::collections::BTreeMap::<u16, Vec<u32>>::new();
        for (idx, cell) in table.cells.iter().enumerate() {
            if cell.row_span == 1 {
                rows.entry(cell.row).or_default().push(idx as u32);
            }
        }
        rows.into_iter()
            .find(|(_, cells)| {
                if cells.len() < 3 {
                    return false;
                }
                if !cells
                    .iter()
                    .any(|idx| table.cells[*idx as usize].col_span > 1)
                {
                    return false;
                }
                let widths: Vec<_> = cells
                    .iter()
                    .map(|idx| table_cell_render_width_hu(&doc, pos, *idx))
                    .collect();
                widths.iter().any(|width| *width != widths[0])
            })
            .expect("샘플에는 폭이 다른 가로 병합 포함 행이 있어야 함")
    };
    let stable_cell = {
        let Control::Table(table) =
            &doc.document().sections[pos.section].paragraphs[pos.para].controls[pos.control]
        else {
            panic!("expected table control");
        };
        table
            .cells
            .iter()
            .enumerate()
            .find(|(_, cell)| cell.row != selected_row && cell.row_span == 1 && cell.col_span == 1)
            .map(|(idx, _)| idx as u64)
            .expect("선택 행 밖의 안정성 확인 셀이 있어야 함")
    };

    let before_json = doc
        .get_table_properties(pos.section as u32, pos.para as u32, pos.control as u32)
        .expect("get before table properties");
    let before: Value = serde_json::from_str(&before_json).expect("parse before table properties");
    let before_width = before["tableWidth"].as_u64().expect("before tableWidth");
    let before_stable_x = table_cell_bbox_value(&doc, pos, stable_cell, "x");
    let before_stable_w = table_cell_bbox_value(&doc, pos, stable_cell, "w");
    let before_widths: Vec<_> = selected_cells
        .iter()
        .map(|idx| table_cell_render_width_hu(&doc, pos, *idx))
        .collect();
    assert!(
        before_widths.iter().any(|width| *width != before_widths[0]),
        "테스트 전 선택 행 폭은 서로 달라야 함: {before_widths:?}"
    );

    let avg_width =
        (before_widths.iter().sum::<i64>() as f64 / before_widths.len() as f64).round() as i64;
    let updates = selected_cells
        .iter()
        .map(|idx| {
            let model_width = table_cell_property_i64(&doc, pos, *idx, "width");
            format!(
                r#"{{"cellIdx":{idx},"widthDelta":{},"localResize":true,"renderWidth":{avg_width}}}"#,
                avg_width - model_width
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    doc.resize_table_cells(
        pos.section as u32,
        pos.para as u32,
        pos.control as u32,
        &format!("[{updates}]"),
    )
    .expect("cell width equal local resize");

    let after_widths: Vec<_> = selected_cells
        .iter()
        .map(|idx| table_cell_render_width_hu(&doc, pos, *idx))
        .collect();
    for width in &after_widths {
        assert!(
            (*width - avg_width).abs() <= 1,
            "선택 셀 표시 폭은 평균 폭으로 같아져야 함: avg={avg_width}, after={after_widths:?}"
        );
    }
    let after_json = doc
        .get_table_properties(pos.section as u32, pos.para as u32, pos.control as u32)
        .expect("get after table properties");
    let after: Value = serde_json::from_str(&after_json).expect("parse after table properties");
    let after_stable_x = table_cell_bbox_value(&doc, pos, stable_cell, "x");
    let after_stable_w = table_cell_bbox_value(&doc, pos, stable_cell, "w");

    assert_eq!(
        after["tableWidth"].as_u64(),
        Some(before_width),
        "로컬 너비 균등화는 표 common width를 흔들면 안 됨: {after_json}"
    );
    assert!(
        (after_stable_x - before_stable_x).abs() <= 0.2
            && (after_stable_w - before_stable_w).abs() <= 0.2,
        "선택 행 밖 셀은 전역 grid 회귀로 흔들리면 안 됨: x {before_stable_x}->{after_stable_x}, w {before_stable_w}->{after_stable_w}"
    );
}

#[test]
fn local_then_global_column_resize_preserves_unaffected_rows() {
    let bytes = sample_bytes("samples/셀보호2.hwp");
    let parsed = parse_document(&bytes).expect("parse 셀보호2.hwp");
    let pos = find_first_table(&parsed);
    let mut doc = HwpDocument::from_bytes(&bytes).expect("load HwpDocument");

    let local_delta_hu = 1200_i64;
    let cell5_before_w = table_cell_bbox_value(&doc, pos, 5, "w");
    let cell6_before_w = table_cell_bbox_value(&doc, pos, 6, "w");
    let cell5_model_w = table_cell_property_i64(&doc, pos, 5, "width");
    let cell6_model_w = table_cell_property_i64(&doc, pos, 6, "width");
    let cell5_render_w = (cell5_before_w * 75.0).round() as i64 + local_delta_hu;
    let cell6_render_w = (cell6_before_w * 75.0).round() as i64 - local_delta_hu;
    let cell7_render_w = (table_cell_bbox_value(&doc, pos, 7, "w") * 75.0).round() as i64;
    let cell8_render_w = (table_cell_bbox_value(&doc, pos, 8, "w") * 75.0).round() as i64;
    let cell9_render_w = (table_cell_bbox_value(&doc, pos, 9, "w") * 75.0).round() as i64;

    doc.resize_table_cells(
        pos.section as u32,
        pos.para as u32,
        pos.control as u32,
        &format!(
            r#"[
            {{"cellIdx":5,"widthDelta":{},"localResize":true,"renderWidth":{cell5_render_w}}},
            {{"cellIdx":6,"widthDelta":{},"localResize":true,"renderWidth":{cell6_render_w}}},
            {{"cellIdx":7,"widthDelta":0,"localResize":true,"renderWidth":{cell7_render_w}}},
            {{"cellIdx":8,"widthDelta":0,"localResize":true,"renderWidth":{cell8_render_w}}},
            {{"cellIdx":9,"widthDelta":0,"localResize":true,"renderWidth":{cell9_render_w}}}
        ]"#,
            cell5_render_w - cell5_model_w,
            cell6_render_w - cell6_model_w,
        ),
    )
    .expect("first local row resize");

    let after_local_cell5_w = table_cell_bbox_value(&doc, pos, 5, "w");
    let after_local_cell7_x = table_cell_bbox_value(&doc, pos, 7, "x");
    let after_local_cell2_x = table_cell_bbox_value(&doc, pos, 2, "x");
    let after_local_cell23_x = table_cell_bbox_value(&doc, pos, 23, "x");
    let after_local_cell23_w = table_cell_bbox_value(&doc, pos, 23, "w");
    let after_local_cell24_x = table_cell_bbox_value(&doc, pos, 24, "x");
    let before_global_cell0_w = table_cell_bbox_value(&doc, pos, 0, "w");
    let before_global_cell1_w = table_cell_bbox_value(&doc, pos, 1, "w");
    let global_delta_hu = 1200_i64;
    let target_cells = [0_u32, 10, 15];
    let neighbor_cells = [1_u32, 11, 16];

    let updates = (0_u32..25)
        .map(|idx| {
            let current_render_w =
                (table_cell_bbox_value(&doc, pos, idx as u64, "w") * 75.0).round() as i64;
            let desired_render_w = if target_cells.contains(&idx) {
                current_render_w + global_delta_hu
            } else if neighbor_cells.contains(&idx) {
                current_render_w - global_delta_hu
            } else {
                current_render_w
            };
            let model_w = table_cell_property_i64(&doc, pos, idx, "width");
            format!(
                r#"{{"cellIdx":{idx},"widthDelta":{},"localResize":true,"renderWidth":{desired_render_w}}}"#,
                desired_render_w - model_w
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    doc.resize_table_cells(
        pos.section as u32,
        pos.para as u32,
        pos.control as u32,
        &format!("[{updates}]"),
    )
    .expect("global column resize after local row resize");

    let after_global_cell0_w = table_cell_bbox_value(&doc, pos, 0, "w");
    let after_global_cell1_w = table_cell_bbox_value(&doc, pos, 1, "w");
    let after_global_cell5_w = table_cell_bbox_value(&doc, pos, 5, "w");
    let after_global_cell7_x = table_cell_bbox_value(&doc, pos, 7, "x");
    let after_global_cell2_x = table_cell_bbox_value(&doc, pos, 2, "x");
    let after_global_cell23_x = table_cell_bbox_value(&doc, pos, 23, "x");
    let after_global_cell23_w = table_cell_bbox_value(&doc, pos, 23, "w");
    let after_global_cell24_x = table_cell_bbox_value(&doc, pos, 24, "x");

    assert!(
        after_global_cell0_w > before_global_cell0_w + 10.0,
        "전체 컬럼 대상 셀은 커져야 함: before={before_global_cell0_w}, after={after_global_cell0_w}"
    );
    assert!(
        after_global_cell1_w < before_global_cell1_w - 10.0,
        "전체 컬럼 보상 이웃 셀은 줄어야 함: before={before_global_cell1_w}, after={after_global_cell1_w}"
    );
    assert!(
        (after_global_cell2_x - after_local_cell2_x).abs() <= 0.2,
        "대상/이웃 뒤쪽 셀은 같이 밀리면 안 됨: before={after_local_cell2_x}, after={after_global_cell2_x}"
    );
    assert!(
        (after_global_cell5_w - after_local_cell5_w).abs() <= 0.2
            && (after_global_cell7_x - after_local_cell7_x).abs() <= 0.2,
        "이전에 Shift로 분리한 행은 일반 컬럼 resize 뒤에도 유지되어야 함: 5w {after_local_cell5_w}->{after_global_cell5_w}, 7x {after_local_cell7_x}->{after_global_cell7_x}"
    );
    assert!(
        (after_global_cell23_x - after_local_cell23_x).abs() <= 0.2
            && (after_global_cell23_w - after_local_cell23_w).abs() <= 0.2
            && (after_global_cell24_x - after_local_cell24_x).abs() <= 0.2,
        "업데이트 대상이 아닌 마지막 행 뒤쪽 셀은 전역 fallback 때문에 흔들리면 안 됨: 23x {after_local_cell23_x}->{after_global_cell23_x}, 23w {after_local_cell23_w}->{after_global_cell23_w}, 24x {after_local_cell24_x}->{after_global_cell24_x}"
    );
}

#[test]
fn recovered_shift_resize_row_keeps_independent_widths() {
    let bytes = sample_bytes("samples/셀보호2.hwp");
    let parsed = parse_document(&bytes).expect("parse 셀보호2.hwp");
    let pos = find_first_table(&parsed);
    let mut doc = HwpDocument::from_bytes(&bytes).expect("load HwpDocument");

    let before_cell5_w = table_cell_bbox_value(&doc, pos, 5, "w");
    let before_cell6_w = table_cell_bbox_value(&doc, pos, 6, "w");
    let first_delta_hu = -6000_i64;
    let first_cell5_render_w = table_cell_render_width_hu(&doc, pos, 5) + first_delta_hu;
    let first_cell6_render_w = table_cell_render_width_hu(&doc, pos, 6) - first_delta_hu;
    let first_cell7_render_w = table_cell_render_width_hu(&doc, pos, 7);
    let first_cell8_render_w = table_cell_render_width_hu(&doc, pos, 8);
    let first_cell9_render_w = table_cell_render_width_hu(&doc, pos, 9);

    resize_cells_to_render_widths(
        &mut doc,
        pos,
        &[
            (5, first_cell5_render_w),
            (6, first_cell6_render_w),
            (7, first_cell7_render_w),
            (8, first_cell8_render_w),
            (9, first_cell9_render_w),
        ],
    );

    let after_local_cell5_w = table_cell_bbox_value(&doc, pos, 5, "w");
    let after_local_cell6_w = table_cell_bbox_value(&doc, pos, 6, "w");
    let after_local_cell7_x = table_cell_bbox_value(&doc, pos, 7, "x");
    let after_local_cell23_x = table_cell_bbox_value(&doc, pos, 23, "x");

    assert!(
        after_local_cell5_w < before_cell5_w - 50.0,
        "첫 Shift resize에서 대상 셀은 줄어야 함: before={before_cell5_w}, after={after_local_cell5_w}"
    );
    assert!(
        after_local_cell6_w > before_cell6_w + 50.0,
        "첫 Shift resize에서 이웃 셀은 커져야 함: before={before_cell6_w}, after={after_local_cell6_w}"
    );

    let exported = doc.export_hwp().expect("export recovered source hwp");
    let recovered_parsed = parse_document(&exported).expect("parse recovered source hwp");
    let recovered_pos = find_first_table(&recovered_parsed);
    let mut recovered = HwpDocument::from_bytes(&exported).expect("load recovered HwpDocument");

    let recovered_cell5_w = table_cell_bbox_value(&recovered, recovered_pos, 5, "w");
    let recovered_cell6_w = table_cell_bbox_value(&recovered, recovered_pos, 6, "w");
    let recovered_cell7_x = table_cell_bbox_value(&recovered, recovered_pos, 7, "x");
    let recovered_cell23_x = table_cell_bbox_value(&recovered, recovered_pos, 23, "x");

    assert!(
        (recovered_cell5_w - after_local_cell5_w).abs() <= 0.2
            && (recovered_cell6_w - after_local_cell6_w).abs() <= 0.2
            && (recovered_cell7_x - after_local_cell7_x).abs() <= 0.2,
        "복구본은 저장 전 행 단위 폭을 그대로 렌더해야 함: 5w {after_local_cell5_w}->{recovered_cell5_w}, 6w {after_local_cell6_w}->{recovered_cell6_w}, 7x {after_local_cell7_x}->{recovered_cell7_x}"
    );
    assert!(
        (recovered_cell23_x - after_local_cell23_x).abs() <= 0.2,
        "복구본 로드는 다른 행의 x를 밀면 안 됨: before={after_local_cell23_x}, after={recovered_cell23_x}"
    );

    let second_delta_hu = 6000_i64;
    let second_cell5_render_w =
        table_cell_render_width_hu(&recovered, recovered_pos, 5) + second_delta_hu;
    let second_cell6_render_w =
        table_cell_render_width_hu(&recovered, recovered_pos, 6) - second_delta_hu;
    let second_cell7_render_w = table_cell_render_width_hu(&recovered, recovered_pos, 7);
    let second_cell8_render_w = table_cell_render_width_hu(&recovered, recovered_pos, 8);
    let second_cell9_render_w = table_cell_render_width_hu(&recovered, recovered_pos, 9);

    resize_cells_to_render_widths(
        &mut recovered,
        recovered_pos,
        &[
            (5, second_cell5_render_w),
            (6, second_cell6_render_w),
            (7, second_cell7_render_w),
            (8, second_cell8_render_w),
            (9, second_cell9_render_w),
        ],
    );

    let after_second_cell5_w = table_cell_bbox_value(&recovered, recovered_pos, 5, "w");
    let after_second_cell6_w = table_cell_bbox_value(&recovered, recovered_pos, 6, "w");
    let after_second_cell7_x = table_cell_bbox_value(&recovered, recovered_pos, 7, "x");
    let after_second_cell23_x = table_cell_bbox_value(&recovered, recovered_pos, 23, "x");

    assert!(
        after_second_cell5_w > recovered_cell5_w + 50.0
            && after_second_cell6_w < recovered_cell6_w - 50.0,
        "복구본에서 두 번째 Shift resize도 대상/이웃 셀만 조절되어야 함: 5w {recovered_cell5_w}->{after_second_cell5_w}, 6w {recovered_cell6_w}->{after_second_cell6_w}"
    );
    assert!(
        (after_second_cell7_x - recovered_cell7_x).abs() <= 0.2
            && (after_second_cell23_x - recovered_cell23_x).abs() <= 0.2,
        "복구본에서 두 번째 Shift resize는 뒤쪽/다른 행을 다시 밀면 안 됨: 7x {recovered_cell7_x}->{after_second_cell7_x}, 23x {recovered_cell23_x}->{after_second_cell23_x}"
    );
}

#[test]
fn vertical_resize_keeps_row_cells_aligned() {
    let bytes = sample_bytes("samples/셀보호2.hwp");
    let parsed = parse_document(&bytes).expect("parse 셀보호2.hwp");
    let pos = find_first_table(&parsed);
    let mut doc = HwpDocument::from_bytes(&bytes).expect("load HwpDocument");

    let target_idx = 18_u32;
    let neighbor_idx = 23_u32;
    let stable_same_row_idx = 17_u32;
    let stable_neighbor_row_idx = 22_u32;
    let before_target_h = table_cell_bbox_value(&doc, pos, target_idx as u64, "h");
    let before_neighbor_h = table_cell_bbox_value(&doc, pos, neighbor_idx as u64, "h");
    let before_same_row_h = table_cell_bbox_value(&doc, pos, stable_same_row_idx as u64, "h");
    let before_neighbor_row_h =
        table_cell_bbox_value(&doc, pos, stable_neighbor_row_idx as u64, "h");
    let delta_hu = 1200_i64;

    let updates = {
        let Control::Table(table) =
            &doc.document().sections[pos.section].paragraphs[pos.para].controls[pos.control]
        else {
            panic!("expected table control");
        };
        let target_row = table.cells[target_idx as usize].row;
        let neighbor_row = table.cells[neighbor_idx as usize].row;
        table
            .cells
            .iter()
            .enumerate()
            .filter(|(_, cell)| {
                cell.row_span == 1 && (cell.row == target_row || cell.row == neighbor_row)
            })
            .map(|(idx, _)| {
                let cell = &table.cells[idx];
                let delta = if cell.row == target_row {
                    delta_hu
                } else {
                    -delta_hu
                };
                format!(r#"{{"cellIdx":{idx},"heightDelta":{delta}}}"#)
            })
            .collect::<Vec<_>>()
            .join(",")
    };

    doc.resize_table_cells(
        pos.section as u32,
        pos.para as u32,
        pos.control as u32,
        &format!("[{updates}]"),
    )
    .expect("vertical row resize table cells");

    let after_target_h = table_cell_bbox_value(&doc, pos, target_idx as u64, "h");
    let after_neighbor_h = table_cell_bbox_value(&doc, pos, neighbor_idx as u64, "h");
    let after_same_row_h = table_cell_bbox_value(&doc, pos, stable_same_row_idx as u64, "h");
    let after_neighbor_row_h =
        table_cell_bbox_value(&doc, pos, stable_neighbor_row_idx as u64, "h");

    assert!(
        after_target_h > before_target_h + 10.0
            && after_same_row_h > before_same_row_h + 10.0,
        "세로 resize는 대상 행 전체 높이를 함께 키워야 함: target {before_target_h}->{after_target_h}, same-row {before_same_row_h}->{after_same_row_h}"
    );
    assert!(
        (after_target_h - after_same_row_h).abs() <= 0.2
            && (after_neighbor_h - after_neighbor_row_h).abs() <= 0.2,
        "세로 resize 후 같은 행의 셀 높이는 한컴처럼 정렬되어야 함: target row {after_target_h}/{after_same_row_h}, neighbor row {after_neighbor_h}/{after_neighbor_row_h}"
    );
    assert!(
        after_neighbor_h <= before_neighbor_h + 0.2
            && after_neighbor_row_h <= before_neighbor_row_h + 0.2,
        "내용/여백 때문에 이웃 행이 더 줄지 못하더라도 커지면 안 됨: neighbor {before_neighbor_h}->{after_neighbor_h}, same-row {before_neighbor_row_h}->{after_neighbor_row_h}"
    );
}

#[test]
fn vertical_shift_local_height_keeps_unrelated_cells_stable() {
    let bytes = sample_bytes("samples/셀보호2.hwp");
    let parsed = parse_document(&bytes).expect("parse 셀보호2.hwp");
    let pos = find_first_table(&parsed);
    let mut doc = HwpDocument::from_bytes(&bytes).expect("load HwpDocument");

    let target_idx = 18_u32;
    let neighbor_idx = 23_u32;
    let same_row_idx = 17_u32;
    let same_col_above_idx = 13_u32;
    let neighbor_row_side_idx = 22_u32;
    let before_target_h = table_cell_bbox_value(&doc, pos, target_idx as u64, "h");
    let before_neighbor_h = table_cell_bbox_value(&doc, pos, neighbor_idx as u64, "h");
    let before_same_row_h = table_cell_bbox_value(&doc, pos, same_row_idx as u64, "h");
    let before_same_col_above_y = table_cell_bbox_value(&doc, pos, same_col_above_idx as u64, "y");
    let before_same_col_above_h = table_cell_bbox_value(&doc, pos, same_col_above_idx as u64, "h");
    let before_neighbor_row_side_h =
        table_cell_bbox_value(&doc, pos, neighbor_row_side_idx as u64, "h");
    let delta_hu = 900_i64;
    let height_updates = {
        let Control::Table(table) =
            &doc.document().sections[pos.section].paragraphs[pos.para].controls[pos.control]
        else {
            panic!("expected table control");
        };
        let target_col = table.cells[target_idx as usize].col;
        table
            .cells
            .iter()
            .enumerate()
            .filter(|(_, cell)| cell.col == target_col && cell.col_span == 1 && cell.row_span == 1)
            .map(|(idx, _)| {
                let idx = idx as u32;
                let current = table_cell_render_height_hu(&doc, pos, idx);
                let desired = if idx == target_idx {
                    current + delta_hu
                } else if idx == neighbor_idx {
                    current - delta_hu
                } else {
                    current
                };
                (idx, desired)
            })
            .collect::<Vec<_>>()
    };

    resize_cells_to_render_heights(&mut doc, pos, &height_updates);

    let Control::Table(table) =
        &doc.document().sections[pos.section].paragraphs[pos.para].controls[pos.control]
    else {
        panic!("expected table control");
    };
    let local_height_cells = table
        .local_resize_cell_heights
        .iter()
        .map(|(idx, _)| *idx as u32)
        .collect::<std::collections::BTreeSet<_>>();
    assert!(
        local_height_cells.contains(&target_idx)
            && local_height_cells.contains(&neighbor_idx)
            && local_height_cells.contains(&same_col_above_idx),
        "세로 Shift local height 힌트는 대상/이웃 변경과 같은 열 보존 힌트를 함께 남겨야 함: {local_height_cells:?}"
    );

    let after_target_h = table_cell_bbox_value(&doc, pos, target_idx as u64, "h");
    let after_neighbor_h = table_cell_bbox_value(&doc, pos, neighbor_idx as u64, "h");
    let after_same_row_h = table_cell_bbox_value(&doc, pos, same_row_idx as u64, "h");
    let after_same_col_above_y = table_cell_bbox_value(&doc, pos, same_col_above_idx as u64, "y");
    let after_same_col_above_h = table_cell_bbox_value(&doc, pos, same_col_above_idx as u64, "h");
    let after_neighbor_row_side_h =
        table_cell_bbox_value(&doc, pos, neighbor_row_side_idx as u64, "h");

    assert!(
        after_target_h > before_target_h + 5.0 && after_neighbor_h < before_neighbor_h - 5.0,
        "세로 Shift resize는 대상/이웃 셀 높이만 보상 조절해야 함: target {before_target_h}->{after_target_h}, neighbor {before_neighbor_h}->{after_neighbor_h}"
    );
    assert!(
        (after_same_row_h - before_same_row_h).abs() <= 0.2
            && (after_neighbor_row_side_h - before_neighbor_row_side_h).abs() <= 0.2,
        "세로 Shift resize는 옆 셀 행 높이를 흔들면 안 됨: same-row {before_same_row_h}->{after_same_row_h}, neighbor-row-side {before_neighbor_row_side_h}->{after_neighbor_row_side_h}"
    );
    assert!(
        (after_same_col_above_y - before_same_col_above_y).abs() <= 0.2
            && (after_same_col_above_h - before_same_col_above_h).abs() <= 0.2,
        "세로 Shift resize는 같은 열의 무관한 셀을 흔들면 안 됨: y {before_same_col_above_y}->{after_same_col_above_y}, h {before_same_col_above_h}->{after_same_col_above_h}"
    );
}

#[test]
fn explicit_cell_inner_margin_sample_matches_hancom_saved_result() {
    for rel in ["samples/셀보호2.hwp", "samples/셀보호2.hwpx"] {
        let bytes = sample_bytes(rel);
        let doc = parse_document(&bytes).unwrap_or_else(|e| panic!("parse {rel}: {e:?}"));
        let pos = find_first_table(&doc);
        assert_inner_margin_sample_attrs(&doc, pos);

        let api = HwpDocument::from_bytes(&bytes).expect("load HwpDocument");
        let json = api
            .get_cell_properties(pos.section as u32, pos.para as u32, pos.control as u32, 20)
            .expect("get explicit margin cell properties");
        let props: Value = serde_json::from_str(&json).expect("parse cell properties");
        assert_eq!(props["applyInnerMargin"].as_bool(), Some(true), "{json}");
        assert_eq!(props["paddingLeft"].as_i64(), Some(2834), "{json}");
        assert_eq!(props["paddingRight"].as_i64(), Some(2834), "{json}");
        assert_eq!(props["paddingTop"].as_i64(), Some(0), "{json}");
        assert_eq!(props["paddingBottom"].as_i64(), Some(0), "{json}");
    }
}

#[test]
fn cell_protect_and_form_editable_survive_hwpx_roundtrip() {
    let bytes = sample_bytes("samples/셀보호.hwpx");
    let doc = parse_hwpx(&bytes).expect("parse 셀보호.hwpx");
    let pos = find_first_table(&doc);
    assert_cell_attrs(&doc, pos);

    let serialized = serialize_hwpx(&doc).expect("serialize hwpx");
    let xml = hwpx_section0_xml(&serialized);
    assert_eq!(
        xml.matches(r#"protect="1""#).count(),
        3,
        "serialized section0.xml should keep three protected cells"
    );
    assert_eq!(
        xml.matches(r#"editable="1""#).count(),
        1,
        "serialized section0.xml should keep one form-editable cell"
    );
    assert_eq!(
        xml.matches(r#"hasMargin="0""#).count(),
        25,
        "serialized section0.xml should keep all cell inner-margin flags off"
    );
    let named_cell = named_cell_opening_tag(&xml);
    assert!(
        named_cell.contains(r#"protect="1""#) && named_cell.contains(r#"editable="1""#),
        "serialized named cell should keep protect/editable attrs: {named_cell}"
    );

    let reparsed = parse_hwpx(&serialized).expect("reparse serialized hwpx");
    let pos2 = find_first_table(&reparsed);
    assert_cell_attrs(&reparsed, pos2);
}

#[test]
fn explicit_cell_inner_margin_survives_hwpx_roundtrip() {
    let bytes = sample_bytes("samples/셀보호2.hwpx");
    let doc = parse_hwpx(&bytes).expect("parse 셀보호2.hwpx");
    let pos = find_first_table(&doc);
    assert_inner_margin_sample_attrs(&doc, pos);

    let serialized = serialize_hwpx(&doc).expect("serialize hwpx");
    let xml = hwpx_section0_xml(&serialized);
    assert_eq!(
        xml.matches(r#"hasMargin="1""#).count(),
        1,
        "serialized section0.xml should keep one explicit inner-margin cell"
    );
    assert_eq!(
        xml.matches(r#"hasMargin="0""#).count(),
        24,
        "serialized section0.xml should keep the other cells with inner-margin flags off"
    );
    assert!(
        xml.contains(r#"<hp:cellMargin left="2834" right="2834" top="0" bottom="0"/>"#),
        "serialized section0.xml should keep Hancom's 10mm/10mm/0/0 cell margin"
    );
}

#[test]
fn set_cell_border_properties_do_not_overwrite_cell_size() {
    let bytes = sample_bytes("samples/셀보호.hwp");
    let parsed = parse_document(&bytes).expect("parse 셀보호.hwp");
    let pos = find_first_table(&parsed);
    let mut doc = HwpDocument::from_bytes(&bytes).expect("load HwpDocument");

    let before_json = doc
        .get_cell_properties(pos.section as u32, pos.para as u32, pos.control as u32, 0)
        .expect("get before cell properties");
    let before: Value = serde_json::from_str(&before_json).expect("parse before properties");
    let before_width = before["width"].as_u64().expect("before width");
    let before_height = before["height"].as_u64().expect("before height");

    doc.set_cell_properties(
        pos.section as u32,
        pos.para as u32,
        pos.control as u32,
        0,
        r##"{
          "borderLeft":{"type":1,"width":3,"color":"#ff0000"},
          "borderRight":{"type":2,"width":4,"color":"#00ff00"},
          "borderTop":{"type":3,"width":5,"color":"#0000ff"},
          "borderBottom":{"type":4,"width":6,"color":"#112233"},
          "fillType":"solid",
          "fillColor":"#ddeeff",
          "patternColor":"#445566",
          "patternType":1
        }"##,
    )
    .expect("set cell border/fill properties");

    let after_json = doc
        .get_cell_properties(pos.section as u32, pos.para as u32, pos.control as u32, 0)
        .expect("get after cell properties");
    let after: Value = serde_json::from_str(&after_json).expect("parse after properties");

    assert_eq!(after["width"].as_u64(), Some(before_width), "{after_json}");
    assert_eq!(
        after["height"].as_u64(),
        Some(before_height),
        "{after_json}"
    );
    assert_eq!(
        after["borderLeft"]["width"].as_u64(),
        Some(3),
        "{after_json}"
    );
    assert_eq!(
        after["borderRight"]["width"].as_u64(),
        Some(4),
        "{after_json}"
    );
    assert_eq!(after["fillType"].as_str(), Some("solid"), "{after_json}");
    assert_eq!(after["fillColor"].as_str(), Some("#ddeeff"), "{after_json}");
}

#[test]
fn set_cell_properties_updates_apply_inner_margin_flag() {
    let bytes = sample_bytes("samples/셀보호.hwp");
    let parsed = parse_document(&bytes).expect("parse 셀보호.hwp");
    let pos = find_first_table(&parsed);
    let mut doc = HwpDocument::from_bytes(&bytes).expect("load HwpDocument");

    doc.set_cell_properties(
        pos.section as u32,
        pos.para as u32,
        pos.control as u32,
        0,
        r#"{"applyInnerMargin":true,"paddingLeft":1134,"paddingRight":0,"paddingTop":0,"paddingBottom":0}"#,
    )
    .expect("set applyInnerMargin true");
    let on_json = doc
        .get_cell_properties(pos.section as u32, pos.para as u32, pos.control as u32, 0)
        .expect("get cell properties on");
    let on: Value = serde_json::from_str(&on_json).expect("parse on properties");
    assert_eq!(on["applyInnerMargin"].as_bool(), Some(true), "{on_json}");
    assert_eq!(on["paddingLeft"].as_i64(), Some(1134), "{on_json}");

    doc.set_cell_properties(
        pos.section as u32,
        pos.para as u32,
        pos.control as u32,
        0,
        r#"{"applyInnerMargin":false}"#,
    )
    .expect("set applyInnerMargin false");
    let off_json = doc
        .get_cell_properties(pos.section as u32, pos.para as u32, pos.control as u32, 0)
        .expect("get cell properties off");
    let off: Value = serde_json::from_str(&off_json).expect("parse off properties");
    assert_eq!(off["applyInnerMargin"].as_bool(), Some(false), "{off_json}");
    assert_eq!(
        off["paddingLeft"].as_i64(),
        Some(1134),
        "체크 해제는 padding 원값을 지우지 않고 적용 플래그만 끈다: {off_json}"
    );
}

#[test]
fn set_cell_properties_reflows_text_after_inner_margin_change() {
    let bytes = sample_bytes("samples/셀보호.hwp");
    let parsed = parse_document(&bytes).expect("parse 셀보호.hwp");
    let pos = find_first_table(&parsed);
    let mut doc = HwpDocument::from_bytes(&bytes).expect("load HwpDocument");

    doc.set_cell_properties(
        pos.section as u32,
        pos.para as u32,
        pos.control as u32,
        2,
        r#"{"applyInnerMargin":true,"paddingLeft":2835,"paddingRight":2835,"paddingTop":0,"paddingBottom":0}"#,
    )
    .expect("set wide inner margin");

    let Control::Table(table) =
        &doc.document().sections[pos.section].paragraphs[pos.para].controls[pos.control]
    else {
        panic!("expected table control");
    };
    let named_cell_para = &table.cells[2].paragraphs[0];
    assert!(
        named_cell_para.line_segs.len() > 1,
        "좌우 안 여백 지정 후에는 한컴처럼 새 내부 폭 기준으로 셀 문단을 다시 줄바꿈해야 함: {:?}",
        named_cell_para.line_segs
    );
    assert_eq!(
        table.cells[2].padding.top, 0,
        "안 여백 지정 상태의 0mm 값은 표 기본 여백으로 되살리면 안 됨"
    );
    assert!(
        table.cells[2].apply_inner_margin,
        "안 여백 지정 플래그가 켜져 있어야 함"
    );
}

#[test]
fn set_cell_properties_reflows_text_after_inner_margin_turns_off() {
    let bytes = sample_bytes("samples/셀보호2.hwp");
    let parsed = parse_document(&bytes).expect("parse 셀보호2.hwp");
    let pos = find_first_table(&parsed);
    let mut doc = HwpDocument::from_bytes(&bytes).expect("load HwpDocument");

    doc.set_cell_properties(
        pos.section as u32,
        pos.para as u32,
        pos.control as u32,
        20,
        r#"{"applyInnerMargin":false}"#,
    )
    .expect("turn applyInnerMargin off");

    let Control::Table(table) =
        &doc.document().sections[pos.section].paragraphs[pos.para].controls[pos.control]
    else {
        panic!("expected table control");
    };
    let cell = &table.cells[20];
    assert!(
        !cell.apply_inner_margin,
        "안 여백 지정 플래그가 꺼져 있어야 함"
    );
    assert_eq!(
        cell.padding.left, 2834,
        "체크 해제는 좌측 padding 원값을 보존"
    );
    assert_eq!(
        cell.padding.right, 2834,
        "체크 해제는 우측 padding 원값을 보존"
    );
    assert_eq!(
        cell.paragraphs[0].line_segs.len(),
        1,
        "안 여백 지정 off 후에는 보존된 10mm 값이 아니라 표 기본 여백 기준으로 한 줄 reflow"
    );
}
