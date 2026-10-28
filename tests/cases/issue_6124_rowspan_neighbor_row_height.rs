//! [Issue #6124] 세로 병합 칸의 마지막 줄이 칸 하단 괘선에 잘린다
//! (2737927 별표 1, 8쪽 "정정 필요함 **").
//!
//! 근인: TAC 표 비례 축소(#5748 계약)의 행별 **내용 하한**이 `row_span == 1`
//! 셀만 본다. 4행 병합 평가방법 칸은 하한이 없는 것처럼 취급돼, 표를 선언
//! 높이에 맞추는 0.916배 축소가 그 묶음을 179.5 → 164.4px 로 눌렀다. 칸
//! 내용은 여백까지 178.0px 이라 마지막 줄이 칸 밖으로 밀려 clip 됐다.
//!
//! 수정: 병합 셀의 내용 하한을 걸친 행들에 현재 높이 비례로 나눠 싣는다 —
//! 묶음 전체가 내용 아래로 내려가지 않으면서, 여유 있는 다른 행은 부족분을
//! 계속 흡수한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6124/2737927_housing_evaluation_guideline.hwpx";
/// 결함이 나타나는 쪽(0-based).
const PAGE: u32 = 7;
/// 잘리던 마지막 줄.
const LAST_LINE: &str = "정정 필요함";

#[test]
fn issue_6124_merged_cell_keeps_its_last_line_inside_the_cell() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core
        .build_page_render_tree(PAGE)
        .expect("page 8 render tree");

    let cell = find_cell_with_text(&page.root, LAST_LINE).expect("평가방법 칸");
    let cell_bottom = cell.bbox.y + cell.bbox.height;
    let line_bottom = last_line_bottom(cell, LAST_LINE).expect("마지막 줄");

    assert!(
        line_bottom <= cell_bottom + 0.5,
        "마지막 줄이 칸 밖으로 나가 괘선에 잘린다: 줄 아래끝={line_bottom:.1}, 칸 아래끝={cell_bottom:.1}"
    );

    // 저장 lineseg 내용(168.2) + 셀 여백(3.8) = 171.9 이 이 칸의 하한이다.
    // 결함 시 164.4 로 눌렸다. 한글 2020 은 ≈177px 로 그리는데, 남는 ~5px 은
    // 줄 피치 차(19.4 vs 21.2px)라 이 축과 별개다 — 이슈에 잔존으로 남긴다.
    assert!(
        cell.bbox.height >= 171.5,
        "병합 칸이 저장 내용 아래로 눌렸다: h={:.1} (하한 171.9, 결함 시 164.4)",
        cell.bbox.height
    );
}

fn find_cell_with_text<'a>(node: &'a RenderNode, needle: &str) -> Option<&'a RenderNode> {
    if matches!(node.node_type, RenderNodeType::TableCell(_)) && contains_text(node, needle) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_cell_with_text(child, needle))
}

fn contains_text(node: &RenderNode, needle: &str) -> bool {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(needle) {
            return true;
        }
    }
    node.children
        .iter()
        .any(|child| contains_text(child, needle))
}

fn last_line_bottom(node: &RenderNode, needle: &str) -> Option<f64> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(needle) {
            return Some(node.bbox.y + node.bbox.height);
        }
    }
    node.children
        .iter()
        .filter_map(|child| last_line_bottom(child, needle))
        .fold(None, |acc: Option<f64>, bottom| {
            Some(acc.map_or(bottom, |best: f64| best.max(bottom)))
        })
}
