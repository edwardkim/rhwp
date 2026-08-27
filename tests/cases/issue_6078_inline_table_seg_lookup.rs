//! [Issue #6078] HWP3 TAC 표 뒤의 용지 규격 줄
//! `210㎜×297㎜(신문용지 54g/㎡(재활용품))` 이 용지 밖(y=1863.6, 용지 1122.5)에
//! 그려져 소실된다.
//!
//! 근인: 인라인 표 문단 레이아웃이 `line_segs[0] = 표 줄`, `[1] = 텍스트 줄` 을
//! **가정**한다. HWP3 국세청 납세담보 확인서는 반대로 저장한다 —
//! `ls[0] lh=1300`(제목 텍스트 줄), `ls[1] lh=67616`(표 줄). 그래서 `￼` 자리표시
//! 조각이 **표 줄의 baseline**(57473HU=766.3px)을 텍스트 줄 높이로 받아 문단 바닥을
//! 표 높이만큼 한 번 더 밀었다.
//!
//! 수정: 표가 실제로 속한 seg 를 `control_line_seg_index` 로 **조회**하고, 텍스트 줄
//! 메트릭은 그 seg 가 아닌 seg 에서 가져온다.
//!
//! 저장 사다리가 정답을 말한다 — 뒤 문단 `vpos=68916HU(918.9px)`, 본문 상단 75.6
//! → **994.5px**. (한글 서식은 이 줄을 표 아래 본문 하단 밴드에 그린다.)
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/hwp3-table-caption.hwp";
/// 용지 세로 크기(px) — 이 아래로 나가면 소실이다.
const PAGE_BOTTOM_PX: f64 = 1122.5;
/// 저장 사다리가 지시하는 자리: 본문 상단 75.6 + vpos 68916HU(918.9px).
const EXPECTED_TOP_PX: f64 = 994.5;
/// 용지 규격 줄에만 있는 글자.
const PAPER_SPEC: &str = "신문용지";

#[test]
fn issue_6078_paper_spec_line_stays_inside_the_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    let page = core.build_page_render_tree(0).expect("page 1 render tree");
    let top = run_top(&page.root, PAPER_SPEC).expect("용지 규격 줄");

    assert!(
        top < PAGE_BOTTOM_PX,
        "용지 규격 줄이 용지 밖으로 나갔다 — 위끝 {top:.1} (용지 {PAGE_BOTTOM_PX})"
    );
    assert!(
        (top - EXPECTED_TOP_PX).abs() <= 1.0,
        "용지 규격 줄은 저장 사다리 자리({EXPECTED_TOP_PX:.1})에 와야 한다 — 실측 {top:.1}"
    );
}

/// `needle` 을 포함한 첫 텍스트 런의 위끝.
fn run_top(node: &RenderNode, needle: &str) -> Option<f64> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(needle) {
            return Some(node.bbox.y);
        }
    }
    node.children
        .iter()
        .find_map(|child| run_top(child, needle))
}
