//! #6311의 실제 틀은 #6619에서 확인한 cellzone이 소유한다.
//! zone이 없는 기존 합성 입력의 독립 Print는 제목 칸 네 변만 출력한다.
//! #8153의 Native/fresh WASM raw100 확인 후 잘못된 표 외곽 기대값을 정정했다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn collect_lines(node: &RenderNode, out: &mut Vec<(f64, f64, f64, f64, f64)>) {
    if let RenderNodeType::Line(line) = &node.node_type {
        out.push((line.x1, line.y1, line.x2, line.y2, line.style.width));
    }
    for child in &node.children {
        collect_lines(child, out);
    }
}

fn find_table(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        return Some(node);
    }
    node.children.iter().find_map(find_table)
}

#[test]
fn issue_6311_without_cellzone_preserves_title_only_borders() {
    // 기존 callout_document()가 생성한 입력을 바이트 그대로 보존했다.
    // 독립 한컴 2020 Print: pdf/issue8153/6311-synthetic-no-cellzone-2020.pdf.
    let bytes =
        include_bytes!("../fixtures/issue_8153_cell_none_border/6311_synthetic_no_cellzone.hwpx");
    let core = DocumentCore::from_bytes(bytes).expect("독립 Print와 동일한 합성 입력");
    let page = core.build_page_render_tree(0).expect("render tree");
    let table = find_table(&page.root).expect("제목과 열린 본문 칸의 표");
    let title = table
        .children
        .iter()
        .find(|node| {
            matches!(&node.node_type, RenderNodeType::TableCell(cell) if cell.row == 0 && cell.col == 1)
        })
        .expect("테두리를 소유한 제목 칸");
    let b = title.bbox;
    let (left, top, right, bottom) = (b.x, b.y, b.x + b.width, b.y + b.height);
    assert!(b.width > 0.0 && b.height > 0.0);
    assert!(left > table.bbox.x && bottom < table.bbox.y + table.bbox.height);

    // 제목 칸의 네 변을 관계로 보호한다. 바깥 빈 칸을 덮는 선은 허용하지 않는다.
    let perimeter = [
        (left, top, right, top),
        (left, bottom, right, bottom),
        (left, top, left, bottom),
        (right, top, right, bottom),
    ];
    // 용지 상단의 clip은 중심선을 획 안쪽으로 이동시킬 수 있다.
    // 임의 px 핀 대신 해당 획 너비의 영역이 제목 칸의 변을 소유하는지 확인한다.
    let same_edge = |a: (f64, f64, f64, f64, f64), b: (f64, f64, f64, f64)| {
        assert!(a.4.is_finite() && a.4 > 0.0);
        let same_point = |x: f64, y: f64, bx: f64, by: f64| {
            (x - bx).abs() <= a.4 + 1e-6 && (y - by).abs() <= a.4 + 1e-6
        };
        (same_point(a.0, a.1, b.0, b.1) && same_point(a.2, a.3, b.2, b.3))
            || (same_point(a.0, a.1, b.2, b.3) && same_point(a.2, a.3, b.0, b.1))
    };
    let mut lines = Vec::new();
    collect_lines(table, &mut lines);
    for edge in perimeter {
        assert!(
            lines.iter().any(|line| same_edge(*line, edge)),
            "제목 칸의 원본 테두리 보존: {edge:?}, actual={lines:?}"
        );
    }
    assert!(
        lines
            .iter()
            .all(|line| perimeter.iter().any(|edge| same_edge(*line, *edge))),
        "zone이 없는 열린 칸을 표 기본 테두리로 닫으면 안 된다: {lines:?}"
    );
}
