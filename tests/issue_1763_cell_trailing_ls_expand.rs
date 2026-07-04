//! Issue #1763: 다문단 셀의 마지막 줄 trailing line_spacing 이 선언 셀높이를 초과
//! 확장시키면 안 된다 (한컴 선언높이 권위).
//!
//! Regression shape (samples/task1763/cell_trailing_ls_expand.hwp):
//! - 12×16 TAC 표 row0 셀(선언 h=10668HU=142.24px, 5문단: 빈 줄 + 대형 폰트 제목 +
//!   텍스트 3줄, 콘텐츠 스팬 10016HU).
//! - 수정 전: 측정이 셀 마지막 줄 trailing ls(600HU=8px)를 포함 → required 149.1px >
//!   선언 → 행 확장 +7px (한글 find_tables 는 142.1px = 선언 유지).
//! - 수정 후: trailing 제외 콘텐츠+pad 가 선언 안이므로 선언높이로 clamp → 142.2px.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/task1763/cell_trailing_ls_expand.hwp";

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e))
}

fn find_row0_cell_height(node: &serde_json::Value) -> Option<f64> {
    if node.get("type").and_then(|t| t.as_str()) == Some("Table") {
        for c in node.get("children")?.as_array()? {
            if c.get("type").and_then(|t| t.as_str()) == Some("Cell")
                && c.get("row").and_then(|r| r.as_u64()) == Some(0)
            {
                return c.get("bbox")?.get("h")?.as_f64();
            }
        }
    }
    for c in node
        .get("children")
        .and_then(|c| c.as_array())
        .into_iter()
        .flatten()
    {
        if let Some(h) = find_row0_cell_height(c) {
            return Some(h);
        }
    }
    None
}

#[test]
fn issue_1763_row0_respects_declared_cell_height() {
    let doc = load_doc();
    let json = doc.get_page_render_tree(0).expect("render tree page 0");
    let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");
    let h = find_row0_cell_height(&tree).expect("row0 cell height");

    // 선언 셀높이 10668HU = 142.24px. trailing ls 포함 확장(149.1px) 회귀 방지.
    assert!(
        (h - 142.2).abs() < 1.0,
        "row0 셀 높이는 선언높이(≈142.2px)를 유지해야 한다 (한글 정합), got {h}"
    );
}
