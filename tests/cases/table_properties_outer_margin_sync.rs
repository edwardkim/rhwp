#![cfg(not(target_arch = "wasm32"))]
//! `setTableProperties` 로 바꾼 표 바깥 여백은 조판과 HWPX 저장이 읽는 `outer_margin_*` 에도
//! 들어가야 한다.
//!
//! 종전 명령은 `common.margin` 과 HWP 원본 바이트에만 써서, 편집한 표는 HWP로 저장해 다시
//! 열기 전에는 움직이지 않고 HWPX 저장에서는 여백을 잃었다. 원본 바이트가 없는 HWPX 표는
//! 편집이 통째로 버려졌다. 조회는 `common.margin` 을 읽어 새로 만든 표의 여백을 0으로
//! 돌려줬고, 그 값을 그대로 다시 쓰면 HWP 저장본의 여백이 0이 됐다.
use rhwp::model::control::Control;
use rhwp::model::table::Table;
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const HWPX_TABLE: &str = "samples/issue6025/3232693_employment_support_criteria.hwpx";

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

/// 새 문서 첫 줄에 2×2 표를 만들고 그 (문단, 컨트롤) 번호를 돌려준다.
fn first_line_table() -> (HwpDocument, u32, u32) {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document().unwrap();
    let made: Value = serde_json::from_str(&doc.create_table(0, 0, 0, 2, 2).unwrap()).unwrap();
    let index = |key: &str| made[key].as_u64().unwrap() as u32;
    (doc, index("paraIdx"), index("controlIdx"))
}

fn open(bytes: &[u8]) -> HwpDocument {
    HwpDocument::from_bytes(bytes).unwrap()
}

/// HWP·HWPX로 저장해 다시 연 문서.
fn reopened(doc: &HwpDocument) -> [(&'static str, HwpDocument); 2] {
    [
        ("HWP", open(&doc.export_hwp().unwrap())),
        ("HWPX", open(&doc.export_hwpx().unwrap())),
    ]
}

fn props(doc: &HwpDocument, para: u32, ctrl: u32) -> Value {
    serde_json::from_str(&doc.get_table_properties(0, para, ctrl).unwrap()).unwrap()
}

fn outer(doc: &HwpDocument, para: u32, ctrl: u32) -> [i64; 4] {
    let p = props(doc, para, ctrl);
    ["outerLeft", "outerRight", "outerTop", "outerBottom"].map(|key| p[key].as_i64().unwrap())
}

fn table<'a>(doc: &'a HwpDocument, para: u32, ctrl: u32) -> &'a Table {
    match &doc.document().sections[0].paragraphs[para as usize].controls[ctrl as usize] {
        Control::Table(table) => table,
        _ => panic!("표가 아니다"),
    }
}

fn bbox(doc: &HwpDocument, para: u32, ctrl: u32) -> (f64, f64) {
    let bbox: Value = serde_json::from_str(&doc.get_table_bbox(0, para, ctrl).unwrap()).unwrap();
    (bbox["x"].as_f64().unwrap(), bbox["y"].as_f64().unwrap())
}

#[test]
fn outer_margin_edit_moves_table_now_and_after_reopen() {
    let (mut doc, para, ctrl) = first_line_table();
    let (_, before) = bbox(&doc, para, ctrl);
    doc.set_table_properties(0, para, ctrl, r#"{"outerTop":5000}"#)
        .unwrap();
    let (_, after) = bbox(&doc, para, ctrl);
    let moved = hu_to_px(5000 - 283);
    assert!(
        (after - before - moved).abs() < 0.5,
        "단 맨 위 표는 바깥 위 여백만큼 내려앉는다: 283→5000HU 뒤 위끝 {before:.1}→{after:.1}px, \
         {moved:.1}px 내려가야 한다"
    );
    assert_eq!(outer(&doc, para, ctrl), [283, 283, 5000, 283]);
    for (format, doc) in reopened(&doc) {
        assert_eq!(outer(&doc, para, ctrl), [283, 283, 5000, 283], "{format}");
        let (_, top) = bbox(&doc, para, ctrl);
        assert!(
            (top - after).abs() < 0.5,
            "{format}로 다시 연 표 위끝 {top:.1}px가 편집 직후 {after:.1}px와 다르다"
        );
    }
}

#[test]
fn hwpx_table_outer_margin_edit_is_not_dropped() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(HWPX_TABLE);
    let mut doc = open(&std::fs::read(path).unwrap());
    let (para, ctrl) = (1, 0);
    assert!(table(&doc, para, ctrl).raw_ctrl_data.is_empty());
    let (x_before, _) = bbox(&doc, para, ctrl);
    doc.set_table_properties(0, para, ctrl, r#"{"outerLeft":1000,"outerTop":1000}"#)
        .unwrap();
    let t = table(&doc, para, ctrl);
    assert_eq!(
        [
            t.outer_margin_left,
            t.outer_margin_top,
            t.common.margin.left,
            t.common.margin.top
        ],
        [1000; 4]
    );
    assert_eq!(outer(&doc, para, ctrl), [1000, 141, 1000, 141]);
    let (x_after, _) = bbox(&doc, para, ctrl);
    let moved = hu_to_px(1000 - 141);
    assert!(
        (x_after - x_before - moved).abs() < 0.5,
        "바깥 왼쪽 여백 141→1000HU 뒤 표 왼끝 {x_before:.1}→{x_after:.1}px, {moved:.1}px 옮겨야 한다"
    );
    for (format, doc) in reopened(&doc) {
        assert_eq!(outer(&doc, para, ctrl), [1000, 141, 1000, 141], "{format}");
    }
}

#[test]
fn created_table_margins_survive_a_properties_round_trip() {
    let (mut doc, para, ctrl) = first_line_table();
    let read = outer(&doc, para, ctrl);
    assert_eq!(read, [283; 4], "새 표의 바깥 여백은 1mm(283HU)다");
    let (_, top) = bbox(&doc, para, ctrl);
    doc.set_table_properties(
        0,
        para,
        ctrl,
        &format!(
            r#"{{"outerLeft":{},"outerRight":{},"outerTop":{},"outerBottom":{}}}"#,
            read[0], read[1], read[2], read[3]
        ),
    )
    .unwrap();
    assert_eq!(bbox(&doc, para, ctrl).1, top);
    for (format, doc) in reopened(&doc) {
        assert_eq!(outer(&doc, para, ctrl), [283; 4], "{format}");
    }
}

#[test]
fn outer_margin_edit_is_undone_by_snapshot() {
    let (mut doc, para, ctrl) = first_line_table();
    let (_, before) = bbox(&doc, para, ctrl);
    let snapshot = doc.save_snapshot();
    doc.set_table_properties(0, para, ctrl, r#"{"outerTop":5000,"outerBottom":5000}"#)
        .unwrap();
    assert!(
        bbox(&doc, para, ctrl).1 > before + 1.0,
        "편집이 표를 옮겨야 한다"
    );
    doc.restore_snapshot(snapshot).unwrap();
    assert_eq!(bbox(&doc, para, ctrl).1, before);
    assert_eq!(outer(&doc, para, ctrl), [283; 4]);
    let t = table(&doc, para, ctrl);
    assert_eq!([t.outer_margin_top, t.outer_margin_bottom], [283; 2]);
}
