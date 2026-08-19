//! Issue #5590: 행마다 다른 열 구획을 선언한 표에서, 전역 열 grid 하나로 모든 행을
//! 그리다 어느 행의 셀 폭이 원본과 어긋나던 회귀의 가드.
//!
//! 보고 문서 00288(약장 배치표)은 **모든 행의 셀 폭 합이 표 폭과 정확히 같은데도**
//! 마지막 열이 1,006HU(13.4px) 깎였다 — 전역 grid 가 앞 열들을 다른 행 기준으로 풀고
//! 남은 폭을 마지막 열에 떠넘긴 결과다. 코퍼스 표본에서 1,255표 중 62표(4.9%)가 같은 축.
//!
//! 재현 문서 (tracked 합성 샘플): `samples/issue5590_per_row_column_widths.hwpx`
//! - 6열 2행, 표 폭 36,000HU(480px). 두 행 모두 **선언 폭 합 = 480px** 로 자기 구획을
//!   완결하지만 구획이 서로 어긋난다.
//!   - row0: c0(span2) 160 · c2 80 · c3 80 · c4 80 · c5 80
//!   - row1: c0 60 · c1(span2) 200 · c3 80 · c4(span2) 140
//! - 수정 전: row0 c2 = 100(+20), row1 c4 = 160(+20), 표 상자 500(+20).
//! - 수정 후: 두 행 모두 선언대로, 표 상자 480.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue5590_per_row_column_widths.hwpx";

/// (row, col) → 선언 폭(px). 96dpi 기준 HWPUNIT/75.
const EXPECTED: &[((u64, u64), f64)] = &[
    ((0, 0), 160.0),
    ((0, 2), 80.0),
    ((0, 3), 80.0),
    ((0, 4), 80.0),
    ((0, 5), 80.0),
    ((1, 0), 60.0),
    ((1, 1), 200.0),
    ((1, 3), 80.0),
    ((1, 4), 140.0),
];

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e))
}

fn collect(node: &serde_json::Value, cells: &mut Vec<((u64, u64), f64)>, table_w: &mut f64) {
    match node.get("type").and_then(|t| t.as_str()) {
        Some("Table") => {
            if let Some(w) = node
                .get("bbox")
                .and_then(|b| b.get("w"))
                .and_then(|w| w.as_f64())
            {
                *table_w = w;
            }
        }
        Some("Cell") => {
            if let (Some(r), Some(c), Some(w)) = (
                node.get("row").and_then(|v| v.as_u64()),
                node.get("col").and_then(|v| v.as_u64()),
                node.get("bbox")
                    .and_then(|b| b.get("w"))
                    .and_then(|w| w.as_f64()),
            ) {
                cells.push(((r, c), w));
            }
        }
        _ => {}
    }
    for child in node
        .get("children")
        .and_then(|c| c.as_array())
        .into_iter()
        .flatten()
    {
        collect(child, cells, table_w);
    }
}

#[test]
fn issue_5590_rows_keep_their_declared_column_widths() {
    let doc = load_doc();
    let json = doc.get_page_render_tree(0).expect("render tree page 0");
    let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");

    let mut cells = Vec::new();
    let mut table_w = 0.0;
    collect(&tree, &mut cells, &mut table_w);
    assert_eq!(cells.len(), EXPECTED.len(), "셀 수 불일치: {cells:?}");

    for (key, expected) in EXPECTED {
        let actual = cells
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, w)| *w)
            .unwrap_or_else(|| panic!("셀 (r{}, c{}) 없음", key.0, key.1));
        assert!(
            (actual - expected).abs() < 1.0,
            "셀 (r{}, c{}) 폭이 선언과 다르다 — #5590 회귀. 선언 {expected:.1}, 렌더 {actual:.1}",
            key.0,
            key.1
        );
    }

    // 행이 선언 폭에 맞으면 표 상자도 선언 폭이어야 한다 — 안 그러면 표 오른쪽에
    // 빈 띠가 남고 세로 테두리가 행 끝과 어긋난다.
    assert!(
        (table_w - 480.0).abs() < 1.0,
        "표 상자 폭이 선언(480.0)과 다르다: {table_w:.1}"
    );
}
