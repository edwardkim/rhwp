//! [#7059] 근사-축소가 한/글이 축소하지 않는 표에서도 발동한다 — 갈림은 칸의 저장 사다리다.
//!
//! ## 무엇이 문제였나
//!
//! `#6590` 은 최상위 글자처럼(TAC) 표의 선언 폭이 본문 폭을 근소 초과하면 본문 폭으로 비례
//! 축소했다. 근거는 **host 문단의 저장 lineseg** 였다 — "host 줄이 본문 폭이니 한/글이 표를
//! 본문 폭 줄박스에 실었다". 그 읽기가 틀렸다. host 줄이 본문 폭인 것은 **"표가 그 줄을
//! 오른쪽으로 넘친다"** 는 뜻이기도 하다.
//!
//! ## 정본이 가르는 것
//!
//! 표가 정말 축소돼 저장됐다면 **칸 안 줄**도 축소 폭이어야 한다. 그쪽이 갈림이고, 정본이
//! 그 판정을 지지한다. 네 문서 모두 정본 PDF 가 저장소에 있다.
//!
//! ```text
//!   문서                      선언      본문     정본 괘선            사다리   #6590   이 PR
//!   BlogForm_BookReview      357.19pt 351.49  379.37 우단 = 선언    유지     축소✗   유지✓
//!   BlogForm_MovieReview     357.19   351.49  379.37 우단 = 선언    유지     축소✗   유지✓
//!   hwpx_sample2 p3          541.36   538.60  538~539 폭  = 본문    축소     축소✓   축소✓
//!   BlogForm_Recipe          357.19   351.49  379.37 우단 = 선언    축소     축소✗   축소✗
//! ```
//!
//! 정본으로 판정 가능한 네 건에서 `#6590` 규칙은 1/4, 이 PR 은 **3/4** 를 맞춘다.
//!
//! ⚠ `BlogForm_Recipe` 는 **전후 모두 틀린다**(회귀가 아니라 미해결). 그 문서는 칸 두 개가
//! 모두 축소 폭에 수렴하는데 정본은 선언 폭이다 — 저장값과 정본이 직접 어긋나는 별개 축이라
//! 따로 파야 한다. 이 시험은 그 상태를 **명시적으로 고정**해, 나중에 그 축을 건드리면 소리가
//! 나게 한다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 정본 `pdf/basic/BlogForm_BookReview-hwp-2020.pdf` 외 2판 — 괘선 우단 379.37pt.
const BOOK_REVIEW: &str = "samples/basic/BlogForm_BookReview.hwp";
/// 정본 `pdf/basic/BlogForm_MovieReview-hwp-2020.pdf` — 같은 좌표.
const MOVIE_REVIEW: &str = "samples/basic/BlogForm_MovieReview.hwp";
/// 정본 `pdf/hwpx_sample2-2020.pdf` 3쪽 — 표 폭 538~539pt = 본문 폭.
const HWPX_SAMPLE2: &str = "samples/hwpx_sample2.hwp";
/// ⚠ 사다리와 정본이 어긋나는 미해결 표본. 상태 고정용.
const RECIPE: &str = "samples/basic/BlogForm_Recipe.hwp";

fn max_table_width(node: &RenderNode) -> f64 {
    let mut best = f64::NEG_INFINITY;
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        best = node.bbox.width;
    }
    for child in &node.children {
        best = best.max(max_table_width(child));
    }
    best
}

/// (그려진 표 폭, 선언 폭, 본문 폭) — 전부 px.
fn widths(sample: &str, page: u32) -> (f64, f64, f64) {
    let bytes = std::fs::read(sample).expect("정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let page_def = &core.document().sections[0].section_def.page_def;
    let body = page_def.width - page_def.margin_left - page_def.margin_right;
    let tree = core.build_page_render_tree(page).expect("쪽 렌더");
    let drawn = max_table_width(&tree.root);
    assert!(
        drawn.is_finite(),
        "{sample} {page}쪽에 표 노드가 있어야 한다"
    );
    let px = |hu: u32| rhwp::renderer::hwpunit_to_px(hu as i32, rhwp::renderer::DEFAULT_DPI);
    // 선언 폭은 표 자신의 것을 쓴다 — 문서마다 다르다.
    let declared = drawn.max(px(body));
    (drawn, declared, px(body))
}

/// 사다리가 **선언 폭**을 말하는 표는 축소하지 않는다 — 정본과 같다.
///
/// 수정 전에는 둘 다 본문 폭으로 축소돼 정본보다 6.9px 좁았다.
#[test]
fn ladder_says_declared_so_table_keeps_its_width() {
    for sample in [BOOK_REVIEW, MOVIE_REVIEW] {
        let (drawn, _, body_px) = widths(sample, 0);
        assert!(
            drawn > body_px + 5.0,
            "{sample}: 정본은 표를 본문 폭({body_px:.1}px) 밖으로 6.9px 내보낸다. 그려진 폭 {drawn:.1}px"
        );
        // 정본 괘선 폭 356.86pt = 475.81px. 선폭(약 0.35pt) 안쪽이라 1.5px 로 잠근다.
        assert!(
            (drawn - 476.25).abs() <= 1.5,
            "{sample}: 정본 괘선 폭은 475.8px 다(선언 476.25px). 그려진 폭 {drawn:.2}px"
        );
    }
}

/// 사다리가 **축소 폭**을 말하는 표는 종전대로 축소한다 — 규칙이 죽지 않았다.
#[test]
fn ladder_says_shrunk_so_the_rule_still_fires() {
    let (drawn, _, body_px) = widths(HWPX_SAMPLE2, 2);
    // 선언 폭 54136 HU = 541.36 pt = 721.8 px · 본문 폭 53860 HU = 718.1 px.
    // 정본 3쪽 괘선은 538~539 pt(717.3~718.7 px) 로 본문 쪽이다.
    let declared_px = 721.8_f64;
    assert!(
        (drawn - body_px).abs() < (drawn - declared_px).abs(),
        "hwpx_sample2 3쪽 표는 본문 폭 쪽이어야 한다(정본 538~539pt = 717.3~718.7px).          그려진 {drawn:.1}px · 본문 {body_px:.1}px · 선언 {declared_px:.1}px"
    );
    assert!(
        (drawn - body_px).abs() <= 2.5,
        "본문 폭과 2.5px 안에서 맞아야 한다. 그려진 {drawn:.1}px · 본문 {body_px:.1}px"
    );
}

/// ⚠ 미해결 — 사다리와 정본이 어긋나는 표본의 상태를 고정한다.
///
/// `BlogForm_Recipe` 는 칸 두 개가 모두 축소 폭에 수렴하는데 정본 괘선은 선언 폭
/// (`pdf/basic/BlogForm_Recipe-hwp-2020.pdf` x 22.51..379.37pt, 형제 표본과 같다)이다.
/// `#6590` 규칙에서도 이 PR 에서도 축소된다 — **회귀가 아니라 미해결**이다. 이 축을 건드리면
/// 이 시험이 먼저 소리를 낸다.
#[test]
fn recipe_still_disagrees_with_its_oracle() {
    let (drawn, _, body_px) = widths(RECIPE, 0);
    assert!(
        (drawn - body_px).abs() <= 1.5,
        "Recipe 는 아직 축소된다(정본은 선언 폭). 상태가 바뀌었으면 정본으로 다시 재고 \
         이 시험과 모듈 주석을 함께 갱신해라. 그려진 폭 {drawn:.1}px · 본문 {body_px:.1}px"
    );
}
