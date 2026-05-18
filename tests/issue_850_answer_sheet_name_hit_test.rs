//! Issue #850: rhwp-studio 상단 답안지 `성명` 칸 입력 회귀.
//!
//! 재현 문서: `samples/exam_social.hwp`, `samples/exam_science.hwp`
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

fn assert_answer_sheet_name_hit(
    hit: &Value,
    outer_control_index: u64,
    expected_path: &[(usize, usize, usize)],
) {
    assert_eq!(hit["sectionIndex"].as_u64(), Some(0), "hit={hit}");
    assert_eq!(
        hit["parentParaIndex"].as_u64(),
        Some(0),
        "answer sheet name field must keep the outer table parent paragraph, hit={hit}"
    );
    assert_eq!(
        hit["controlIndex"].as_u64(),
        Some(outer_control_index),
        "answer sheet name field must expose the outer table control index, hit={hit}"
    );
    assert_eq!(
        hit["cellIndex"].as_u64(),
        Some(expected_path[0].1 as u64),
        "public cellIndex must remain the outer table cell, hit={hit}"
    );
    assert_eq!(
        hit["cellParaIndex"].as_u64(),
        Some(expected_path[0].2 as u64),
        "public cellParaIndex must remain the outer table cell paragraph, hit={hit}"
    );
    assert_eq!(
        path_tuples(hit),
        expected_path,
        "answer sheet name field must preserve the full nested table path, hit={hit}"
    );
}

fn assert_name_insert_by_path(
    doc: &mut HwpDocument,
    sample_name: &str,
    outer_control_index: u64,
    expected_path: &[(usize, usize, usize)],
) {
    // 1쪽 상단 답안지 `성명` 오른쪽 빈 입력칸 내부 좌표.
    let hit = hit_json(doc, 0, 250.0, 210.0);
    assert_answer_sheet_name_hit(&hit, outer_control_index, expected_path);

    let path = path_tuples(&hit);
    let path_json = serde_json::to_string(&hit["cellPath"]).expect("cellPath json");
    let char_offset = hit["charOffset"].as_u64().unwrap_or(0) as usize;
    doc.insert_text_in_cell_by_path(0, 0, &path, char_offset, "홍")
        .unwrap_or_else(|e| panic!("{sample_name}: insert_text_in_cell_by_path failed: {e}"));

    let inserted = doc
        .get_text_in_cell_by_path(0, 0, &path, char_offset, 1)
        .unwrap_or_else(|e| panic!("{sample_name}: get_text_in_cell_by_path failed: {e}"));
    assert_eq!(
        inserted, "홍",
        "{sample_name}: inserted text must be readable by path"
    );

    let rect_json = doc
        .get_cursor_rect_by_path(0, 0, &path_json, (char_offset + 1) as u32)
        .unwrap_or_else(|e| panic!("{sample_name}: get_cursor_rect_by_path failed: {e:?}"));
    let rect: Value = serde_json::from_str(&rect_json)
        .unwrap_or_else(|e| panic!("{sample_name}: parse cursor rect `{rect_json}`: {e}"));
    assert_eq!(
        rect["pageIndex"].as_u64(),
        Some(0),
        "{sample_name}: rect={rect}"
    );
    assert!(
        rect["height"].as_f64().unwrap_or(0.0) > 0.0,
        "{sample_name}: cursor rect height must be positive, rect={rect}"
    );
}

#[test]
fn issue_850_exam_social_answer_sheet_name_cell_keeps_outer_path() {
    let mut doc = load_sample("exam_social.hwp");
    assert_name_insert_by_path(&mut doc, "exam_social.hwp", 4, &[(4, 0, 3), (0, 1, 0)]);
}

#[test]
fn issue_850_exam_science_answer_sheet_name_cell_keeps_outer_path() {
    let mut doc = load_sample("exam_science.hwp");
    assert_name_insert_by_path(&mut doc, "exam_science.hwp", 6, &[(6, 0, 3), (0, 1, 0)]);
}

#[test]
fn issue_850_exam_social_overlay_images_api_stays_compact_for_input_loop() {
    let doc = load_sample("exam_social.hwp");
    let overlay_json = doc
        .get_page_overlay_images_native(0)
        .expect("overlay image json");
    let layer_json = doc
        .get_page_layer_tree_native(0)
        .expect("full page layer tree json");
    let overlay: Value = serde_json::from_str(&overlay_json)
        .unwrap_or_else(|e| panic!("parse overlay json `{overlay_json}`: {e}"));

    assert_eq!(overlay["behind"].as_array().map(Vec::len), Some(0));
    assert_eq!(overlay["front"].as_array().map(Vec::len), Some(0));
    assert!(
        overlay["imageCount"].as_u64().unwrap_or(0) > 0,
        "flow images must still be counted for decode retry scheduling: {overlay_json}"
    );
    assert!(
        overlay_json.len() < 128,
        "input loop overlay JSON must remain compact: len={}, json={overlay_json}",
        overlay_json.len()
    );
    assert!(
        layer_json.len() > 1_000_000,
        "test fixture should demonstrate the avoided full layer JSON cost: len={}",
        layer_json.len()
    );
}
