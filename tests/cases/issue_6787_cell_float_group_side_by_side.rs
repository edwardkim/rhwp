//! [Issue #6787] 칸 안 한 문단의 문단-기준 자리차지 중첩 표 2개가 **나란히** 대신
//! **세로로** 쌓여, 그 칸이 3.4배로 부풀고 표 전체가 용지를 넘어 **361자가 잘렸다.**
//!
//! `16774617` 1쪽 후보자 카드 2장은 가로 위치가 문서에 실려 있다.
//!
//! ```text
//!   카드 A  tac=false wrap=TopAndBottom vert=Para(-20484) horz=Para( 1547)  w=18991
//!   카드 B  tac=false wrap=TopAndBottom vert=Para(-20579) horz=Para(23743)  w=18991
//! ```
//!
//! 두 가로 구간 `1547..20538` 과 `23743..42734` 는 **겹치지 않는다.** 한/글 2024 PDF 도
//! 두 카드를 같은 줄에 놓는다(카드 상자 x `122.7..376.8` / `418.5..672.4`,
//! y 둘 다 `311.0`). rhwp 는 오프셋을 버리고 둘 다 가운데 정렬한 뒤 `para_y` 를 표
//! 높이만큼 전진시켜 세로로 쌓았다.
//!
//! ⭐⭐ **측정과 페인트를 함께 고쳐야 한다**(`#6754` 와 같은 교훈).
//!
//! 1. `height_measurer` — 나란히 무리의 칸 높이는 **합이 아니라 최댓값**.
//!    (합산이면 칸이 선언 251.97px 대비 854.7px)
//! 2. `table_layout` 칸 경로 — 두 번째 표부터 **같은 y**, x 는 문서의 `horzOffset`.
//!    ⚠ `compute_table_x_position` 이 override 에 non-TAC `horzOffset` 을 **스스로
//!    더한다** — 오프셋까지 실으면 두 번 실린다(카드 A 119.2 → 139.8).
//!
//! 결과: 칸 `305.7..619.9`(한/글 `305.4..619.2` — **0.4px**), 넘침·용지밖 1·1 → 0·0,
//! 공백 제거 글자 수 쪽별 `[87, 582]` → **`[448, 582]` = 한/글과 정확히 일치**.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6787/16774617-electronic-ballot-form.hwp";

/// 정식 fixture는 `MANIFEST.json`의 SHA-256로 고정된다. fixture 부재는 회귀 시험의
/// 성공 조건이 아니므로 읽기 실패를 즉시 드러낸다.
fn sample() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
        .expect("#6787 정식 HWP fixture 읽기")
}

fn find_body(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node);
    }
    node.children.iter().find_map(find_body)
}

/// 카드 크기(253.2px 안팎)의 중첩 표 상자들.
fn card_boxes(node: &RenderNode, out: &mut Vec<(f64, f64)>) {
    if matches!(node.node_type, RenderNodeType::Table { .. })
        && (240.0..270.0).contains(&node.bbox.width)
        && node.bbox.height > 200.0
    {
        out.push((node.bbox.x, node.bbox.y));
    }
    for child in &node.children {
        card_boxes(child, out);
    }
}

/// 두 카드는 **같은 줄**에 나란히 놓여야 한다.
///
/// 수정 전에는 같은 x(272.1)에 y 가 `311.4` / `600.5` 로 289.1px 벌어져 있었다.
#[test]
fn cell_float_group_shares_one_line() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let body = find_body(&tree.root).expect("Body 노드");

    let mut cards = Vec::new();
    card_boxes(body, &mut cards);
    assert_eq!(cards.len(), 2, "후보자 카드 2장을 찾아야 한다: {cards:?}");
    cards.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let dy = (cards[0].1 - cards[1].1).abs();
    assert!(
        dy <= 2.0,
        "두 카드는 같은 줄에 있어야 한다 — #6787 회귀          \
         (y 차 {dy:.1}px, 카드 {cards:?}; 수정 전 289.1px)"
    );
    let dx = cards[1].0 - cards[0].0;
    assert!(
        dx > 200.0,
        "두 카드는 가로로 떨어져 있어야 한다 — #6787 회귀          \
         (x 차 {dx:.1}px, 카드 {cards:?}; 수정 전 0.0px)"
    );
}

/// 1쪽 본문이 용지 하한을 넘으면 안 된다.
///
/// 수정 전에는 바깥 표가 `1462.7px`(본문 `1009.1px`)로 부풀어 용지 밖 `415.8px`,
/// 한/글 대비 **361자**가 잘렸다.
#[test]
fn ballot_table_fits_the_page() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(core.page_count(), 2, "한/글 2024 와 같은 2쪽이어야 한다");

    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let paper_bottom = tree.root.bbox.y + tree.root.bbox.height;

    fn worst(node: &RenderNode, bottom: f64, out: &mut f64) {
        if matches!(node.node_type, RenderNodeType::Table { .. }) {
            *out = out.max(node.bbox.y + node.bbox.height - bottom);
        }
        for child in &node.children {
            worst(child, bottom, out);
        }
    }
    let mut over = 0.0f64;
    worst(&tree.root, paper_bottom, &mut over);

    assert!(
        over <= 0.5,
        "표가 용지 밖으로 나가면 안 된다 — #6787 회귀          \
         (초과 {over:.1}px, 용지 하한 {paper_bottom:.1}; 수정 전 +415.8px)"
    );
}
