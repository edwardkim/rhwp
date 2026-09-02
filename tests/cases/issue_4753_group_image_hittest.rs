//! [Issue #4753] 묶음 자식 Image의 페이지 전체 bbox가 160쪽 표 셀 hit-test를 가로채지 않는다.
//!
//! `samples/2025 행정업무운영 편람(최종).hwp` 사용자 160쪽(0-based 159)의 표
//! (section 4, paragraph 60, control 0) 위에는 treat_as_char 묶음 개체의 자식
//! Image가 페이지 전체 bbox로 올라온다. 그 Image를 본문 인라인 hit로 등록하면
//! 표 안 클릭이 전부 paragraph 0으로 떨어진다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const SAMPLE: &str = "samples/2025 행정업무운영 편람(최종).hwp";
const PAGE: u32 = 159;
const TABLE_SECTION: u64 = 4;
const TABLE_PARENT_PARA: u64 = 60;
const TABLE_CONTROL: u64 = 0;

fn load_handbook() -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    HwpDocument::from_bytes(&bytes).expect("parse handbook hwp")
}

fn hit_json(doc: &HwpDocument, x: f64, y: f64) -> Value {
    let json = doc
        .hit_test_native(PAGE, x, y)
        .unwrap_or_else(|e| panic!("hit_test_native({PAGE}, {x}, {y}): {e}"));
    serde_json::from_str(&json).unwrap_or_else(|e| panic!("parse hit json `{json}`: {e}"))
}

fn assert_table_cell_hit(hit: &Value, x: f64, y: f64) {
    assert_eq!(
        hit["sectionIndex"].as_u64(),
        Some(TABLE_SECTION),
        "({x},{y}) hit={hit}"
    );
    assert_eq!(
        hit["parentParaIndex"].as_u64(),
        Some(TABLE_PARENT_PARA),
        "({x},{y}) must enter the page-160 table, not the group-child image, hit={hit}"
    );
    assert_eq!(
        hit["controlIndex"].as_u64(),
        Some(TABLE_CONTROL),
        "({x},{y}) hit={hit}"
    );
    assert!(
        hit.get("cellIndex").is_some(),
        "({x},{y}) cell context required, hit={hit}"
    );
    assert!(
        hit.get("cellPath").and_then(|p| p.as_array()).is_some(),
        "({x},{y}) cellPath required, hit={hit}"
    );
}

#[test]
fn issue_4753_page160_table_cells_not_stolen_by_group_child_image() {
    let doc = load_handbook();
    // 표 bbox ≈ (113.4, 147.4, 532.6, 733.7). 내부 여러 지점이 모두 셀로 들어가야 한다.
    for &(x, y) in &[(200.0, 250.0), (320.0, 400.0), (500.0, 600.0)] {
        assert_table_cell_hit(&hit_json(&doc, x, y), x, y);
    }
}
