//! 이어받기 조각의 세로 괘선은 같은 밴드를 소유하는 실제 셀 경계와 일치한다.
//! 같은 원본/독립 한컴 Print의 Native·fresh WASM 전31쪽 최저93.45386% 확인 뒤 등록했다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    document_core::DocumentCore,
    renderer::render_tree::{RenderNode, RenderNodeType},
};
use std::path::Path;
fn outer_table(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        return Some(node);
    }
    node.children.iter().find_map(outer_table)
}
fn same_coordinate(a: f64, b: f64) -> bool {
    // SVG 속성 직렬화와 부동소수 덧셈 오차만 허용한다. 문서의 위치를 고정하지 않는다.
    (a - b).abs() <= 1e-8 * a.abs().max(b.abs()).max(1.0)
}
#[test]
fn issue_5877_continuation_fragment_uses_its_own_column_grid() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/issue5877/fragment_ghost_vrules.hwp");
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("원본")).expect("열기");
    let page = core.build_page_render_tree(14).expect("15쪽 실제 셀");
    let table = outer_table(&page.root).expect("바깥 표 조각");
    let cells: Vec<_> = table
        .children
        .iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::TableCell(_)))
        .collect();
    assert!(!cells.is_empty(), "실제 셀 경계가 있어야 검증이 유효하다");
    let top = cells.iter().map(|n| n.bbox.y).fold(f64::INFINITY, f64::min);
    let end = cells
        .iter()
        .filter(|n| same_coordinate(n.bbox.y, top))
        .map(|n| n.bbox.y + n.bbox.height)
        .fold(f64::INFINITY, f64::min);
    assert!(end > top, "양수 상단 밴드");
    let probe = (top + end) / 2.0;
    let mut boundaries: Vec<_> = cells
        .iter()
        .filter(|n| n.bbox.y < probe && probe < n.bbox.y + n.bbox.height)
        .flat_map(|n| [n.bbox.x, n.bbox.x + n.bbox.width])
        .collect();
    boundaries.sort_by(f64::total_cmp);
    boundaries.dedup_by(|a, b| same_coordinate(*a, *b));
    assert!(boundaries.len() > 2, "여러 내부 열을 가진 원본 밴드");
    let svg = core.render_page_svg_native(14).expect("같은 쪽 괘선 출력");
    let mut verticals = Vec::new();
    for cap in svg.split("<line ").skip(1) {
        let head = &cap[..cap.find('>').unwrap_or(cap.len())];
        let (Some(x1), Some(x2), Some(y1), Some(y2)) = (
            attr(head, "x1=\""),
            attr(head, "x2=\""),
            attr(head, "y1=\""),
            attr(head, "y2=\""),
        ) else {
            continue;
        };
        if same_coordinate(x1, x2) && y1.min(y2) < probe && probe < y1.max(y2) {
            verticals.push(x1);
        }
    }
    assert!(!verticals.is_empty(), "그 밴드를 지나는 괘선");
    for x in &verticals {
        assert!(
            boundaries.iter().any(|b| same_coordinate(*x, *b)),
            "원본 다른 행의 균등 격자를 그리면 실제 셀 경계와 어긋난다: {x}, {boundaries:?}"
        );
    }
    for x in boundaries {
        assert!(
            verticals.iter().any(|v| same_coordinate(x, *v)),
            "실제 셀 경계 괘선을 빠뜨리면 안 된다: {x}, {verticals:?}"
        );
    }
}
fn attr(head: &str, key: &str) -> Option<f64> {
    let rest = head.split_once(key)?.1;
    rest[..rest.find('"')?].parse().ok()
}
