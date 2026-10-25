//! #8153: 병합 셀의 안쪽 열 간격과 걸침 셀의 동일한 물리 경계를 보존한다.
//! 두 원본 35쪽은 독립 Print와 Native/fresh WASM raw100인 정상 대조군이다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn find<'a>(node: &'a RenderNode, row: u16, col: u16, out: &mut Vec<&'a RenderNode>) {
    if let RenderNodeType::TableCell(cell) = &node.node_type {
        if cell.row == row && cell.col == col {
            out.push(node);
        }
    }
    for child in &node.children {
        find(child, row, col, out);
    }
}

fn check(extension: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 383, "독립 Print 쪽수");
    let tree = core.build_page_render_tree(34).unwrap();
    let mut spanning = Vec::new();
    let mut next = Vec::new();
    find(&tree.root, 6, 2, &mut spanning);
    find(&tree.root, 7, 1, &mut next);
    assert_eq!(spanning.len(), 1, "민원 문서의 걸침 셀");
    assert_eq!(next.len(), 1, "그 다음 행의 일반문서 셀");
    let (spanning, next) = (spanning[0], next[0]);
    let actual_gap = spanning.bbox.x - (next.bbox.x + next.bbox.width);
    let source_gap = 255.0 * 96.0 / 7200.0;
    assert!(
        (actual_gap - source_gap).abs() <= 96.0 / 7200.0,
        "같은 걸침 셀 앞에는 원본 셀 간격 한 개만 남는다: 실제 {actual_gap}, 원본 {source_gap}"
    );
    assert!(next.bbox.y >= spanning.bbox.y);
    assert!(next.bbox.y + next.bbox.height <= spanning.bbox.y + spanning.bbox.height);
}
#[test]
fn hwp_merged_spacing_preserves_the_covered_cell_frame() {
    check("hwp");
}
#[test]
fn hwpx_merged_spacing_preserves_the_covered_cell_frame() {
    check("hwpx");
}
