//! Issue #6524: 어울림 개체를 안은 문단의 저장 줄이 **좌·우 조각 2개**라고 해서 저장
//! 사다리 질의가 물러나면 안 된다 — 물러나면 그 문단의 진행량이 0 이 되어 뒤따르는 본문
//! 전체가 한 줄만큼 위로 올라온다.
//!
//! `30098` 3쪽 `pi=36` 은 도표(gso 2개)를 안은 빈 본문 문단이고, 저장 `LINE_SEG` 는 도표를
//! 피해 좌·우 띠로 쪼개진 조각 둘로 남는다.
//!
//! ```text
//! pi=36  ls[0] vpos=21722 lh=1500 ls=900 cs=0      sw=3402   tag=0x00020000 (FIRST_SEGMENT)
//!        ls[1] vpos=21722 lh=1500 ls=900 cs=45305  sw=2883   tag=0x00050000 (EMPTY|LAST)
//! ```
//!
//! 종전 술어(`[seg]` — 조각이 정확히 하나)는 이 문단에서 통째로 물러났고, `pi=37` 이
//! `pi=36` 의 자리로 올라와 본문 전체가 **15.00pt** 상승했다. 그 결과 `추진경과`(pi=55)
//! 제목이 도표 테두리 하단(651.74pt) 안쪽 645.73pt 에 놓여 **6.01pt 겹쳤다**
//! (한/글 2018: 659.95pt, 여유 8.70pt).
//!
//! 같은 `vertical_pos` 를 공유하면서 `column_start` 가 갈라지는 조각은 **한 줄**이다
//! (#6299 술어). 그것을 한 줄로 받으면 저장 델타 그대로 24.00pt 를 전진한다.
//!
//! 쪽수는 15/15 로 한/글과 같고 글자도 전부 트리에 남으므로, 쪽수·텍스트·넘침 지표가
//! 전부 침묵한다. 그래서 좌표를 직접 고정한다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue6524/30098_float_host_split_lineseg.hwp";
const PAGE_INDEX: u32 = 2; // 0-based — `추진배경 및 경과` 가 있는 3쪽

/// 저장 사다리의 `pi=35`(vpos=20122) → `pi=55`(vpos=59282) 간격.
///
/// `39160 HU = 391.60pt = 522.13px @96dpi`. 이 문서에서 두 문단 사이에는 도표 옆을 흐르는
/// 빈 문단(pi=36..54)만 있으므로, 이 간격이 곧 "그 문단들이 저장대로 전진했는가" 다.
/// 회귀 시에는 `pi=36` 이 통째로 빠져 **502.13px**(-20.0px = -15.00pt) 이었다.
const STORED_GAP_PX: f64 = 522.133;
const TOLERANCE_PX: f64 = 2.0;

const PARA_ANCHOR: usize = 35; // `(YTN '10.9.1일자, 중앙일보 ’11…` — 도표 위 마지막 본문 줄
const PARA_TARGET: usize = 55; // `추진경과` 제목

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn text_run_top(nodes: &[&RenderNode], para_index: usize) -> Option<f64> {
    nodes
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run) if run.para_index == Some(para_index) => Some(node.bbox.y),
            _ => None,
        })
        .fold(None, |acc: Option<f64>, y| {
            Some(acc.map_or(y, |cur| cur.min(y)))
        })
}

#[test]
fn issue_6524_split_lineseg_host_advances_one_line() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let document = HwpDocument::from_bytes(&bytes).expect("parse issue6524 sample");
    let tree = document
        .build_page_render_tree(PAGE_INDEX)
        .expect("render p3");

    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);

    let anchor = text_run_top(&nodes, PARA_ANCHOR)
        .unwrap_or_else(|| panic!("3쪽에 pi={PARA_ANCHOR} 의 글이 있어야 한다"));
    let target = text_run_top(&nodes, PARA_TARGET)
        .unwrap_or_else(|| panic!("3쪽에 pi={PARA_TARGET}(`추진경과`) 이 있어야 한다"));

    let gap = target - anchor;
    assert!(
        (gap - STORED_GAP_PX).abs() <= TOLERANCE_PX,
        "도표 옆 빈 문단들의 진행량이 저장 사다리와 다르다 — pi={PARA_ANCHOR}→pi={PARA_TARGET} \
         실측 {gap:.2}px, 저장 {STORED_GAP_PX:.2}px (회귀 시 502.13px = 한 줄 24.00pt 부족)"
    );
}
