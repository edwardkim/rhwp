#![cfg(not(target_arch = "wasm32"))]

//! [#6590] 최상위 글자처럼(TAC) 표의 선언 폭이 본문 폭을 **근소 초과**할 때, 표를 본문 폭으로
//! 비례 축소해 그리는 계약의 회귀 시험.
//!
//! 표본 `samples/basic/BlogForm_BookReview.hwp` 는 본문 폭 35149HU(468.7px) 문서에
//! 선언 폭 35719HU(476.3px) 짜리 4행 TAC 표를 담고 있다(초과 570HU). host 문단의
//! 저장 lineseg `segment_width` 는 35148HU 로 본문 폭과 같다 — 한/글이 이 표를 본문
//! 폭 줄박스에 실었다는 직접 증거다.
//!
//! 종전에는 선언 폭을 그대로 써서 표 우단이 본문 우단을 7.6px 넘었다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/basic/BlogForm_BookReview.hwp";

/// 본문 폭(HWPUNIT) — 용지 폭에서 좌·우 여백을 뺀 값.
fn body_right_px(core: &DocumentCore) -> f64 {
    let page_def = &core.document().sections[0].section_def.page_def;
    rhwp::renderer::hwpunit_to_px(
        (page_def.width - page_def.margin_right) as i32,
        rhwp::renderer::DEFAULT_DPI,
    )
}

fn max_table_right(node: &RenderNode) -> f64 {
    let mut best = f64::NEG_INFINITY;
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        best = node.bbox.x + node.bbox.width;
    }
    for child in &node.children {
        best = best.max(max_table_right(child));
    }
    best
}

#[test]
fn nearfit_tac_table_is_scaled_into_body_width() {
    let bytes = std::fs::read(SAMPLE).expect("read sample");
    let core = DocumentCore::from_bytes(&bytes).expect("parse");
    let tree = core.build_page_render_tree(0).expect("render page 1");
    let table_right = max_table_right(&tree.root);
    assert!(table_right.is_finite(), "표본 1쪽에 표 노드가 있어야 한다");
    let body_right = body_right_px(&core);
    assert!(
        table_right <= body_right + 0.5,
        "본문 폭을 근소 초과하는 TAC 표는 본문 폭으로 축소돼야 한다 \
         (표 우단 {table_right:.1}px > 본문 우단 {body_right:.1}px)"
    );
}

/// 축소는 near-fit 에만 적용된다 — 표가 본문 폭 안에 있으면 폭을 건드리지 않는다.
#[test]
fn table_within_body_width_keeps_declared_width() {
    let bytes = std::fs::read(SAMPLE).expect("read sample");
    let core = DocumentCore::from_bytes(&bytes).expect("parse");
    let page_def = &core.document().sections[0].section_def.page_def;
    let body_width = page_def.width - page_def.margin_left - page_def.margin_right;
    let tree = core.build_page_render_tree(0).expect("render page 1");
    let table_right = max_table_right(&tree.root);
    let body_width_px =
        rhwp::renderer::hwpunit_to_px(body_width as i32, rhwp::renderer::DEFAULT_DPI);
    // 축소 폭이 본문 폭에 맞춰졌는지(과소 축소가 아닌지) 확인한다.
    let left =
        rhwp::renderer::hwpunit_to_px(page_def.margin_left as i32, rhwp::renderer::DEFAULT_DPI);
    assert!(
        table_right - left >= body_width_px - 1.0,
        "축소는 본문 폭까지만 — 표 폭 {:.1}px 이 본문 폭 {body_width_px:.1}px 보다 작다",
        table_right - left
    );
}
