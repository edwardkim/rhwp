//! [#8105] 내용 컷으로 나뉜 행의 선언 높이가 첫 조각 상자보다 크면 첫 조각을 닫는 선까지 닫는다.
//!
//! 걸침 칸이 없어도 나뉜 행 자신의 선언 높이가 남으면 그 행이 쪽 끝 띠를 소유한다. 한/글 2020
//! (정본 `pdf/task1749/saved_bounds_cumulative_page_break-hwpx-2020.pdf`) 4쪽 표 문단 52 의 행 2 는
//! 아래 괘선을 닫는 선(본문 아래 − 바깥 아래 여백 − 100HU)에 긋고, 5쪽 조각은 선언에서 그 상자를
//! 뺀 나머지다. 수정 전 rhwp 는 4쪽 상자를 내용 높이(83.0px)에서 닫고 남는 4.4px 를 5쪽 조각에
//! 실어 5쪽 전체가 내려갔다(Visual Sweep 71.0 → 100.0).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/task1749/saved_bounds_cumulative_page_break.hwpx";

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("parse task1749 hwpx")
}

fn find<'a>(node: &'a RenderNode, pred: &dyn Fn(&RenderNode) -> bool) -> Option<&'a RenderNode> {
    if pred(node) {
        return Some(node);
    }
    node.children.iter().find_map(|child| find(child, pred))
}

/// `(본문 아래, 칸 (행, 열) 의 위, 아래)` — 표 문단 `para` 의 이 쪽 조각.
fn body_bottom_and_cell(
    core: &DocumentCore,
    page: u32,
    para: usize,
    cell: (u16, u16),
) -> (f64, f64, f64) {
    let tree = core
        .build_page_render_tree(page)
        .unwrap_or_else(|error| panic!("{}쪽 렌더 트리: {error:?}", page + 1));
    let body = find(&tree.root, &|node| {
        matches!(node.node_type, RenderNodeType::Body { .. })
    })
    .expect("본문 영역");
    let table = find(
        &tree.root,
        &|node| matches!(&node.node_type, RenderNodeType::Table(t) if t.para_index == Some(para)),
    )
    .unwrap_or_else(|| panic!("{}쪽 표 문단 {para}", page + 1));
    let cell_node = find(
        table,
        &|node| matches!(&node.node_type, RenderNodeType::TableCell(c) if (c.row, c.col) == cell),
    )
    .unwrap_or_else(|| panic!("{}쪽 칸 {cell:?}", page + 1));
    (
        body.bbox.y + body.bbox.height,
        cell_node.bbox.y,
        cell_node.bbox.y + cell_node.bbox.height,
    )
}

#[test]
fn split_row_first_fragment_closes_at_the_closing_line() {
    let core = core();
    let Control::Table(table) = &core.document().sections[0].paragraphs[52].controls[0] else {
        panic!("문단 52 첫 개체는 표");
    };
    let declared = table
        .cells
        .iter()
        .filter(|cell| cell.row == 2 && cell.row_span == 1)
        .map(|cell| cell.height as f64 / 75.0)
        .fold(0.0, f64::max);
    let (body_bottom, top4, bottom4) = body_bottom_and_cell(&core, 3, 52, (2, 0));
    let closing = body_bottom - (table.outer_margin_bottom as f64 + 100.0) / 75.0;
    assert!(
        (bottom4 - closing).abs() <= 0.5,
        "4쪽 행 2 아래 = 닫는 선 {closing:.2}(정본 괘선 1024.0pt 척도), 실제 {bottom4:.2}"
    );
    let (_, top5, bottom5) = body_bottom_and_cell(&core, 4, 52, (2, 0));
    let first = bottom4 - top4;
    let rest = bottom5 - top5;
    assert!(
        declared > first && (first + rest - declared).abs() <= 0.5,
        "행 2 선언 {declared:.2}px = 4쪽 {first:.2} + 5쪽 {rest:.2}"
    );
}
