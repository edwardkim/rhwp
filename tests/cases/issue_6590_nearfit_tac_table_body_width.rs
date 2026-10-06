#![cfg(not(target_arch = "wasm32"))]

//! [#6590 -> #7059] 최상위 글자처럼(TAC) 표의 선언 폭이 본문 폭을 **근소 초과**할 때
//! 무엇을 그리는가.
//!
//! 표본 `samples/basic/BlogForm_BookReview.hwp` 는 본문 폭 35149HU(468.7px) 문서에 선언
//! 폭 35719HU(476.3px) 짜리 4행 TAC 표를 담고 있다(초과 570HU).
//!
//! `#6590` 은 host 문단의 저장 lineseg 가 본문 폭(35148HU)인 것을 "한/글이 표를 본문 폭
//! 줄박스에 실었다"는 증거로 읽고 표를 본문 폭으로 비례 축소했다. **정본이 그 읽기를
//! 반증한다** — host 줄이 본문 폭인 것은 "표가 그 줄을 오른쪽으로 넘친다"는 뜻이기도 하다.
//!
//! 정본 세 판본(`pdf/basic/BlogForm_BookReview-hwp-2020.pdf` · `-2022.pdf` ·
//! `pdf/BlogForm_BookReview-2020.pdf`)이 글자 단위로 같다.
//!
//! ```text
//!   본문 영역     22.68 .. 374.17 pt      (= 우단 498.9 px)
//!   선언 표 폭    357.19 pt (35719 HU)
//!   정본 괘선     22.51 .. 379.37 pt      (= 우단 505.8 px · 폭 356.86 pt)
//! ```
//!
//! 한/글은 이 표를 **축소하지 않고 본문 우단을 5.2 pt(약 6.9 px) 넘겨 그린다.**
//!
//! ```text
//!   표 우단   정본 505.8 px
//!             #6590 규칙   498.9 px  (-6.9)
//!             #7059 이후   506.5 px  (+0.7 — 괘선 선폭 안쪽)
//! ```
//!
//! 갈림은 **표 첫 칸의 저장 `LINE_SEG`** 다(`#7059`). 이 표본은 34696 + 실효 여백 1020 =
//! 35716 으로 **선언 35719 에 수렴**한다(축소 가정 35149 와는 567 차이). 사다리가 축소 폭을
//! 말하는 표는 종전대로 축소한다 — 판정 가능한 표 30개 중 18개가 그쪽이다.

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

/// 사다리가 선언 폭을 말하는 near-fit 표는 **축소하지 않는다** — 정본과 같다.
#[test]
fn nearfit_tac_table_keeps_declared_width_like_the_oracle() {
    let bytes = std::fs::read(SAMPLE).expect("read sample");
    let core = DocumentCore::from_bytes(&bytes).expect("parse");
    let tree = core.build_page_render_tree(0).expect("render page 1");
    let table_right = max_table_right(&tree.root);
    assert!(table_right.is_finite(), "표본 1쪽에 표 노드가 있어야 한다");

    // 정본 괘선 우단 379.37 pt = 505.83 px. 괘선 선폭(약 0.35 pt) 안쪽이라 1.0px 로 잠근다.
    let oracle_right_px = 505.83_f64;
    assert!(
        (table_right - oracle_right_px).abs() <= 1.0,
        "정본 괘선 우단은 {oracle_right_px:.2}px 다(#6590 규칙은 498.9px 로 축소했다). got {table_right:.2}px"
    );

    // 그 값은 본문 우단을 넘는다 — 한/글이 실제로 하는 일이다.
    let body_right = body_right_px(&core);
    assert!(
        table_right > body_right + 5.0,
        "정본은 표를 본문 우단({body_right:.1}px) 밖으로 6.9px 내보낸다. got {table_right:.1}px"
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
