//! 쪽에서 나뉜 표의 뒤쪽 조각에만 놓인 칸은 첫 문단이 글자처럼 둔 중첩 표만 품어도 캐럿 자리가 있다.
//! 전에는 빈 칸 대체 경로가 표 첫 쪽만 뒤져 "셀 커서 위치를 찾을 수 없습니다"로 실패했다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

// 바깥 2칸 표가 쪽을 넘고, 둘째 칸(cell 1)은 다음 쪽에만 있으며 그 첫 문단은 중첩 표 하나뿐이다.
const SAMPLE: &str = "tests/fixtures/issue7419-boundaries/partial-nested-wrapper.hwpx";

fn caret(doc: &HwpDocument, cell: usize) -> Value {
    let rect = doc
        .get_cursor_rect_in_cell_native(0, 4, 0, cell, 0, 0)
        .unwrap_or_else(|e| panic!("cell {cell} 캐럿: {e}"));
    serde_json::from_str(&rect).unwrap()
}

#[test]
fn continuation_cell_with_only_a_nested_table_has_a_caret() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let doc = HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let first = caret(&doc, 0);
    let second = caret(&doc, 1);
    // 둘째 칸 캐럿은 그 칸이 놓인 뒤쪽에 선다.
    assert!(
        second["pageIndex"].as_u64() > first["pageIndex"].as_u64(),
        "첫 칸 {first} / 둘째 칸 {second}"
    );
}
