//! [Issue #6822] 그러데이션의 `step`(띠 개수)·`stepCenter`(전이 위치 %)가 렌더 IR 에
//! 실리지 않아 색 전이가 **언제나 축의 50%** 에 고정되던 결함의 가드.
//!
//! ## 계약
//!
//! 한/글은 그러데이션 축을 `step` 개의 **띠**로 잘라 각 띠를 단색으로 칠하고, 띠 경계들의
//! 가운데를 `stepCenter`% 지점에 놓는다. `stepCenter == 50` 이면 균등이므로 **기본값 문서의
//! 그림은 달라지지 않는다.**
//!
//! ## 실측 — `samples/issue6551/113424_evaluation_guideline.hwpx`
//!
//! 이 문서 하나가 결함군과 통제군을 함께 갖는다. 정본은
//! `pdf/113424_evaluation_guideline-2024.pdf`(한/글 2024, 저장 제품 `hancom-office-2024`)다.
//!
//! ```text
//!   결함군  7쪽 장 제목 막대   step=2  stepCenter=8   #339966 → #FFFFFF
//!           한/글: 축의 7.9% 지점에서 하드 경계, 왼쪽 초록 · 오른쪽 흰색
//!           수정 전: 막대 전폭에 매끄러운 램프 — 상자 안쪽 픽셀의 91.3% 가 오차 Δ>16
//!           수정 후: 8.00% 하드 경계 — 오차 픽셀 7.2% (잔여는 축 각도 결함, 별건)
//!
//!   통제군  29쪽 구분 막대     step=50 stepCenter=50  #000080 → #99CCFF
//!           한/글: 균등한 50개 띠(경계 간격 7.86pt × 50 = 막대 폭 392.4pt)
//!           수정 전후 모두 띠 중앙에서 한/글과 Δ<=2 — 회귀 없음
//! ```
//!
//! ⚠ 통제군이 이 수정의 핵심 안전장치다. `stepCenter` 를 무시하고 균등 분포로 두는 현행
//! 동작이 `stepCenter == 50` 문서에서는 **맞다**. 전이 위치를 바꾸는 어떤 판이든 여기를
//! 깨뜨리면 안 된다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::renderer::{expand_gradient_steps, GradientFillInfo};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue6551/113424_evaluation_guideline.hwpx";

/// 0-based — 결함군(장 제목 `Ⅰ 총 칙`)이 있는 물리 7쪽.
const DEFECT_PAGE: u32 = 6;
/// 0-based — 통제군(구분 막대)이 있는 물리 29쪽.
const CONTROL_PAGE: u32 = 28;

// `ColorRef` 는 리틀엔디언 BGR 이다 — `color_to_svg` 가 `r = color & 0xFF`,
// `b = (color >> 16) & 0xFF` 로 읽는다. 아래는 문서가 선언한 RGB 를 그 순서로 적은 값이다.
/// `#339966`
const GREEN: u32 = 0x0066_9933;
/// `#FFFFFF`
const WHITE: u32 = 0x00ff_ffff;
/// `#000080`
const NAVY: u32 = 0x0080_0000;
/// `#99CCFF`
const SKY: u32 = 0x00ff_cc99;

const EPS: f64 = 1e-6;

fn document() -> HwpDocument {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("#6822 공개 fixture 읽기 {}: {error}", path.display()));
    HwpDocument::from_bytes(&bytes).expect("parse 113424")
}

fn collect_gradients(node: &RenderNode, out: &mut Vec<GradientFillInfo>) {
    if let RenderNodeType::Rectangle(rect) = &node.node_type {
        if let Some(grad) = &rect.gradient {
            out.push((**grad).clone());
        }
    }
    for child in &node.children {
        collect_gradients(child, out);
    }
}

/// 시작색으로 대상 그러데이션을 집는다 — 각 쪽에서 유일하다.
fn gradient_starting_with(page: u32, first: u32) -> GradientFillInfo {
    let tree = document()
        .build_page_render_tree(page)
        .unwrap_or_else(|error| panic!("쪽 idx {page} render tree: {error:?}"));
    let mut all = Vec::new();
    collect_gradients(&tree.root, &mut all);
    let mut hit: Vec<GradientFillInfo> = all
        .into_iter()
        .filter(|g| g.colors.first().copied() == Some(first))
        .collect();
    assert_eq!(
        hit.len(),
        1,
        "쪽 idx {page} 에 #{first:06x} 로 시작하는 그러데이션이 정확히 하나여야 한다"
    );
    hit.pop().expect("대상 그러데이션")
}

/// 결함군 — `step=2 stepCenter=8` 은 축의 8% 에서 끊긴 **두 띠**여야 한다.
#[test]
fn step_two_gradation_is_a_hard_two_band_split_at_step_center() {
    let grad = gradient_starting_with(DEFECT_PAGE, GREEN);

    assert_eq!(
        grad.colors,
        vec![GREEN, GREEN, WHITE, WHITE],
        "두 띠는 각각 단색이어야 한다 — 회귀 시 [초록, 흰색] 두 stop 의 매끄러운 램프가 된다"
    );
    let expected = [0.0, 0.08, 0.08, 1.0];
    assert_eq!(grad.positions.len(), expected.len());
    for (got, want) in grad.positions.iter().zip(expected) {
        assert!(
            (got - want).abs() < EPS,
            "전이 위치가 stepCenter(8%)와 달라졌다 — got {:?}, want {expected:?}",
            grad.positions
        );
    }
}

/// 통제군 — `step=50 stepCenter=50` 은 균등한 50개 띠이고, 그림이 달라지면 안 된다.
#[test]
fn default_step_center_keeps_fifty_uniform_bands() {
    let grad = gradient_starting_with(CONTROL_PAGE, NAVY);

    assert_eq!(grad.colors.len(), 100, "50개 띠 × stop 2개여야 한다");
    assert_eq!(grad.positions.len(), grad.colors.len());
    assert_eq!(grad.colors[0], NAVY, "첫 띠는 시작색 그대로여야 한다");
    assert_eq!(
        *grad.colors.last().expect("마지막 stop"),
        SKY,
        "마지막 띠는 끝색 그대로여야 한다"
    );

    // stepCenter == 50 이면 경계가 균등하다 — 이 등식이 깨지면 기본값 문서의 그림이 바뀐다.
    for band in 0..50usize {
        let (start, end) = (grad.positions[band * 2], grad.positions[band * 2 + 1]);
        assert!(
            (start - band as f64 / 50.0).abs() < EPS
                && (end - (band + 1) as f64 / 50.0).abs() < EPS,
            "띠 {band} 의 경계가 균등하지 않다 — {start}..{end}"
        );
    }
}

/// 음성 통제군 — 띠를 만들 수 없는 값이면 원본을 그대로 둔다.
///
/// `step` 이 없는(0) 문서까지 건드리면 코퍼스 전체의 그림이 바뀐다.
#[test]
fn degenerate_step_values_leave_the_ramp_untouched() {
    let colors = vec![GREEN, WHITE];
    let positions = vec![0.0, 1.0];

    for step in [0i16, 1, -3] {
        let (c, p) = expand_gradient_steps(&colors, &positions, step, 8);
        assert_eq!(c, colors, "step={step} 이면 색을 그대로 둬야 한다");
        assert_eq!(p, positions, "step={step} 이면 위치를 그대로 둬야 한다");
    }

    // 색이 하나뿐이면 띠를 나눌 수 없다.
    let single = vec![GREEN];
    let (c, p) = expand_gradient_steps(&single, &[0.0], 50, 8);
    assert_eq!(c, single);
    assert_eq!(p, vec![0.0]);
}

/// `stepCenter == 50` 은 항등 사상이어야 한다 — 임의의 띠 개수에서 균등 분포가 나온다.
#[test]
fn step_center_fifty_is_the_identity_warp() {
    let colors = vec![NAVY, SKY];
    for bands in [2i16, 3, 7, 50, 100] {
        let (_, p) = expand_gradient_steps(&colors, &[0.0, 1.0], bands, 50);
        for band in 0..bands as usize {
            let want = band as f64 / bands as f64;
            assert!(
                (p[band * 2] - want).abs() < EPS,
                "bands={bands} 띠 {band} 시작이 {}, 기대 {want}",
                p[band * 2]
            );
        }
    }
}
