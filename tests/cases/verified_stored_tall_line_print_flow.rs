//! 동일 원본의 한컴 Print가 입증한 큰 표 뒤 문단의 소속·순서·괘선을 보호한다.
//! Native/fresh WASM 100%·96 DPI 증적과 PDF 괘선 좌표는 fixture README를 따른다.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn nodes<'a>(node: &'a RenderNode, found: &mut Vec<&'a RenderNode>) {
    found.push(node);
    for child in &node.children {
        nodes(child, found);
    }
}

fn check(bytes: &[u8]) {
    let core = DocumentCore::from_bytes(bytes).expect("검증한 HWPX");
    assert_eq!(core.page_count(), 1, "독립 Print와 같은 전체 쪽수");
    let tree = core.build_page_render_tree(0).unwrap();
    let mut all = Vec::new();
    nodes(&tree.root, &mut all);
    let tables: Vec<_> = all
        .iter()
        .copied()
        .filter(|node| matches!(node.node_type, RenderNodeType::Table(_)))
        .collect();
    assert_eq!(tables.len(), 1);
    let table = tables[0];
    let text = |needle: &str| {
        let found: Vec<_> = all
            .iter()
            .copied()
            .filter(|node| {
                matches!(&node.node_type, RenderNodeType::TextRun(run)
                if run.display_or_text().contains(needle))
            })
            .collect();
        assert_eq!(found.len(), 1, "누락·중복 없이 보존: {needle}");
        found[0]
    };
    let heading = text("머리");
    let inside = text("표");
    let successor = text("다음 쪽");
    assert!(heading.bbox.y + heading.bbox.height <= table.bbox.y);
    assert!(table.bbox.y <= inside.bbox.y);
    assert!(inside.bbox.y + inside.bbox.height <= table.bbox.y + table.bbox.height);
    assert!(table.bbox.y + table.bbox.height <= successor.bbox.y);
    assert!(successor.bbox.y + successor.bbox.height <= tree.root.bbox.height);
    // 독립 PDFium path 잉크 범위(96 DPI). 논리 상자와 선 두께 차이는 ±2px로 허용한다.
    let actual = [
        table.bbox.x,
        table.bbox.y,
        table.bbox.width,
        table.bbox.height,
    ];
    let independent = [74.840001, 115.073324, 641.094650, 866.733348];
    for (actual, expected) in actual.into_iter().zip(independent) {
        assert!(
            (actual - expected).abs() <= 2.0,
            "PDF 괘선: {actual} vs {expected}"
        );
    }
}

#[test]
fn printed_tall_line_rewind_preserves_table_and_successor_on_one_page() {
    check(include_bytes!(
        "../fixtures/pr8036_stored_tall_line/zero.hwpx"
    ));
}

#[test]
fn printed_tall_line_continuation_preserves_table_and_successor_on_one_page() {
    check(include_bytes!(
        "../fixtures/pr8036_stored_tall_line/continuing.hwpx"
    ));
}
