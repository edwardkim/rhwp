//! [Issue #5929 후속] 어울림(Square) 그림의 **빈 앵커 문단이 자기 줄을 잃어**, 뒤따르는
//! 자리차지 표가 한 줄만큼 위로 올라온다.
//!
//! `0c2f1cfe3`(#5929)가 겹침 자체는 이미 막았다 — 그 수정은 그림의 페인트 바닥을
//! 하한으로 삼아 표를 밀어낸다. 남은 것은 **간격**이다: 표가 그림 바닥에 딱 붙어
//! (간격 0.0px) 그려졌다. HwpViewer 2024(이슈 첨부 PDF)는 24.3px 를 둔다.
//!
//! 근인: `#5809` 의 "빈 host 문단도 줄을 차지한다" 증언 경로가 두 겹으로 막힌다 —
//! (1) 다음 문단이 **가시 텍스트**일 것을 요구하는데 이 문서는 다음도 빈 문단이고,
//! (2) 이 문서의 lineseg 는 전부 합성(`tag & 0x8000_0000 != 0`)이라 사다리 증언
//! 자체가 없다. 폴백은 `InFrontOfText` 만 인정해 어울림은 0 을 받았다.
//!
//! 수정: 사다리가 **아예 없는** 문서에서는 조판을 따른다 — typeset 은 이 문단을
//! `sb + lines + sa`(형제 빈 문단과 같은 54.1px)로 계상한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue5929/table_below_square_pic.hwpx";
/// 어울림 그림을 매단 빈 앵커 문단.
const HOST_PARA: usize = 8;
/// 앵커 문단의 줄 몫(sb 16.0 + lines 34.1 + sa 4.0). 이보다 좁으면 줄이 유실된 것이다.
const MIN_GAP_PX: f64 = 20.0;

#[test]
fn issue_5929_square_host_paragraph_keeps_its_line() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    let page = core.build_page_render_tree(0).expect("page 1 render tree");
    let picture_bottom = host_picture_bottom(&page.root).expect("어울림 그림");
    let table_top = first_table_top(&page.root).expect("자리차지 표");

    let gap = table_top - picture_bottom;
    assert!(
        gap >= MIN_GAP_PX,
        "어울림 그림의 빈 앵커 문단이 제 줄을 차지해야 한다 — \
         그림 아래끝 {picture_bottom:.1}, 표 위끝 {table_top:.1}, 간격 {gap:.1}px \
         (최소 {MIN_GAP_PX})"
    );
}

/// 앵커 문단이 매단 그림의 아래끝.
fn host_picture_bottom(node: &RenderNode) -> Option<f64> {
    let own = match &node.node_type {
        RenderNodeType::Image(img) if img.para_index == Some(HOST_PARA) => {
            Some(node.bbox.y + node.bbox.height)
        }
        _ => None,
    };
    node.children
        .iter()
        .filter_map(host_picture_bottom)
        .chain(own)
        .fold(None, |acc: Option<f64>, bottom| {
            Some(acc.map_or(bottom, |best: f64| best.max(bottom)))
        })
}

/// 쪽에서 가장 위에 놓인 표의 위끝.
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
