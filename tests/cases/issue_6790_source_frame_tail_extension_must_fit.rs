//! [Issue #6790] `#5584 ②` 저장 프레임 꼬리 확장이 **위아래(TopAndBottom)** 배치에서
//! 상한이 없어, `RowBreak` 표의 첫 조각이 쪽 예산을 **254px** 넘겨 배치됐다.
//!
//! `17544911` 실측 — 행 2 하나가 1,934.7px 라 인트라-셀 컷이 필요하다.
//!
//! ```text
//!   avail_for_rows = 1005.4        행 0·1 소비 31.1 + 38.6 = 69.7
//!   행 2 에 남는 예산 ≈ 920.6 (패딩 15.1 제외)
//!
//!   DIAG_SCAN CUT_TRY r=2 budget=1178.5 ...   ← +257.9px (한 유닛 규모 24px 의 10배)
//!   DIAG_SPLITSCAN cursor=0 end_row=3 consumed=1263.3 avail=1005.4
//!   TABLE_SPLIT_RESULT: ... fits=false        ← 그런데 그대로 배치된다
//! ```
//!
//! `#6549`(PR #6559)가 **어울림(Square)** 에 상한을 달았지만 이 문서는
//! **위아래(TopAndBottom)** 라 그 갈래에 안 들어간다.
//!
//! ⚠⚠ **크기로 가르지 않는다** — `#6549` 가 382쪽 편람 핀(`issue_3931`·`issue_3930`·
//! `issue_5801`, 확장 15.3~107.4px)과 크기 축으로 갈리지 않는다고 기록했다. 대신
//! 확장의 **결과**를 본다: 저장 프레임은 "한글이 여기서 끊었다"는 증거지 "쪽을
//! 넘겨도 된다"는 허가가 아니다. 이미 예산을 넘긴 조각은 건드리지 않는다 — 그
//! 초과는 `#5057` 저장 첫-조각 허용치가 따로 판정한다.
//!
//! 결과: 쪽수 2 → **3**(= 한/글 2024), 넘침·용지밖 1·1 → **0·0**,
//! 공백 제거 글자 결손 105 → **26**자.
//!
//! ⚠ **남는 축** — 잔여 26자와 2쪽 중첩 표 안 글자겹침 2건은 **중첩 표 이어지는
//! 조각**의 별개 축이다. 이 시험은 쪽수와 넘침·용지밖만 계약한다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6790/17544911-sericulture-training-criteria.hwp";

/// 정식 fixture는 `MANIFEST.json`의 SHA-256로 고정된다. fixture 부재는 회귀 시험의
/// 성공 조건이 아니므로 읽기 실패를 즉시 드러낸다.
fn sample() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
        .expect("#6790 정식 HWP fixture 읽기")
}

fn find_body(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node);
    }
    node.children.iter().find_map(find_body)
}

fn worst_table_overflow(node: &RenderNode, bottom: f64, out: &mut f64) {
    if matches!(node.node_type, RenderNodeType::Table { .. }) {
        *out = out.max(node.bbox.y + node.bbox.height - bottom);
    }
    for child in &node.children {
        worst_table_overflow(child, bottom, out);
    }
}

/// 한/글 2024 와 같은 3쪽이어야 한다.
///
/// 수정 전에는 첫 조각이 예산을 254px 넘겨 표가 안 쪼개지고 2쪽이 됐다.
#[test]
fn row_break_table_splits_into_three_pages() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(
        core.page_count(),
        3,
        "한/글 2024 와 같은 3쪽이어야 한다 — #6790 회귀 (수정 전 2쪽)"
    );
}

/// 1쪽 표가 본문 하한을 넘으면 안 된다.
///
/// 수정 전 `+254.2px`(용지 밖 `+216.4px`).
#[test]
fn first_fragment_stays_inside_the_body() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let body = find_body(&tree.root).expect("Body 노드");
    let bottom = body.bbox.y + body.bbox.height;

    let mut over = 0.0f64;
    worst_table_overflow(body, bottom, &mut over);

    assert!(
        over <= 0.5,
        "1쪽 표가 본문 하한을 넘으면 안 된다 — #6790 회귀          \
         (초과 {over:.1}px, 본문 하한 {bottom:.1}; 수정 전 +254.2px)"
    );
}
