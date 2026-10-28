//! 관계형·고정형: 문단 사이의 로컬 좌표 재시작과 표 뒤 본문 위치를 보호한다.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! HWP 223쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 223쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 같은 원본·독립 Print의 223쪽은 Native/표준 최적화 fresh WASM 모두 raw 100%다.
//! 근거: mydocs/plans/issue_8153_ordinary_row_remainder_20261027.md
//! 글꼴 공급이나 Native paint 없이 원본 저장 줄의 소유·후속 본문 위치를 검사한다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile},
    DocumentCore,
};
use std::path::PathBuf;
fn headings(node: &LayerNode, found: &mut Vec<(String, f64)>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                headings(child, found)
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            headings(child, found)
        }
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                if let PaintOp::TextRun { bbox, run, .. } = op {
                    if run.para_index == Some(243)
                        && run.cell_context.is_none()
                        && run.text.contains("정책연구 수행")
                    {
                        assert!(run.style.font_position.is_none());
                        found.push((run.text.clone(), bbox.y + run.baseline));
                    }
                }
            }
        }
    }
}
fn check(ext: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{ext}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 383);
    let original = core.export_hwp_native().unwrap();
    let page = core.dump_page_items_json(Some(222));
    let items = page[0]["columns"][0]["items"].as_array().unwrap();
    let table = items
        .iter()
        .position(|i| i["kind"] == "partialTable" && i["paraIndex"] == 239)
        .unwrap();
    let heading = items
        .iter()
        .position(|i| i["kind"] == "fullParagraph" && i["paraIndex"] == 243)
        .unwrap();
    assert!(table < heading);
    assert_eq!(items[table]["startCut"], serde_json::json!([1, 44, 1]));
    assert_eq!(items[table]["endRow"], 3);
    assert_eq!(items[table]["endCut"], serde_json::json!([]));
    for (index, id) in [240, 241, 242].into_iter().enumerate() {
        assert_eq!(
            items[table + index + 1]["paraIndex"],
            id,
            "표 뒤의 빈 문단·주석 순서 보존"
        );
    }
    let tree = core
        .build_page_layer_tree_with_profile(222, RenderProfile::Print)
        .unwrap();
    let mut found = Vec::new();
    headings(&tree.root, &mut found);
    assert_eq!(found.len(), 1, "후속 제목 누락·중복");
    assert!(
        (found[0].1 - 556.0).abs() <= 2.0,
        "{ext}: 독립 Print 제목 기준선 556±2px, 실제 {}",
        found[0].1
    );
    assert_eq!(core.export_hwp_native().unwrap(), original);
}
#[test]
fn hwp_paragraph_local_restart_does_not_extend_completed_table_into_following_body() {
    check("hwp")
}
#[test]
fn hwpx_paragraph_local_restart_does_not_extend_completed_table_into_following_body() {
    check("hwpx")
}
