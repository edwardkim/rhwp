//! [Issue #4068] **줄보다 큰 인라인 표가 위 바깥여백을 잃는다.**
//!
//! `paragraph_layout.rs` 의 TAC 표 배치는 표 하단을 베이스라인에 맞춘다.
//!
//! ```text
//!   tbl_y = tac_table_stored_outer_band_top(..)                       // = 줄상단 + om_top
//!           .unwrap_or(|| (current_y + baseline_dist + om_bottom - tbl_h).max(current_y))
//! ```
//!
//! 앞 갈래는 저장 줄높이가 바깥상자(`om_top + 선언높이 + om_bottom`)를 담았다고
//! **증명될 때만**(`#5729`) 쓴다. 표가 줄보다 크면 그 증명이 원리적으로 불가능하고,
//! 뒷식은 줄 상단 **위로** 올라가 `.max(current_y)` 에 걸린다 — 그러면서 선언된
//! `outer_margin_top` 까지 같이 버려졌다.
//!
//! ```text
//!   간장 기증자 보고서 33쪽 · 5행×11열 · om_top=140HU(1.87px)
//!     cur_y 83.16 · baseline 182.31 · om_bottom 1.87 · tbl_h 210.75
//!     raw = 56.59  <  cur_y      → 종전 83.16
//!     한/글 2020 정본 괘선        → 85.03      (= cur_y + om_top, 정확)
//! ```
//!
//! 클램프가 걸린다는 것은 "이 줄의 baseline 모델이 성립하지 않는다"는 뜻이고, 그때
//! 한/글의 답은 줄 상단 + 선언 위 여백이다. 임계값은 쓰지 않는다 — 문서가 준
//! `outer_margin_top` 과 기존 클램프 조건만 본다.
//!
//! 아래 셋이 **양쪽을 잠근다** — 여백을 못 되찾으면 ①이, 클램프가 안 걸리는 표까지
//! 밀면 ②·③이 깨진다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str =
    "samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp";

fn core() -> DocumentCore {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    DocumentCore::from_bytes(&bytes).expect("parse fixture")
}

/// 그 쪽에서 가장 위에 있는 최상위 표의 상단 y.
fn top_table_y(root: &RenderNode) -> f64 {
    fn walk(node: &RenderNode, best: &mut f64) {
        if matches!(node.node_type, RenderNodeType::Table(_)) {
            *best = best.min(node.bbox.y);
        }
        for child in &node.children {
            walk(child, best);
        }
    }
    let mut best = f64::INFINITY;
    walk(root, &mut best);
    assert!(best.is_finite(), "이 쪽에 표가 없다 — 시험 전제가 깨졌다");
    best
}

fn top_table_y_on(page_index: u32) -> f64 {
    let core = core();
    let tree = core
        .build_page_render_tree(page_index)
        .unwrap_or_else(|e| panic!("p{} render: {e}", page_index + 1));
    top_table_y(&tree.root)
}

/// ① 클램프가 걸리는 표는 줄 상단 + `om_top` 에 선다.
///
/// 한/글 2020 정본 괘선 85.03. 종전 83.20(= 줄 상단, 여백 소실).
#[test]
fn a_clamped_inline_table_keeps_its_declared_outer_top_margin() {
    let y = top_table_y_on(32);
    assert!(
        (84.6..=85.4).contains(&y),
        "33쪽 표 상단이 정본(85.03)에서 벗어났다 — 실측 {y:.2}px. \
         종전 결함값은 83.20(= om_top 140HU 소실)이다."
    );
}

/// ② **반례** — 저장 밴드가 바깥상자를 증명하는 표는 움직이지 않는다.
///
/// 35쪽 표는 `lh == om_top + 선언높이 + om_bottom` 이라 첫 갈래(`#5729`)로 가고,
/// 이 변경은 그 경로를 건드리지 않는다. 정본 괘선 85.03.
#[test]
fn the_band_proven_table_is_untouched() {
    let y = top_table_y_on(34);
    assert!(
        (84.6..=85.4).contains(&y),
        "35쪽(밴드 증명 경로) 표가 움직였다 — 실측 {y:.2}px"
    );
}

/// ③ **반례** — 클램프가 걸리지 않는 표도 움직이지 않는다.
///
/// 34쪽 표는 베이스라인-하단 식이 줄 상단 아래에 있어 클램프가 발동하지 않는다.
/// 정본 괘선 271.38.
#[test]
fn the_unclamped_baseline_table_is_untouched() {
    let y = top_table_y_on(33);
    assert!(
        (271.2..=272.2).contains(&y),
        "34쪽(클램프 미발동) 표가 움직였다 — 실측 {y:.2}px"
    );
}
