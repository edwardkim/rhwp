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
