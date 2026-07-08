//! Issue #2020: 첨부 문서 렌더링 차이 회귀 게이트.
//!
//! 첨부/참조 문서는 하나의 이슈 범위에서 다룬다. 이 테스트는 자동 판정 가능한
//! 페이지 수와 FSC HWP/HWPX 흐름 동기화를 먼저 고정한다.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;
use std::fs;
use std::path::Path;

fn load_doc(rel_path: &str) -> HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel_path);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {rel_path}: {e}"));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {rel_path}: {e:?}"))
}

fn has_table(root: &RenderNode, para_index: usize, control_index: usize) -> bool {
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if let RenderNodeType::Table(table) = &node.node_type {
            if table.para_index == Some(para_index) && table.control_index == Some(control_index) {
                return true;
            }
        }
        for child in &node.children {
            stack.push(child);
        }
    }
    false
}

#[test]
fn issue_2020_reference_documents_keep_expected_page_counts() {
    assert_eq!(
        load_doc("samples/issue2020/passport_application_lawgo.hwp").page_count(),
        2
    );
    assert_eq!(
        load_doc("samples/issue2020/fsc_20250813.hwp").page_count(),
        5
    );
    assert_eq!(
        load_doc("samples/issue2020/fsc_20250813.hwpx").page_count(),
        5
    );
    assert_eq!(load_doc("samples/복학원서.hwp").page_count(), 1);
    assert_eq!(
        load_doc("samples/2022년 국립국어원 업무계획.hwp").page_count(),
        35
    );
}

#[test]
fn issue_2020_fsc_hwp_keeps_tail_table_on_page_two() {
    let doc = load_doc("samples/issue2020/fsc_20250813.hwp");
    let tree = doc
        .build_page_render_tree(1)
        .expect("render FSC HWP page 2");

    assert!(
        has_table(&tree.root, 24, 0),
        "FSC HWP pi=24 14x15 표는 HWPX/한컴 기준처럼 2쪽 하단에 남아야 한다"
    );
}
