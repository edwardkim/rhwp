//! Issue #717: 표 셀 빈 영역 클릭 시 커서가 다른 위치로 이동.
//!
//! 재현 문서: `samples/exam_social.hwp`
//! 대상: 1/4쪽 왼쪽 첫 번째 자료 표 제목 행의 빈 영역.

use std::path::Path;

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn load_exam_social() -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/exam_social.hwp");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    HwpDocument::from_bytes(&bytes).expect("parse exam_social.hwp")
}

fn hit_json(doc: &HwpDocument, page: u32, x: f64, y: f64) -> Value {
    let json = doc
        .hit_test_native(page, x, y)
        .unwrap_or_else(|e| panic!("hit_test_native({page}, {x}, {y}): {e}"));
    serde_json::from_str(&json).unwrap_or_else(|e| panic!("parse hit json `{json}`: {e}"))
}

fn assert_table_hit(
    hit: &Value,
    parent_para: u64,
    control: u64,
    y_range: std::ops::RangeInclusive<f64>,
) {
    assert_eq!(hit["sectionIndex"].as_u64(), Some(0), "hit={hit}");
    assert_eq!(
        hit["parentParaIndex"].as_u64(),
        Some(parent_para),
        "cell whitespace click must stay in clicked table, hit={hit}"
    );
    assert_eq!(
        hit["controlIndex"].as_u64(),
        Some(control),
        "cell whitespace click must stay in clicked table, hit={hit}"
    );
    assert!(
        hit.get("cellIndex").is_some(),
        "cell whitespace click must return a cell context, hit={hit}"
    );
    assert!(
        hit.get("cellPath").is_some(),
        "cell whitespace click must preserve a cellPath, hit={hit}"
    );
    let cursor_y = hit["cursorRect"]["y"].as_f64().expect("cursorRect.y");
    assert!(
        y_range.contains(&cursor_y),
        "caret must stay inside clicked table bbox, hit={hit}"
    );
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

#[test]
fn issue_717_exam_social_title_empty_area_stays_in_clicked_table() {
    let doc = load_exam_social();

    // 이슈 #717 기준 좌표: page 0, 왼쪽 첫 번째 자료 표 제목 행의 빈 영역.
    // SVG debug overlay 대상 표: s0:pi=1 ci=0 1x1, bbox≈x85.7 y324.9 w411.9 h197.0.
    let hit = hit_json(&doc, 0, 191.0, 356.0);

    assert_table_hit(&hit, 1, 0, 320.0..=530.0);
}

#[test]
fn issue_717_exam_social_nested_header_empty_area_returns_editable_path() {
    let mut doc = load_exam_social();

    // 왼쪽 자료 표의 회색 헤더 내부표에서 텍스트 왼쪽 빈 영역.
    // 기존에는 내부표 cellIndex=1을 최외곽 표의 cellIndex처럼 반환하여
    // Studio 입력 라우팅이 존재하지 않는 외곽 셀로 향했다.
    let hit = hit_json(&doc, 0, 100.0, 350.0);

    assert_table_hit(&hit, 1, 0, 330.0..=370.0);
    assert_eq!(
        hit["cellIndex"].as_u64(),
        Some(0),
        "public cellIndex must remain the outer table cell, hit={hit}"
    );
    assert_eq!(
        path_tuples(&hit),
        vec![(0, 0, 0), (1, 1, 0)],
        "nested table hit must preserve the full editable path, hit={hit}"
    );

    let path = path_tuples(&hit);
    doc.insert_text_in_cell_by_path(0, 1, &path, hit["charOffset"].as_u64().unwrap() as usize, "X")
        .unwrap_or_else(|e| panic!("insert_text_in_cell_by_path failed: {e}"));
    let inserted = doc
        .get_text_in_cell_by_path(0, 1, &path, 0, 1)
        .expect("get inserted text");
    assert_eq!(inserted, "X");

    let rect_json = doc
        .get_cursor_rect_by_path(0, 1, &serde_json::to_string(&hit["cellPath"]).unwrap(), 1)
        .unwrap_or_else(|e| panic!("cursor rect after nested cell insertion: {e:?}"));
    let rect: Value = serde_json::from_str(&rect_json).expect("cursor rect json");
    let rect_y = rect["y"].as_f64().expect("rect.y");
    assert!(
        (330.0..=370.0).contains(&rect_y),
        "inserted text caret must stay in nested header cell, rect={rect}"
    );
}

#[test]
fn issue_717_exam_social_view_table_empty_area_stays_in_clicked_table() {
    let doc = load_exam_social();

    // 같은 페이지 왼쪽 <보기> 표 빈 영역. dump-pages 기준 s0:pi=6 ci=0 3x3.
    let hit = hit_json(&doc, 0, 110.0, 865.0);

    assert_table_hit(&hit, 6, 0, 845.0..=975.0);
}
