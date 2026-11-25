//! #9399.
//! 글상자 안에서 글자 밖(글 끝 뒤·줄 아래·안쪽 여백)을 누르면 그 글상자 글의 가장 가까운
//! 줄과 글자에 커서를 둔다. 표 칸과 글상자가 함께 클릭을 품으면 더 깊은 쪽이 받는다.
//!
//! 고치기 전에는
//! - 본문 글상자에서 글 끝 뒤를 누르면 글상자 첫 문단 0번째 글자로 갔고,
//! - 표 칸 안 묶음의 둘째 글상자에서 글 끝 뒤를 누르면 칸 분기가 `innermost().cell_index`
//!   (글상자 번호 1)로 바깥 칸(0)의 run을 고르지 못해 첫 글상자 글 끝으로 갔다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;
use serde_json::{json, Value};

fn open(path: &str) -> HwpDocument {
    let bytes = std::fs::read(format!("{}/{path}", env!("CARGO_MANIFEST_DIR"))).unwrap();
    HwpDocument::from_bytes(&bytes).unwrap()
}

fn caret(doc: &HwpDocument, parent_para: u32, path: &Value, offset: u32) -> Value {
    serde_json::from_str(
        &doc.get_cursor_rect_by_path(0, parent_para, &path.to_string(), offset)
            .unwrap_or_else(|_| panic!("커서: {path} {offset}")),
    )
    .unwrap()
}

/// 커서 상자에서 (dx, 높이의 dy배) 떨어진 곳을 누른 결과.
fn click_from(doc: &HwpDocument, at: &Value, dx: f64, dy: f64) -> Value {
    let page = at["pageIndex"].as_u64().unwrap() as u32;
    let x = at["x"].as_f64().unwrap() + dx;
    let y = at["y"].as_f64().unwrap() + at["height"].as_f64().unwrap() * dy;
    serde_json::from_str(&doc.hit_test_native(page, x, y).unwrap()).unwrap()
}

/// 본문 글상자: samples/issue6974/synth_textbox_vgrow.hwp 본문 5문단의 글상자(컨트롤 0).
/// 문단 1은 " * 새 노래"(7자), 문단 5는 " - 빨대 로켓 만들기"(12자)다.
#[test]
fn click_after_line_end_in_body_text_box_lands_on_that_line() {
    let doc = open("samples/issue6974/synth_textbox_vgrow.hwp");
    let para = |p: usize| json!([{"controlIndex": 0, "cellIndex": 0, "cellParaIndex": p}]);

    for (p, len) in [(1usize, 7u32), (5, 12)] {
        let end = caret(&doc, 5, &para(p), len);
        // 글 끝 오른쪽 6px, 줄 가운데.
        let hit = click_from(&doc, &end, 6.0, 0.5);
        assert_eq!(hit["cellPath"], para(p), "문단 {p} 글 끝 뒤: {hit}");
        assert_eq!(hit["charOffset"], len, "문단 {p} 글 끝 뒤: {hit}");
        assert_eq!(hit["isTextBox"], true, "{hit}");
    }

    // 문단 1 줄 바로 아래(줄 사이)를 누르면 더 가까운 문단 1의 그 x 자리로 간다.
    let end = caret(&doc, 5, &para(1), 7);
    let hit = click_from(&doc, &end, 6.0, 1.1);
    assert_eq!(hit["cellPath"], para(1), "줄 사이: {hit}");
    assert_eq!(hit["charOffset"], 7, "줄 사이: {hit}");

    // 글자 위를 누르는 기존 경로는 그대로다.
    let start = caret(&doc, 5, &para(5), 0);
    let mid = caret(&doc, 5, &para(5), 6);
    let hit = click_from(&doc, &mid, 1.0, 0.5);
    assert_eq!(hit["cellPath"], para(5), "글자 위: {hit}");
    assert_eq!(hit["charOffset"], 6, "글자 위: {hit}");
    assert!(start["x"].as_f64() < mid["x"].as_f64());
}

/// 표 칸 안 묶음 글상자: tests/fixtures/group_textbox_cell/cell-group-textbox.hwp.
/// 본문 1문단의 표(컨트롤 0) 셀 0 · 셀 문단 1에 글상자 둘을 묶은 개체(컨트롤 0)가 있다.
#[test]
fn click_after_text_in_grouped_text_box_inside_cell_stays_in_that_text_box() {
    let doc = open("tests/fixtures/group_textbox_cell/cell-group-textbox.hwp");
    let text_box = |child: usize| {
        json!([
            {"controlIndex": 0, "cellIndex": 0, "cellParaIndex": 1},
            {"controlIndex": 0, "cellIndex": child, "cellParaIndex": 0}
        ])
    };

    for (child, len) in [(0usize, 4u32), (1, 5)] {
        let end = caret(&doc, 1, &text_box(child), len);
        for (dx, dy) in [(4.0, 0.5), (20.0, 0.5), (0.0, 1.2)] {
            let hit = click_from(&doc, &end, dx, dy);
            assert_eq!(
                hit["cellPath"],
                text_box(child),
                "글상자 {child} 글 끝에서 ({dx}, {dy}): {hit}"
            );
            assert_eq!(hit["charOffset"], len, "글상자 {child} ({dx}, {dy}): {hit}");
        }
    }
}

/// 반례: 글상자 안 표 칸(`[글상자, 표 셀]`)은 칸이 받는다.
/// samples/issue8513/textbox_floating_table_offset.hwpx 본문 0문단 글상자(컨트롤 2) 첫 문단의
/// 1칸 표(컨트롤 0), 칸 글 "BOX".
#[test]
fn click_after_text_in_table_cell_inside_text_box_stays_in_that_cell() {
    let doc = open("samples/issue8513/textbox_floating_table_offset.hwpx");
    let cell = json!([
        {"controlIndex": 2, "cellIndex": 0, "cellParaIndex": 0},
        {"controlIndex": 0, "cellIndex": 0, "cellParaIndex": 0}
    ]);
    let end = caret(&doc, 0, &cell, 3);
    let hit = click_from(&doc, &end, 4.0, 0.5);
    assert_eq!(hit["cellPath"], cell, "글상자 안 칸 글 끝 뒤: {hit}");
    assert_eq!(hit["charOffset"], 3, "{hit}");
}
