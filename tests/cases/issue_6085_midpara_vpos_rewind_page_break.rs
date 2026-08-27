//! [Issue #6085] HWP5 저장 LINE_SEG 가 **문단 중간**에서 쪽을 가르는 형상
//! (`samples/endnote-01.hwp` pi=24: line0 `vpos=67809` → line1 `vpos=0`).
//!
//! 되감긴 꼬리를 현재 쪽에 붙이면 본문 하단(1028.0)을 넘어 꼬리말·용지 밖으로
//! 그려져 글자가 보이지 않는다. 이슈 등록 시 실측:
//!
//! | 줄 | y | 초과 |
//! |---|---|---|
//! | pi=24 line1 | 1045.8 | +17.8 |
//! | pi=25 line0 | 1079.7 | +51.7 |
//! | pi=25 line1 | 1109.5 | +81.5 |
//!
//! 현재 devel 은 이 문단을 line 1 에서 갈라 꼬리를 다음 쪽 첫머리에 둔다. 이 테스트는
//! **그 상태를 핀**한다 — 고쳐졌지만 잠금이 없어 조용히 되돌아갈 수 있는 축이고,
//! 실제로 저장 vpos 를 쪽 경계로 읽는 변경이 계속 들어온다(#5907·#5921·#6132 계열).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/endnote-01.hwp";
/// 되감김이 일어나는 문단.
const SPLIT_PARA: usize = 24;
/// 되감긴 꼬리가 가야 할 쪽(0-based).
const TAIL_PAGE: u32 = 2;
/// 되감김 직전 쪽(0-based).
const HEAD_PAGE: u32 = 1;
/// 본문 하단(px). 이 아래로 그려지면 꼬리말·용지 밖이다.
const BODY_BOTTOM_PX: f64 = 1028.0;

#[test]
fn issue_6085_midparagraph_rewind_starts_a_new_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    // 되감김 직전 쪽: 이 문단의 줄이 본문 안에서 끝나야 한다.
    let head = core.build_page_render_tree(HEAD_PAGE).expect("page 2");
    let head_lines = para_line_tops(&head.root);
    assert!(
        !head_lines.is_empty(),
        "되감김 직전 쪽에 이 문단의 줄이 있어야 한다"
    );
    let lowest = head_lines.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    assert!(
        lowest < BODY_BOTTOM_PX,
        "되감긴 꼬리를 현재 쪽에 붙이면 본문 밖으로 나간다 — 가장 아래 줄 {lowest:.1} \
         (본문 하단 {BODY_BOTTOM_PX})"
    );

    // 다음 쪽: 꼬리가 첫머리에 와야 한다.
    let tail = core.build_page_render_tree(TAIL_PAGE).expect("page 3");
    let tail_lines = para_line_tops(&tail.root);
    assert!(
        !tail_lines.is_empty(),
        "되감긴 꼬리는 다음 쪽 첫머리에 놓여야 한다"
    );
    let highest = tail_lines.iter().cloned().fold(f64::INFINITY, f64::min);
    assert!(
        highest < 200.0,
        "꼬리가 다음 쪽 첫머리에 와야 한다 — 실측 {highest:.1}"
    );
}

/// `SPLIT_PARA` 에 속한 텍스트 줄들의 위끝.
fn para_line_tops(node: &RenderNode) -> Vec<f64> {
    let mut out = Vec::new();
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if line.para_index == Some(SPLIT_PARA) {
            out.push(node.bbox.y);
        }
    }
    for child in &node.children {
        out.extend(para_line_tops(child));
    }
    out
}
