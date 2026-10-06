//! [Issue #6718] 네이티브 HWP5 의 문단 안 저장 `vpos` 되감김이 **정확히 0** 이면
//! 아무도 쪽 경계로 읽지 않아 본문이 쪽 하한·용지 밖으로 나갔다.
//!
//! `vpos == 0` 은 "되감김"이 아니라 **새 물리 쪽 상단** 표식이라 `[#6542]` 블록이
//! 통째로 배제하고 "별도 기계가 다룬다"고 적어 뒀는데, 그 별도 기계 둘이
//! **네이티브 HWP5 + 각주 없음** 조합에서는 모두 꺼진다.
//!
//! - `internal_vpos_page_break_line` — 호출부의 `source_uses_inline_field_reset` 가
//!   네이티브 HWP5 를 목록에 넣지 않는다(`#6085` 가 지적하고 코드 수정 없이 닫힘).
//! - `native_hwp5_existing_footnote_reset_overlap_break_line` — 각주가 없으면 즉시 반환.
//!
//! `27469` 실측(`layout-anomaly`):
//!
//! ```text
//!   수정 전   넘침 16 · 용지밖 1 · 글자겹침 7      (12쪽, 한/글 2020 도 12쪽)
//!   수정 후   넘침  9 · 용지밖 1 · 글자겹침 2
//! ```
//!
//! ⚠ `pi=62`(9쪽)는 예산을 **0.8px** 차이로 통과해(7줄 합 246.4 vs 예산 247.2) 사다리
//! (@6)를 못 따르고 있었다. 그 0.8px 은 조판 차이가 아니라 계산 오차 규모라
//! `LADDER_FIT_EPSILON_PX = 1.0` 을 둔다.
//!
//! ⚠ 승격은 두 겹으로 좁혔다 — 조각이 쪽 하단 30% 안에서 시작하고, 사다리대로 끊은
//! 뒤 한 줄을 더 얹으면 예산을 넘어야 한다. 둘째 겹이 없으면 `#2070` 시장구조조사가
//! 315 → 316쪽이 된다(그 문서 사다리는 예산이 1.1~1.5줄 남는 자리에서 끊으라고 한다).
//!
//! ⚠⚠ **남은 것** — 둘째 겹이 걷어내는 자리 중 `pi=47`(7쪽)은 이 문서에서는 옳은
//! 자리다(예산이 1.39줄 남는다). 그 하나 때문에 8쪽 마지막 줄의 용지 밖 이탈
//! (`+121.2px`)이 남는다. `#2070` 의 `pi=343`(1.48줄)·`pi=1088`(1.10줄) 과 **줄 단위로
//! 뒤섞여** 있어 슬랙·시작위치·낙폭·직전 항목·문단 수·태그 어느 축으로도 못 갈랐다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6718/27469-child-allowance-retroactive-support.hwp";

/// 정식 fixture는 `MANIFEST.json`의 SHA-256로 고정된다. fixture 부재는 회귀 시험의
/// 성공 조건이 아니므로 읽기 실패를 즉시 드러낸다.
fn sample() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
        .expect("#6718 정식 HWP fixture 읽기")
}

/// 한 쪽의 본문 하한과, 그 아래로 삐져나간 `TextLine` 들의 최대 초과폭.
fn body_overflow(core: &DocumentCore, page: u32) -> Option<(f64, f64)> {
    let tree = core.build_page_render_tree(page).ok()?;
    let body = find_body(&tree.root)?;
    let bottom = body.bbox.y + body.bbox.height;
    let mut worst = 0.0f64;
    collect_line_bottoms(body, &mut |b| {
        worst = worst.max(b - bottom);
    });
    Some((bottom, worst))
}

fn find_body(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node);
    }
    node.children.iter().find_map(find_body)
}

fn collect_line_bottoms(node: &RenderNode, out: &mut impl FnMut(f64)) {
    if matches!(node.node_type, RenderNodeType::TextRun(_)) {
        out(node.bbox.y + node.bbox.height);
    }
    for child in &node.children {
        collect_line_bottoms(child, out);
    }
}

/// 2쪽 — 사다리 `pi=18 @5` 가 지켜져 본문 하한을 넘는 줄이 없어야 한다.
///
/// 수정 전에는 2줄이 `+27.9 / +63.1px` 로 쪽번호 위에 그려졌다.
#[test]
fn page2_zero_vpos_rewind_is_honored() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(core.page_count(), 12, "한/글 2020 과 같은 12쪽이어야 한다");

    let (bottom, worst) = body_overflow(&core, 1).expect("2쪽 render tree");
    assert!(
        worst <= 0.5,
        "2쪽 본문이 하한을 넘으면 안 된다 — #6718 회귀          (초과 {worst:.1}px, 본문 하한 {bottom:.1}; 수정 전 +63.1px)"
    );
}

/// 4쪽 — 사다리 `pi=25 @4` 가 지켜져야 한다. 수정 전 최대 초과 `+108.8px`.
#[test]
fn page4_zero_vpos_rewind_is_honored() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");

    let (bottom, worst) = body_overflow(&core, 3).expect("4쪽 render tree");
    assert!(
        worst < 10.0,
        "4쪽 본문이 쪽 규모로 하한을 넘으면 안 된다 — #6718 회귀          (초과 {worst:.1}px, 본문 하한 {bottom:.1}; 수정 전 +108.8px)"
    );
}

/// 10쪽 — `pi=62` 는 예산을 0.8px 차이로 통과해 사다리(@6)를 못 따랐다.
///
/// 수정 전 `+20.5px`.
#[test]
fn page10_rewind_survives_a_sub_pixel_fit() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");

    let (bottom, worst) = body_overflow(&core, 9).expect("10쪽 render tree");
    assert!(
        worst < 5.0,
        "10쪽 본문이 하한을 넘으면 안 된다 — #6718 회귀          (초과 {worst:.1}px, 본문 하한 {bottom:.1}; 수정 전 +20.5px)"
    );
}
