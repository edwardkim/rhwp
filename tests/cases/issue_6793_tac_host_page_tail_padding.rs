//! [Issue #6793] TAC 표 host 줄의 **꼬리 간격**이 쪽 규모인데 그대로 흐름에 실려,
//! 다음 문단이 492.8px 아래·**용지 밖 363.9px** 로 나갔다.
//!
//! `1611000-201000141` 1쪽 — 표지 상자(1×1 TAC 표, 선언 817.6px)의 저장 사다리:
//!
//! ```text
//!   문단 0.0  ls[0] vpos=0     lh=61600  th=1000     ← 앵커 줄
//!             ls[1] vpos=1600  lh=61600  ls=36960    ← 표 줄 + 꼬리 간격 492.8px
//!   문단 0.1  ls[0] vpos=0                           ← **새 쪽 상단**
//! ```
//!
//! 사다리는 `pi=1` 을 **새 쪽 상단**에 두었으므로 `ls=36960` 은 흐름 간격이 아니라
//! **쪽 끝까지 채우는 패딩**이다. 흐름에 태우면 그만큼 다음 문단이 밀린다.
//!
//! ```text
//!   VPOS_CORR pi=1 prev_lh=61600 prev_ls=36960 y_in=836.80 end_y=1329.60 applied=false
//!   페인트    표 바닥 965.1  →  「< 차 례 >」 y=1459.7   (용지 1122.5 밖 363.9px)
//! ```
//!
//! ⚠ **글자 수 대조는 점 리더(`···`)를 빼고 재야 한다.** 그대로 재면 결손 1,468자로
//! 나오는데 그 대부분이 차례의 탭 채움 점이다(rhwp PDF 는 안 싣는다). 점·공백을 빼면
//! 수정 전 **4자**, 수정 후 **0자**다.
//!
//! ⚠ **남는 축** — 「< 차 례 >」가 rhwp 는 1쪽 끝, 한/글은 2쪽 첫 줄이다(1쪽 글자
//! 37 vs 33). 사다리 `vpos == 0` 을 쪽 경계로 승격하는 별개 축이라 여기서는
//! **용지 밖 이탈이 없을 것**만 계약한다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6793/1611000-201000141-small-air-transport-study.hwp";

/// 정식 fixture는 `MANIFEST.json`의 SHA-256로 고정된다. fixture 부재는 회귀 시험의
/// 성공 조건이 아니므로 읽기 실패를 즉시 드러낸다.
fn sample() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
        .expect("#6793 정식 HWP fixture 읽기")
}

fn worst_text_below(node: &RenderNode, bottom: f64, out: &mut f64) {
    if matches!(node.node_type, RenderNodeType::TextRun(_)) {
        *out = out.max(node.bbox.y + node.bbox.height - bottom);
    }
    for child in &node.children {
        worst_text_below(child, bottom, out);
    }
}

/// 1쪽 글자가 **용지** 밖으로 나가면 안 된다.
///
/// 수정 전 「< 차 례 >」가 `y=1459.7..1486.4` — 용지(1122.5px) 밖 `+363.9px`.
#[test]
fn tac_host_page_tail_padding_keeps_text_on_paper() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(core.page_count(), 12, "한/글 2024 와 같은 12쪽이어야 한다");

    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let paper_bottom = tree.root.bbox.y + tree.root.bbox.height;

    let mut over = 0.0f64;
    worst_text_below(&tree.root, paper_bottom, &mut over);

    assert!(
        over <= 0.5,
        "1쪽 글자가 용지 밖으로 나가면 안 된다 — #6793 회귀          \
         (초과 {over:.1}px, 용지 하한 {paper_bottom:.1}; 수정 전 +363.9px)"
    );
}

/// 표지 상자 다음 문단이 표 바닥 **바로 아래**에 와야 한다.
///
/// 수정 전에는 꼬리 간격 492.8px 을 흐름에 태워 494.6px 아래에 놓였다.
#[test]
fn next_paragraph_follows_the_cover_table_closely() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");

    fn table_bottom(node: &RenderNode, out: &mut f64) {
        if matches!(node.node_type, RenderNodeType::Table { .. }) {
            *out = out.max(node.bbox.y + node.bbox.height);
        }
        for child in &node.children {
            table_bottom(child, out);
        }
    }
    fn lowest_text_top(node: &RenderNode, above: f64, out: &mut f64) {
        if matches!(node.node_type, RenderNodeType::TextRun(_)) && node.bbox.y > above {
            *out = out.min(node.bbox.y);
        }
        for child in &node.children {
            lowest_text_top(child, above, out);
        }
    }
    let mut bottom = 0.0f64;
    table_bottom(&tree.root, &mut bottom);
    assert!(bottom > 0.0, "표지 상자를 찾아야 한다");

    let mut top = f64::MAX;
    lowest_text_top(&tree.root, bottom, &mut top);
    assert!(top < f64::MAX, "표 아래 글줄이 있어야 한다");

    let gap = top - bottom;
    assert!(
        gap < 100.0,
        "표 바닥과 다음 글줄 사이가 쪽 규모로 벌어지면 안 된다 — #6793 회귀          \
         (간격 {gap:.1}px, 표 바닥 {bottom:.1}; 수정 전 494.6px)"
    );
}
