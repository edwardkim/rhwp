//! 표 셀 안 묶음의 글상자 글은 표 셀 단계부터 시작하는 경로로 커서·선택·누르기를 낸다.
//! 시험 문서: 1×1 표의 둘째 셀 문단에 글상자 둘을 묶은 개체를 두고, 글상자마다 "첫 상자"·"둘째 상자"를 쓴 HWP.
//! 한/글 12.30 Mac에서 열면 두 글상자 안에 캐럿을 두고 글을 고칠 수 있다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;

fn fixture() -> HwpDocument {
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/group_textbox_cell/cell-group-textbox.hwp"
    ))
    .unwrap();
    HwpDocument::from_bytes(&bytes).unwrap()
}

#[test]
fn grouped_text_boxes_in_a_cell_have_full_path_carets() {
    let doc = fixture();
    // 본문 1문단의 표(컨트롤 0) 셀 0, 셀 문단 1의 묶음(컨트롤 0)의 자식 글상자 0·1.
    for (child, text) in [(0, "첫 상자"), (1, "둘째 상자")] {
        let path = format!(
            r#"[{{"controlIndex":0,"cellIndex":0,"cellParaIndex":1}},{{"controlIndex":0,"cellIndex":{child},"cellParaIndex":0}}]"#
        );
        let length = text.chars().count() as u32;
        assert_eq!(
            doc.get_text_in_cell_by_path(0, 1, &[(0, 0, 1), (0, child, 0)], 0, 100)
                .unwrap(),
            text
        );
        let caret: serde_json::Value = serde_json::from_str(
            &doc.get_cursor_rect_by_path(0, 1, &path, 1)
                .unwrap_or_else(|_| panic!("{text}: 커서")),
        )
        .unwrap();
        let hit: serde_json::Value = serde_json::from_str(
            &doc.hit_test_native(
                caret["pageIndex"].as_u64().unwrap() as u32,
                caret["x"].as_f64().unwrap() + 1.0,
                caret["y"].as_f64().unwrap() + caret["height"].as_f64().unwrap() / 2.0,
            )
            .unwrap(),
        )
        .unwrap();
        let expected: serde_json::Value = serde_json::from_str(&path).unwrap();
        assert_eq!(hit["cellPath"], expected, "{text}: 누르기 {hit}");
        let rects: serde_json::Value = serde_json::from_str(
            &doc.get_selection_rects_in_cell_by_path(0, 1, &path, 0, 0, 0, length)
                .unwrap(),
        )
        .unwrap();
        assert!(!rects.as_array().unwrap().is_empty(), "{text}: 선택");
    }
}
