//! [#3593] 셀 문단의 `LINE_SEG.vertical_pos == 0` 을 절대 앵커로 쓰지 않는다 — 회귀 가드.
//!
//! `vertical_pos == 0` 은 첫 문단에서는 "셀 상단"이라는 유효값이지만, 둘째 이후
//! 문단에서는 "앵커 없음"의 센티널이다. 이를 절대 위치로 받아들이면 셀 안 모든
//! 문단이 같은 y 로 리셋되어 겹쳐 그려진다.
//!
//! 대상 셀 — `samples/issue1949_giant_cell_nested_tables_perf.hwp`의
//! 중첩 표 안쪽 셀. 두 문단 모두 `vpos=0 lh=1000` 으로 저장돼 있다.
//! 다른 pagination 수정이 해당 셀의 페이지 번호를 바꿀 수 있으므로, 특정 페이지를
//! 고정하지 않고 순차 조판한 전체 트리에서 고유한 문단 쌍을 찾는다.
//!
//! ```text
//! para[0] "로터리 조종기"      line_segs=(vpos=0 lh=1000)
//! para[1] "(Rotary controls)"  line_segs=(vpos=0 lh=1000)
//! ```
//!
//! 두 가지를 함께 고정한다.
//! 1. 배치 — 두 문단의 줄이 서로 다른 y 에 놓인다(겹치지 않는다).
//! 2. 측정 — 그렇게 쌓인 줄이 셀 높이 안에 담긴다(행 괘선을 넘지 않는다).
//!
//! 2 는 1 과 짝이다. 배치만 고치고 셀 높이 측정이 저장 vpos extent(= 1줄분)에
//! 머무르면, 둘째 줄이 행 아래 괘선을 관통한다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue1949_giant_cell_nested_tables_perf.hwp";
const PARA_A: &str = "로터리 조종기";
const PARA_B: &str = "(Rotary controls";

/// 노드 서브트리의 TextRun 텍스트를 이어붙인다.
fn subtree_text(node: &RenderNode) -> String {
    let mut out = String::new();
    fn walk(node: &RenderNode, out: &mut String) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            out.push_str(run.display_or_text());
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    walk(node, &mut out);
    out
}

/// 직속 TextLine 자식 중 `PARA_A` 와 `PARA_B` 를 모두 가진 셀을 찾는다.
fn find_target_cell(node: &RenderNode) -> Option<(BoundingBox, Vec<(String, BoundingBox)>)> {
    if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        let lines: Vec<(String, BoundingBox)> = node
            .children
            .iter()
            .filter(|c| matches!(c.node_type, RenderNodeType::TextLine(_)))
            .map(|c| (subtree_text(c), c.bbox))
            .collect();
        let has_a = lines.iter().any(|(t, _)| t.contains(PARA_A));
        let has_b = lines.iter().any(|(t, _)| t.contains(PARA_B));
        if has_a && has_b {
            return Some((node.bbox, lines));
        }
    }
    node.children.iter().find_map(find_target_cell)
}

fn load_target() -> (BoundingBox, Vec<(String, BoundingBox)>) {
    let bytes = std::fs::read(SAMPLE).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {SAMPLE}: {e:?}"));

    // 페이지를 처음부터 순차로 빌드한다 — 단일 페이지만 빌드하면 조판 상태가 달라
    // 대상 페이지의 트리가 전수 렌더와 일치하지 않는다.
    for page in 0..core.page_count() {
        let tree = core
            .build_page_render_tree(page)
            .unwrap_or_else(|e| panic!("build page {page}: {e:?}"));
        if let Some(cell) = find_target_cell(&tree.root) {
            return cell;
        }
    }
    panic!(
        "{SAMPLE} 전체 {}쪽에서 대상 셀을 찾지 못했다",
        core.page_count()
    )
}

#[test]
fn cell_paragraphs_with_zero_vpos_do_not_share_one_baseline() {
    let (_cell, lines) = load_target();
    let ya = lines
        .iter()
        .find(|(t, _)| t.contains(PARA_A))
        .map(|(_, b)| b.y)
        .expect("para A 줄");
    let yb = lines
        .iter()
        .find(|(t, _)| t.contains(PARA_B))
        .map(|(_, b)| b.y)
        .expect("para B 줄");

    assert!(
        (ya - yb).abs() >= 1.0,
        "셀 안 두 문단이 같은 y 에 겹쳐 그려졌다: {PARA_A}={ya:.1}, {PARA_B}={yb:.1}"
    );
}

#[test]
fn cell_height_contains_stacked_paragraph_lines() {
    let (cell, lines) = load_target();
    let content_bottom = lines
        .iter()
        .map(|(_, b)| b.y + b.height)
        .fold(f64::MIN, f64::max);
    let cell_bottom = cell.y + cell.height;

    assert!(
        content_bottom <= cell_bottom + 0.5,
        "쌓인 줄이 셀 높이를 넘었다 (행 괘선 침범): 콘텐츠 bottom={content_bottom:.1}, \
         셀 bottom={cell_bottom:.1} (셀 y={:.1} h={:.1}), 줄 {}개",
        cell.y,
        cell.height,
        lines.len()
    );
}
