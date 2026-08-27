//! [Issue #6146] 문단 끝 로고 글상자가 2쪽 상단으로 넘어가 「참고」 제목 표를 덮는다
//! (156583583).
//!
//! 근인: 문단 24 의 저장 lineseg 가 `vpos=0`(쪽 리셋)이라 rhwp 는 문단 전체를 다음
//! 쪽으로 넘긴다. 그 리셋은 문단의 **줄**이 다음 쪽이라는 신호지 자리차지 밴드까지
//! 옮기라는 신호가 아니다 — 한글은 밴드를 떠나는 쪽의 흐름 말미에 남기고 본문 아래
//! 여백으로 흘린다(한글 2024 PDF 1쪽 y 1029.4..1079.5, 본문 하단 1039.3).
//!
//! 수정: 저장 리셋 경계에서 비-TAC 자리차지(TopAndBottom, vert=문단) 개체를 떠나는
//! 쪽에 남긴다. 판별은 물리적으로 — 개체 아래끝이 용지 안에 남을 때만.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6146/156583583_press_release_logo_band.hwpx";
/// 1쪽 본문 하단(px) — 밴드는 이 아래 여백으로 흘러야 한다.
const BODY_BOTTOM_PX: f64 = 1039.3;
/// 2쪽 상단의 「참고」 제목 표가 놓이는 띠. 밴드가 여기 오면 제목이 가려진다.
const PAGE2_TITLE_BAND_PX: f64 = 200.0;

#[test]
fn issue_6146_page_tail_float_band_stays_on_the_leaving_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    assert_eq!(core.page_count(), 2, "한글 오라클과 같은 2쪽이어야 한다");

    let page1 = core.build_page_render_tree(0).expect("page 1 render tree");
    let page2 = core.build_page_render_tree(1).expect("page 2 render tree");

    let spilled = images(&page1.root)
        .into_iter()
        .filter(|(_, bottom)| *bottom > BODY_BOTTOM_PX)
        .count();
    assert!(
        spilled > 0,
        "로고 밴드가 1쪽 흐름 말미(본문 하단 {BODY_BOTTOM_PX} 아래 여백)에 남아야 한다 — \
         1쪽 이미지: {:?}",
        images(&page1.root)
    );

    let overlapping: Vec<_> = images(&page2.root)
        .into_iter()
        .filter(|(top, _)| *top < PAGE2_TITLE_BAND_PX)
        .collect();
    assert!(
        overlapping.is_empty(),
        "2쪽 상단 제목 표 자리에 로고 밴드가 겹쳤다: {overlapping:?}"
    );
}

/// 페이지의 모든 이미지 노드 (위끝, 아래끝).
fn images(node: &RenderNode) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    if matches!(node.node_type, RenderNodeType::Image(_)) {
        out.push((node.bbox.y, node.bbox.y + node.bbox.height));
    }
    for child in &node.children {
        out.extend(images(child));
    }
    out
}
