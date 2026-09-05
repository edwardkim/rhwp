//! [Issue #6754] 글자 없는 문단의 `TAC 그림 + TAC 표` 가 나란히가 아니라 **세로로**
//! 쌓여, 그 아래 전부가 밀리고 마지막 표의 캡션 행이 용지 밖으로 나가 19자가 사라졌다.
//!
//! `156585314` 3쪽 — 저장 사다리는 둘을 **같은 `vpos`(9237)** 에 적어 나란히임을
//! 증언한다. 한/글도 사진 오른쪽에 표를 둔다.
//!
//! ```text
//!               저장 y    rhwp(수정 전)   차
//!   pi=33 그림    217.6      217.6         0.0
//!   pi=33 표      217.6      365.5      +147.9    ← 그림 아래로 내려간다
//!   pi=41 표      849.6      983.4      +133.8    ← 그 아래 전부가 밀린다
//!                                                    (캡션 행 1131.6 > 용지 1122.5)
//! ```
//!
//! 세 겹이 함께 막고 있었다.
//!
//! 1. `is_tac_table_inline` 의 폭 합산이 **표만** 셌다 — `그림 + 표` 는 표가 하나뿐이라
//!    `len() >= 2` 를 못 넘고 블록으로 떨어진다(그림 9302 + 표 38274 = 47576 ≤ 줄폭 48188).
//! 2. `layout_empty_runs_line` 에 **표 분기가 없었다** — 인라인으로 분류되면 PageItem
//!    경로도 안 그리므로 표가 통째로 사라진다.
//! 3. `tac_offsets_for_line` 의 폭 0 인 줄은 TAC 를 **하나만** 소유한다 — 둘째(표)가
//!    어느 줄에도 안 실린다.
//!
//! 수정 후 3쪽 글자 491 → **510**(한/글 510), 전체 1447 → **1466**(한/글 1466).

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 원 이슈의 실물 문서를 저장소 샘플로 고정한다.
/// 출처와 원본 SHA-256은 `samples/issue6754/README.md`에 기록한다.
fn sample() -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6754/156585314-ssagirang-barley.hwp");
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("필수 회귀 샘플 {} 읽기 실패: {error}", path.display()))
}

fn tables(node: &RenderNode, out: &mut Vec<(u16, u16, f64, f64)>) {
    if let RenderNodeType::Table(t) = &node.node_type {
        out.push((t.row_count, t.col_count, node.bbox.x, node.bbox.y));
    }
    for child in &node.children {
        tables(child, out);
    }
}

fn lowest_run_bottom(node: &RenderNode) -> f64 {
    let own = if matches!(node.node_type, RenderNodeType::TextRun(_)) {
        node.bbox.y + node.bbox.height
    } else {
        f64::MIN
    };
    node.children
        .iter()
        .map(lowest_run_bottom)
        .fold(own, f64::max)
}

/// 그림 옆에 놓여야 할 4×8 표가 **그려져야** 하고, 저장 사다리가 적은 자리에 있어야 한다.
#[test]
fn tac_table_shares_the_line_with_the_tac_picture() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(core.page_count(), 3, "한/글 2024 와 같은 3쪽이어야 한다");

    let tree = core.build_page_render_tree(2).expect("3쪽 render tree");
    let mut found = Vec::new();
    tables(&tree.root, &mut found);

    let (_, _, _, y) = found
        .iter()
        .find(|(r, c, _, _)| *r == 4 && *c == 8)
        .copied()
        .expect("3쪽에 4×8 표가 그려져야 한다 — #6754 회귀(인라인 분류 뒤 아무도 안 그림)");

    // 저장 사다리 vpos 9237 = 본문 상단 + 123.2px = 217.6
    assert!(
        (y - 217.6).abs() <= 6.0,
        "4×8 표는 그림과 같은 줄(저장 vpos 217.6)에 있어야 한다 — #6754 회귀 \
         (실측 {y:.1}; 수정 전 365.5)"
    );
}

/// 마지막 표의 캡션 행이 용지 안에 있어야 한다.
///
/// ⚠ 이 축은 음성 대조에서 **통과한다** — 결함이 있을 때 그 행의 `TextRun` 은 렌더
/// 트리에도 안 남는다(용지 밖에서 잘린다). 판별력은 위 축이 갖고, 이 축은 뒤에 다른
/// 수정이 그 행을 되살렸을 때 자리를 지키는 가드다.
#[test]
fn last_caption_row_stays_on_the_paper() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let tree = core.build_page_render_tree(2).expect("3쪽 render tree");
    let paper_bottom = tree.root.bbox.y + tree.root.bbox.height;
    let lowest = lowest_run_bottom(&tree.root);

    assert!(
        lowest <= paper_bottom + 0.5,
        "3쪽 글자가 용지 밖으로 나가면 안 된다 — #6754 회귀 \
         (최하단 {lowest:.1} > 용지 {paper_bottom:.1})"
    );
}
