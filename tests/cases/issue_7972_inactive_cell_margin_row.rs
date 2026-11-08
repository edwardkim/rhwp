//! [#7972] `hasMargin=0` 칸의 보존 cellMargin 은 비활성이다 — 모순 칸의 재성장 여백은 실효 여백이다.
//!
//! 선언 높이가 저장 줄보다 작은 모순 칸은 한/글이 줄 흐름 + 위·아래 안 여백으로 키운다(#3386).
//! 그 여백은 칸이 실제로 쓰는 **실효** 안 여백이다 — `hasMargin=0` 칸은 표 안 여백(inMargin)을
//! 쓰고 자기 cellMargin 은 쓰지 않는다. 수정 전 rhwp 는 칸 안 중첩 표 경로에서 보존 cellMargin
//! 을 더해, 표 안 여백 0 인 표의 행이 한/글보다 3.76px(141HU×2) 높았다(#7872 제보 〈보기〉 표).
//!
//! 입력 `samples/issue7972/inactive-cell-margin.hwpx` 는 생성기 `gen_inactive_cell_margin.py` 의
//! 출력이고, 정본 `pdf/issue7972/inactive-cell-margin-2024.pdf` 는 한/글 2024(13.0.0.564) Print 다.
//! 두 중첩 3×4 표의 모든 칸은 `hasMargin=0`·보존 cellMargin 위·아래 141HU·선언 높이 200HU·저장 줄
//! 900HU 다. 정본 행 높이: 표 1(표 안 여백 0) 12.0px, 표 2(표 안 여백 141) 15.76px.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const SAMPLE: &str = "samples/issue7972/inactive-cell-margin.hwpx";
const LINE_PX: f64 = 900.0 * 96.0 / 7200.0;
const TABLE_MARGIN_PX: f64 = 2.0 * 141.0 * 96.0 / 7200.0;

/// 칸 안 중첩 표마다 행 높이(위에서 아래로).
fn nested_row_heights(doc: &HwpDocument) -> Vec<Vec<f64>> {
    let tree: Value = serde_json::from_str(&doc.get_page_render_tree(0).unwrap()).unwrap();
    let mut tables = Vec::new();
    fn walk(node: &Value, inside_cell: bool, out: &mut Vec<Vec<f64>>) {
        let kind = node["type"].as_str().unwrap_or_default();
        if kind == "Table" && inside_cell {
            let mut rows: Vec<(f64, f64)> = node["children"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|child| child["type"] == "Cell")
                .map(|cell| {
                    let y = cell["bbox"]["y"].as_f64().unwrap();
                    (y, y + cell["bbox"]["h"].as_f64().unwrap())
                })
                .collect();
            rows.sort_by(|a, b| a.partial_cmp(b).unwrap());
            rows.dedup_by(|a, b| (a.0 - b.0).abs() < 0.01);
            out.push(rows.iter().map(|(top, bottom)| bottom - top).collect());
        }
        for child in node["children"].as_array().into_iter().flatten() {
            walk(child, inside_cell || kind == "Cell", out);
        }
    }
    walk(&tree, false, &mut tables);
    tables
}

#[test]
fn inactive_cell_margin_does_not_regrow_contradicting_rows() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let doc = HwpDocument::from_bytes(&std::fs::read(&path).expect("read")).expect("표본 파싱");
    let tables = nested_row_heights(&doc);
    assert_eq!(tables.len(), 2, "칸 안 중첩 표 두 개: {tables:?}");
    for (index, (rows, expected)) in tables
        .iter()
        .zip([LINE_PX, LINE_PX + TABLE_MARGIN_PX])
        .enumerate()
    {
        assert_eq!(rows.len(), 3, "표 {} 행 수: {rows:?}", index + 1);
        for (row, height) in rows.iter().enumerate() {
            assert!(
                (height - expected).abs() <= 0.6,
                "표 {} 행 {row}: {height:.2}px — 한/글 {expected:.2}px \
                 (줄 + 실효 안 여백; 수정 전 표 1 은 비활성 cellMargin 3.76px 가 더해졌다)",
                index + 1
            );
        }
    }
}
