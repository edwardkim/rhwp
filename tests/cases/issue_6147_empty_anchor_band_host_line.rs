//! [Issue #6147] 자리차지(TopAndBottom) 개체를 매단 **빈 앵커 문단**이 자기 줄
//! 상자를 잃어, 개체 띠 바로 아래 첫 본문 문단이 개체에 딱 붙어 그려진다
//! (보도자료 서식 156741101 1쪽: rhwp 간격 0.0px, 한글 27.5px).
//!
//! 근인: #1147 이 "빈 앵커 vpos 가 이미 갭을 인코딩한다"는 전제로 앵커 줄
//! 간격을 일괄 억제한다. 그 전제가 성립하는지는 저장 사다리가 가른다 —
//! 이 문서의 `문단1.vpos - 문단0.vpos`(31574 - 29818 = 1756HU)는 표 3개 높이를
//! 접고 앵커 줄 advance(`lh 1300 + ls 456`)와 정확히 같다. 즉 한글은 개체 아래에
//! 앵커 줄을 실제로 차지했고, 억제하면 그 줄만큼 본문이 위로 붙는다.
//!
//! 수정: `stored_empty_anchor_band_host_line_advance_hu` — #2439 가 native
//! HWP5·단일 표·양수 offset 으로 갖고 있던 같은 사다리 증거를 HWPX 저장
//! 레이아웃과 다중 자리차지 개체로 넓힌다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6147/156741101_press_release_band.hwpx";
/// 자리차지 표 3개가 얹힌 쪽(0-based).
const PAGE: u32 = 0;
/// 표 3개를 모두 매단 빈 앵커 문단.
const ANCHOR_PARA: usize = 0;
/// 띠 바로 아래 첫 본문 문단(문단 1).
const FIRST_BODY_LINE: &str = "기획예산처는";
/// 앵커 줄 advance 1756HU + 마지막 표 바깥 아래 여백 283HU = 2039HU.
const EXPECTED_GAP_PX: f64 = 2039.0 / 7200.0 * 96.0;

#[test]
fn issue_6147_empty_anchor_band_keeps_its_host_line() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    let page = core
        .build_page_render_tree(PAGE)
        .expect("page 1 render tree");
    let band_bottom = anchor_band_bottom(&page.root).expect("빈 앵커 문단의 자리차지 표 띠");
    let body_top = first_line_top_below(&page.root, FIRST_BODY_LINE, band_bottom - 1.0)
        .expect("띠 아래 첫 본문 줄");

    let gap = body_top - band_bottom;
    assert!(
        (gap - EXPECTED_GAP_PX).abs() <= 1.0,
        "빈 앵커 문단의 줄 상자가 개체 아래 자리를 차지해야 한다 — \
         띠 하단={band_bottom:.1}, 본문 상단={body_top:.1}, 간격={gap:.1}px \
         (기대 {EXPECTED_GAP_PX:.1}px)"
    );
}

/// 앵커 문단이 매단 자리차지 표들의 가장 아래끝.
fn anchor_band_bottom(node: &RenderNode) -> Option<f64> {
    let own = match &node.node_type {
        RenderNodeType::Table(table) if table.para_index == Some(ANCHOR_PARA) => {
            Some(node.bbox.y + node.bbox.height)
        }
        _ => None,
    };
    node.children
        .iter()
        .filter_map(anchor_band_bottom)
        .chain(own)
        .fold(None, |acc: Option<f64>, bottom| {
            Some(acc.map_or(bottom, |best: f64| best.max(bottom)))
        })
}

/// `floor` 아래에서 `needle` 을 포함한 텍스트 런 중 가장 위의 위끝.
fn first_line_top_below(node: &RenderNode, needle: &str, floor: f64) -> Option<f64> {
    let own = match &node.node_type {
        RenderNodeType::TextRun(run) if run.text.contains(needle) && node.bbox.y >= floor => {
            Some(node.bbox.y)
        }
        _ => None,
    };
    node.children
        .iter()
        .filter_map(|child| first_line_top_below(child, needle, floor))
        .chain(own)
        .fold(None, |acc: Option<f64>, top| {
            Some(acc.map_or(top, |best: f64| best.min(top)))
        })
}
