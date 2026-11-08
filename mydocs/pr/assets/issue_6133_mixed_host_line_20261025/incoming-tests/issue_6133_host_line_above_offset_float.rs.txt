//! [Issue #6133] 문단 기준 +47px 오프셋을 가진 자리차지 포스터 그림이 **자기 호스트
//! 문단의 글줄**("붙임2 행사 포스터" TAC 제목 표)까지 그림 아래로 밀어내, 제목이
//! 용지 밖(y=1089.1)에 그려진다 (156483831 4쪽).
//!
//! 근인: `layout_table_item` 의 TAC 재등록 분기는 "앞선 TAC 가 흐름을 진행시켰으면
//! 글줄을 그 자리로 옮긴다"는 규칙인데, 여기서 흐름을 진행시킨 것은 TAC 가 아니라
//! **양수 오프셋을 가진 자리차지 그림**이다. 오프셋(3518HU=46.9px)이 글줄 높이
//! (2831HU=37.7px)보다 커서 한글은 글줄을 그림 **위**(문단 상단)에 그린다.
//!
//! 한글 2024 PDF 실측(원본 4쪽): 제목 표 상자 83.1~120.8, 포스터 그림 129.9~1038.2.
//!
//! 재현물은 원본의 문단 43 하나만 남긴 IR 슬라이스(12KB)이고 이미지는 1×1 PNG 로
//! 갈아 끼웠다 — 위치·크기는 컨트롤 속성이 결정하므로 화소는 무관하다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6133/156483831_poster_title_above_offset_float.hwp";
/// 한글이 그리는 제목 표 상자 상단(= 본문 상단).
const EXPECTED_TITLE_TOP_PX: f64 = 83.1;

#[test]
fn issue_6133_host_line_stays_above_the_offset_float() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    let page = core.build_page_render_tree(0).expect("page 1 render tree");
    let title_top = first_table_top(&page.root).expect("붙임2 제목 표");
    let picture_top = images(&page.root)
        .into_iter()
        .map(|(top, _)| top)
        .fold(f64::INFINITY, f64::min);
    assert!(picture_top.is_finite(), "포스터 그림을 찾지 못했다");

    assert!(
        title_top < picture_top,
        "오프셋이 글줄 높이보다 크면 제목 글줄은 그림 위에 놓여야 한다 — \
         제목 위끝={title_top:.1}, 그림 위끝={picture_top:.1}"
    );
    assert!(
        (title_top - EXPECTED_TITLE_TOP_PX).abs() <= 1.0,
        "제목 표는 본문 상단({EXPECTED_TITLE_TOP_PX:.1})에 놓여야 한다 — 실측 {title_top:.1}"
    );
}

/// 페이지에서 가장 위에 놓인 표의 위끝.
fn first_table_top(node: &RenderNode) -> Option<f64> {
    let own = matches!(node.node_type, RenderNodeType::Table(_)).then_some(node.bbox.y);
    node.children
        .iter()
        .filter_map(first_table_top)
        .chain(own)
        .fold(None, |acc: Option<f64>, top| {
            Some(acc.map_or(top, |best: f64| best.min(top)))
        })
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
