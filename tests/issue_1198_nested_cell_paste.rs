//! Issue #1198: 중첩 표 셀 내부 클립보드 붙여넣기 대상 경로 보존.
//!
//! 재현 문서: `samples/exam_social.hwp`
//! 대상: 1쪽 상단 답안지 영역의 `성명` 오른쪽 빈 입력칸.

use std::path::Path;

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn load_sample(name: &str) -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples")
        .join(name);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {name}: {e}"))
}

fn hit_json(doc: &HwpDocument, page: u32, x: f64, y: f64) -> Value {
    let json = doc
        .hit_test_native(page, x, y)
        .unwrap_or_else(|e| panic!("hit_test_native({page}, {x}, {y}): {e}"));
    serde_json::from_str(&json).unwrap_or_else(|e| panic!("parse hit json `{json}`: {e}"))
}

fn path_tuples(hit: &Value) -> Vec<(usize, usize, usize)> {
    hit["cellPath"]
        .as_array()
        .expect("cellPath array")
        .iter()
        .map(|entry| {
            (
                entry["controlIndex"].as_u64().expect("controlIndex") as usize,
                entry["cellIndex"].as_u64().expect("cellIndex") as usize,
                entry["cellParaIndex"].as_u64().expect("cellParaIndex") as usize,
            )
        })
        .collect()
}

fn first_copyable_char(doc: &HwpDocument) -> (u32, u32, String) {
    let para_count = doc
        .get_paragraph_count(0)
        .unwrap_or_else(|e| panic!("get_paragraph_count: {e:?}"));
    for para_idx in 0..para_count {
        let len = doc
            .get_paragraph_length(0, para_idx)
            .unwrap_or_else(|e| panic!("get_paragraph_length({para_idx}): {e:?}"));
        for offset in 0..len {
            let text = doc
                .get_text_range(0, para_idx, offset, 1)
                .unwrap_or_default();
            if text
                .chars()
                .any(|ch| !ch.is_whitespace() && !ch.is_control())
            {
                return (para_idx, offset, text);
            }
        }
    }
    panic!("copyable source character not found");
}

#[test]
fn issue_1198_exam_social_internal_paste_uses_nested_cell_path() {
    let mut doc = load_sample("exam_social.hwp");

    // 1쪽 상단 답안지 `성명` 오른쪽 빈 입력칸 내부 좌표.
    // [#3386 A/C/D] 중첩 TAC anchor 의 outer-margin top 정합(+3.8px)으로
    // 중첩 표가 한글 좌표로 내려가, 종전 y=210.0 이 경계 밖이 됨 — 셀 안쪽 재고정.
    let hit = hit_json(&doc, 0, 250.0, 216.0);
    let path = path_tuples(&hit);
    assert_eq!(
        path,
        vec![(4, 0, 3), (0, 1, 0)],
        "exam_social.hwp name field must remain a nested cell path: {hit}"
    );

    let path_json = serde_json::to_string(&hit["cellPath"]).expect("cellPath json");
    let char_offset = hit["charOffset"].as_u64().unwrap_or(0) as usize;
    let (source_para, source_offset, expected) = first_copyable_char(&doc);
    let expected_chars = expected.chars().count();

    doc.copy_selection(
        0,
        source_para,
        source_offset,
        source_para,
        source_offset + expected_chars as u32,
    )
    .unwrap_or_else(|e| panic!("copy_selection failed: {e:?}"));
    assert_eq!(doc.get_clipboard_text(), expected);

    let result_json = doc
        .paste_internal_in_cell_by_path(0, 0, &path_json, char_offset as u32)
        .unwrap_or_else(|e| panic!("paste_internal_in_cell_by_path failed: {e:?}"));
    let result: Value = serde_json::from_str(&result_json)
        .unwrap_or_else(|e| panic!("parse paste result `{result_json}`: {e}"));
    assert_eq!(result["ok"].as_bool(), Some(true), "{result_json}");
    assert_eq!(
        result["cellParaIdx"].as_u64(),
        Some(path[path.len() - 1].2 as u64),
        "{result_json}"
    );
    assert_eq!(
        result["charOffset"].as_u64(),
        Some((char_offset + expected_chars) as u64),
        "{result_json}"
    );

    let inserted = doc
        .get_text_in_cell_by_path(0, 0, &path, char_offset, expected_chars)
        .unwrap_or_else(|e| panic!("get_text_in_cell_by_path failed: {e}"));
    assert_eq!(inserted, expected);
}

#[test]
fn issue_1198_exam_social_html_paste_uses_nested_cell_path() {
    let mut doc = load_sample("exam_social.hwp");

    let hit = hit_json(&doc, 0, 250.0, 216.0);
    let path = path_tuples(&hit);
    assert_eq!(
        path,
        vec![(4, 0, 3), (0, 1, 0)],
        "exam_social.hwp name field must remain a nested cell path: {hit}"
    );

    let path_json = serde_json::to_string(&hit["cellPath"]).expect("cellPath json");
    let char_offset = hit["charOffset"].as_u64().unwrap_or(0) as usize;
    let expected = "붙여넣기";
    let html = format!(
        "<html><body><!--StartFragment--><p>{}</p><!--EndFragment--></body></html>",
        expected
    );

    let result_json = doc
        .paste_html_in_cell_by_path(0, 0, &path_json, char_offset as u32, &html)
        .unwrap_or_else(|e| panic!("paste_html_in_cell_by_path failed: {e:?}"));
    let result: Value = serde_json::from_str(&result_json)
        .unwrap_or_else(|e| panic!("parse paste result `{result_json}`: {e}"));
    assert_eq!(result["ok"].as_bool(), Some(true), "{result_json}");
    assert_eq!(
        result["cellParaIdx"].as_u64(),
        Some(path[path.len() - 1].2 as u64),
        "{result_json}"
    );
    assert_eq!(
        result["charOffset"].as_u64(),
        Some((char_offset + expected.chars().count()) as u64),
        "{result_json}"
    );

    let inserted = doc
        .get_text_in_cell_by_path(0, 0, &path, char_offset, expected.chars().count())
        .unwrap_or_else(|e| panic!("get_text_in_cell_by_path failed: {e}"));
    assert_eq!(inserted, expected);
}
