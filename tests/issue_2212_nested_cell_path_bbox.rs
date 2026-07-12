//! Issue #2212: 중첩 표 셀 경로 bbox 해석 실패.
//!
//! 주보(p1 좌측) 표 pi=5 의 셀[10] 안 인라인 TAC 내부 표(18×9)는 run_tacs
//! 경로(paragraph_layout)로 렌더되는데, 이 경로가 cell_context 를 None 으로
//! 전달해 내부 표 TextRun 에 2단 경로가 기록되지 않았다 → studio 의
//! updateCellSelection/renderTableObjectSelection 이 사용하는
//! `get_table_cell_bboxes_by_path` 가 매칭 실패로 예외를 반복했다.
//!
//! 정정: run_tacs 인라인 TAC 표 렌더에 외곽 셀 경로를 확장한 2단
//! cell_context 전달 (table_layout 중첩 분기와 동일 규칙).

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

fn has_two_level_ctx(n: &RenderNode) -> bool {
    if let RenderNodeType::TextRun(tr) = &n.node_type {
        if let Some(ctx) = &tr.cell_context {
            if ctx.parent_para_index == 5 && ctx.path.len() == 2 && ctx.path[0].cell_index == 10 {
                return true;
            }
        }
    }
    n.children.iter().any(has_two_level_ctx)
}

#[test]
fn issue_2212_nested_inline_tac_table_carries_two_level_cell_context() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path =
        Path::new(repo_root).join("samples/basic/issue1994_behindtext_table_20200830.hwp");
    let bytes =
        fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse issue1994_behindtext_table_20200830.hwp");
    let tree = doc.build_page_render_tree(0).expect("render page 1");

    // 1) 내부 표 TextRun 에 2단 경로가 기록된다 (수정 전: 1단 11종뿐 → FAILED).
    assert!(
        has_two_level_ctx(&tree.root),
        "내부 표 TextRun 에 2단 cell_context(외곽 셀10 경유)가 없음 — #2212 회귀"
    );

    // 2) studio 가 쓰는 경로 기반 bbox 조회가 성공하고 내부 셀들을 돌려준다.
    //    (수정 전: Err — 콘솔 '경로 기반 표 셀 bbox를 찾을 수 없습니다' 반복)
    let bboxes = doc
        .get_table_cell_bboxes_by_path(
            0,
            5,
            r#"[{"controlIndex":0,"cellIndex":10,"cellParaIndex":0},{"controlIndex":0,"cellIndex":46,"cellParaIndex":0}]"#,
        )
        .expect("경로 기반 내부 표 셀 bbox 조회");
    let cell_count = bboxes.matches("\"cellIdx\"").count();
    assert!(
        cell_count >= 40,
        "내부 18×9 표(셀 48)의 bbox 개수 부족: {cell_count} — #2212 회귀"
    );
}
