//! [Issue #6312] TOP_AND_BOTTOM 부동 표를 단 **글이 있는** 문단의 자기 글줄
//! (lh+ls)이 통째로 사라져 다음 본문 문단이 표에 붙는다.
//!
//! #6147 은 빈 앵커(`text.is_empty()`)만 복구한다. 글이 있으면
//! `is_current_visible_para_float` 가 표 뒤 줄 높이 가산을 건너뛰어
//! 다음 문단이 표 하단과 0px 로 만난다. 저장 사다리가
//! `next.vpos - host.vpos == lh + ls` 이면 표 높이는 접힌 채 host 줄만
//! 흐름에 계상된 것이므로 그 줄만 더한다 — 표 높이를 다시 더하지 않는다.
//!
//! 픽스처는 #6147 보도자료 머리 밴드를 복제해 마지막 자리차지 표 host 의
//! 빈 `<hp:t/>` 에 글자 하나를 넣은 것이다. 사다리(29818→31574 = 1756HU)는
//! 그대로라, 빈-앵커 특례가 꺼진 뒤에도 같은 간격이 남아야 한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6312/tab_host_own_line.hwpx";
const PAGE: u32 = 0;
const FIRST_BODY_LINE: &str = "기획예산처는";
/// 앵커 줄 advance 1756HU + 마지막 표 바깥 아래 여백 283HU.
const EXPECTED_GAP_PX: f64 = 2039.0 / 7200.0 * 96.0;

#[test]
fn issue_6312_visible_tab_host_keeps_its_own_line() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    let page = core
        .build_page_render_tree(PAGE)
        .expect("page 1 render tree");
    let body_top =
        first_line_top_below(&page.root, FIRST_BODY_LINE, 0.0).expect("띠 아래 첫 본문 줄");
    let band_bottom = table_bottom_above(&page.root, body_top + 1.0).expect("본문 위 자리차지 표");

    let gap = body_top - band_bottom;
    assert!(
        (gap - EXPECTED_GAP_PX).abs() <= 1.0,
        "글이 있는 자리차지 host 의 줄 상자가 표 아래 자리를 차지해야 한다 — \
         띠 하단={band_bottom:.1}, 본문 상단={body_top:.1}, 간격={gap:.1}px \
         (기대 {EXPECTED_GAP_PX:.1}px)"
    );
}

fn table_bottom_above(node: &RenderNode, ceiling: f64) -> Option<f64> {
    let own = match &node.node_type {
        RenderNodeType::Table(_) => {
            let bottom = node.bbox.y + node.bbox.height;
            (bottom <= ceiling).then_some(bottom)
        }
        _ => None,
    };
    node.children
        .iter()
        .filter_map(|child| table_bottom_above(child, ceiling))
        .chain(own)
        .fold(None, |acc: Option<f64>, bottom| {
            Some(acc.map_or(bottom, |best: f64| best.max(bottom)))
        })
}

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
