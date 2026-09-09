//! [Issue #6598] 문단 기준 자리차지 표의 세로 오프셋 기준점이 앵커 문단이 아니라
//! **칼럼 상단**이어서, 테두리 안 양식 전체가 위로 뜨던 결함의 가드.
//!
//! `VertRelTo::Para` 표는 `compute_table_y_position` 에서
//! `anchor_y = para_y.unwrap_or(y_start)` 를 기준으로 `v_offset` 을 얹는다. 그런데
//! `para_y` 가 **칼럼 상단**으로 들어오는 경로가 있다 — `2744465` 1쪽은 표가 문단 1 의
//! 컨트롤인데 `para_index=1, y_offset = para_y = 108.5 = col_area.y` 였다(프로브 실측).
//! 그러면 앵커 문단 진행량만큼 통째로 위로 뜬다.
//!
//! 실측 (px @96dpi, 오라클 한/글 2020 `SaveAs PDF`):
//!
//! ```text
//! 표 y                    139.9 → 171.6   (한/글 ≈171.5)
//! `○○시‧도경찰청`(표 첫 줄)  170.6 → 202.4   (한/글 202.18)
//! `9. 비고`                652.2 → 684.0   (한/글 683.25)
//! `(영문 기관명)`(맨 아래)  883.8 → 915.6   (한/글 914.52)
//! 테두리 그림(프레임 밖)     130.5 그대로   (한/글 130.42)
//! ```
//!
//! 저장 사다리가 그 문단의 흐름 상단을 정확히 적는다(`vertpos=2240HU=29.87px`).
//! 그것을 기준점으로 쓰면 `108.5 + 29.87 + v_offset 31.41 + om_top 1.88 = 171.66`.
//!
//! ⚠ 기준점과 `om_top` 은 **같이** 움직여야 한다 — 하나만 켜면 1.9px 어긋난다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6598/2744465_fingerprint_appraisal.hwp";

/// 앵커 문단(`[별지 제5의2호]`)의 진행량 = 29.87px, 표의 `vertOffset` = 31.41px,
/// 바깥여백 위 = 1.88px. 한/글 실측 표 상단 ≈ 171.5.
const ANCHOR_ADVANCE_PX: f64 = 29.87;
const V_OFFSET_PX: f64 = 31.41;

fn find_table_y(node: &RenderNode) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        return Some(node.bbox.y);
    }
    node.children.iter().find_map(find_table_y)
}

fn first_text_y(node: &RenderNode) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        return Some(node.bbox.y);
    }
    node.children.iter().find_map(first_text_y)
}

#[test]
fn para_relative_float_table_offsets_from_its_own_stored_flow_top() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let page = core.build_page_render_tree(0).expect("render tree");

    let table_y = find_table_y(&page.root).expect("자리차지 표 노드");
    let anchor_y = first_text_y(&page.root).expect("앵커 문단 글줄");

    // 종전: 칼럼 상단 기준이라 앵커 문단 진행량이 통째로 빠졌다(139.9).
    let legacy = anchor_y + V_OFFSET_PX;
    // 기대: 앵커 문단 흐름 상단 + vertOffset + 바깥여백 위 (한/글 ≈171.5).
    let expected = anchor_y + ANCHOR_ADVANCE_PX + V_OFFSET_PX + 1.88;

    assert!(
        (table_y - expected).abs() <= 1.0,
        "문단 기준 자리차지 표는 **앵커 문단 흐름 상단**에서 오프셋을 재야 한다 —          #6598 회귀. table_y={table_y:.1} expected={expected:.1} anchor_y={anchor_y:.1}          (종전 칼럼 상단 기준이면 {legacy:.1})"
    );
    // 결함의 크기 = 앵커 문단 진행량. 그만큼 내려와 있어야 한다.
    assert!(
        table_y > legacy + ANCHOR_ADVANCE_PX - 1.0,
        "표가 여전히 칼럼 상단 기준이다 — #6598 회귀.          table_y={table_y:.1} legacy={legacy:.1}"
    );
}
