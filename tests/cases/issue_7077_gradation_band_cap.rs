//! [Issue #7077] 그러데이션 `step` 을 그대로 펴면 PDF 셰이딩이 통째로 버려지는 결함의 가드.
//!
//! ## 계약
//!
//! `expand_gradient_steps` 는 띠를 [`MAX_GRADIENT_BANDS`] 개까지만 만든다. 띠 하나가 stop
//! 두 개이므로 stop 은 최대 128 개고, PDF 백엔드(svg2pdf)가 만드는 `/FunctionType 3`
//! 하위함수는 최대 127 개다.
//!
//! ## 실측 — `samples/issue2470/36382471_masked.hwpx` 1쪽 제목 띠
//!
//! 이 문서의 `borderFill` 10·11 은 한/글 기본값 `step=255 stepCenter=50` 이다.
//!
//! ```text
//!   수정 전  stop 510 개 → 하위함수 509 개
//!            MuPDF: "too many sub-functions in stitching function" → 셰이딩 폐기
//!            rhwp export-pdf 결과를 PyMuPDF 로 렌더하면 띠 두 줄이 백지 (255,255,255)
//!   수정 후  stop 128 개 → 하위함수 127 개, 띠가 그려진다
//!
//!   step 을 바꿔 잰 경계 — 하위함수 255 개(띠 128)는 그려지고 399 개(띠 200)부터 사라진다.
//! ```
//!
//! 색은 한/글 2024 정본과 어긋나지 않는다. 정본이 칠한 띠 조각 256 개를 x 좌표로 짝지어
//! 재면 채널당 차이가 1 을 넘지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::renderer::{expand_gradient_steps, GradientFillInfo, MAX_GRADIENT_BANDS};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue2470/36382471_masked.hwpx";

// `ColorRef` 는 리틀엔디언 BGR 이다 — 아래는 문서가 선언한 RGB 를 그 순서로 적은 값이다.
/// `#3057B9` — 두 띠의 시작색.
const BLUE: u32 = 0x00b9_5730;
/// `#A0B4E6` — 위 띠(`borderFill` 10)의 끝색.
const LIGHT: u32 = 0x00e6_b4a0;
/// `#DFE6F7` — 아래 띠(`borderFill` 11)의 끝색.
const PALE: u32 = 0x00f7_e6df;

/// MuPDF 가 stitching 함수에서 받아주는 하위함수 수의 실측 상한.
const MUPDF_SUB_FUNCTION_LIMIT: usize = 256;

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

fn page_gradients(page: u32) -> Vec<GradientFillInfo> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("#7077 공개 fixture 읽기 {}: {error}", path.display()));
    let tree = HwpDocument::from_bytes(&bytes)
        .expect("parse 36382471_masked")
        .build_page_render_tree(page)
        .unwrap_or_else(|error| panic!("쪽 idx {page} render tree: {error:?}"));
    let mut out = Vec::new();
    collect_gradients(&tree.root, &mut out);
    out
}

/// `step=255` 문서의 stop 수가 PDF stitching 한도 아래로 내려와야 한다.
#[test]
fn step_255_gradation_stays_under_the_pdf_stitching_limit() {
    let grads: Vec<GradientFillInfo> = page_gradients(0)
        .into_iter()
        .filter(|g| g.colors.first().copied() == Some(BLUE))
        .collect();
    assert_eq!(grads.len(), 2, "1쪽 제목 표의 띠 두 줄을 찾아야 한다");

    for grad in &grads {
        assert_eq!(grad.positions.len(), grad.colors.len());
        assert!(
            grad.colors.len() <= MAX_GRADIENT_BANDS * 2,
            "띠가 {}개로 펴졌다 — 상한 {MAX_GRADIENT_BANDS} 를 넘으면 안 된다",
            grad.colors.len() / 2
        );
        // stop N 개 → stitching 하위함수 N-1 개. 회귀하면 MuPDF 가 셰이딩을 통째로 버린다.
        assert!(
            grad.colors.len() - 1 < MUPDF_SUB_FUNCTION_LIMIT,
            "stop {}개는 하위함수 {}개 — MuPDF 한도 {MUPDF_SUB_FUNCTION_LIMIT} 을 넘어 띠가 사라진다",
            grad.colors.len(),
            grad.colors.len() - 1
        );
    }

    // 끝색은 줄여도 그대로여야 한다 — 두 띠는 끝색만 다르다.
    let mut ends: Vec<u32> = grads
        .iter()
        .map(|g| *g.colors.last().expect("마지막 stop"))
        .collect();
    ends.sort_unstable();
    assert_eq!(
        ends,
        {
            let mut want = vec![LIGHT, PALE];
            want.sort_unstable();
            want
        },
        "띠의 끝색이 바뀌었다"
    );
}

/// 줄인 띠의 이웃 간 색 단차가 눈에 남지 않아야 한다 — 상한을 더 낮추면 깨진다.
#[test]
fn capped_bands_keep_the_step_between_neighbours_invisible() {
    let (colors, _) = expand_gradient_steps(&[BLUE, LIGHT], &[0.0, 1.0], 255, 50);
    let bands = colors.len() / 2;
    assert_eq!(bands, MAX_GRADIENT_BANDS, "상한만큼 띠가 나와야 한다");
    assert_eq!(colors[0], BLUE, "첫 띠는 시작색 그대로여야 한다");
    assert_eq!(
        *colors.last().expect("마지막 stop"),
        LIGHT,
        "마지막 띠는 끝색 그대로여야 한다"
    );

    // 띠가 눈에 보이는지는 이웃 띠의 색 단차가 말한다. 이 문서의 램프는 채널 폭이 112 라
    // 띠 64 개면 단차가 2 를 넘지 않는다 — 상한을 8 로 낮추면 16 이 되어 띠가 드러난다.
    let channel = |color: u32, shift: u32| ((color >> shift) & 0xff) as i32;
    let mut worst = 0;
    for band in 1..bands {
        for shift in [0u32, 8, 16] {
            let step =
                (channel(colors[band * 2], shift) - channel(colors[(band - 1) * 2], shift)).abs();
            worst = worst.max(step);
        }
    }
    assert!(
        worst <= 2,
        "이웃 띠의 색 단차가 {worst} 이다 — 띠 상한이 낮아 그러데이션에 띠가 드러난다"
    );
}

/// 상한 아래의 `step` 은 건드리지 않는다 — #6822 가 세운 띠 구조가 그대로 남아야 한다.
#[test]
fn step_below_the_cap_is_untouched() {
    let (colors, positions) = expand_gradient_steps(&[BLUE, LIGHT], &[0.0, 1.0], 2, 8);
    assert_eq!(colors, vec![BLUE, BLUE, LIGHT, LIGHT]);
    assert_eq!(positions, vec![0.0, 0.08, 0.08, 1.0]);
}
