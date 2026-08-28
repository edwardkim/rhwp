//! [Issue #6172] 합성한 네모 안 숫자(U+F02B0~F02C4)의 전진폭이 반각이라 상자끼리
//! 겹친다 (2599643 1쪽 `②⓪⓪□-□□□` 신청 번호란, 20pt).
//!
//! 근인: 이 PUA 대역은 폰트 글리프가 아니라 렌더러가 사각형+숫자를 벡터로 **합성**해
//! 그린다(#6127 / PR #6137, `boxed_pua_number`). 그런데 `char_width_decision` 이 이
//! 대역을 몰라 마지막 폴백(0.5em)으로 떨어졌고, 상자 폭 0.72em 보다 전진폭이 좁아
//! 상자마다 4.4pt 씩 겹쳤다.
//!
//! 합성물이 닮은 `□`(U+25A1)의 전진폭으로 재면 같은 줄에 이어지는 진짜 `□` 와
//! 상자 간격이 균일해진다. 한글 2020 오라클의 상자 pitch 는 20.04pt 이고,
//! 수정 후 rhwp 는 20.00pt(=1em) 다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6172/2599643_port_call_form.hwp";
/// 결함이 나타나는 쪽(0-based).
const PAGE: u32 = 0;
/// 합성 대상 PUA 세 글자 — `②⓪⓪` (U+F02B2 U+F02B0 U+F02B0).
const BOXED_RUN: &str = "\u{F02B2}\u{F02B0}\u{F02B0}";
/// 같은 줄에 이어지는 진짜 `□` 런.
const SQUARE_RUN: &str = "\u{25A1}-\u{25A1}\u{25A1}\u{25A1}";
/// 이 칸의 글자 크기 20pt = 26.67px.
const FONT_SIZE_PX: f64 = 80.0 / 3.0;

#[test]
fn issue_6172_boxed_pua_number_advances_like_white_square() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    let page = core
        .build_page_render_tree(PAGE)
        .expect("page 1 render tree");
    let boxed = run_bounds(&page.root, BOXED_RUN).expect("합성 상자 런");
    let square = run_bounds(&page.root, SQUARE_RUN).expect("`□` 런");

    // 세 글자 런이므로 전진폭 = 폭 / 3.
    let advance = boxed.1 / 3.0;
    assert!(
        (advance - FONT_SIZE_PX).abs() < 0.5,
        "합성 상자 전진폭이 전각(1em={FONT_SIZE_PX:.2}px)이 아니다: {advance:.2}px \
         (반각 폴백이면 {:.2}px)",
        FONT_SIZE_PX / 2.0,
    );

    // 겹침 없음: 상자 폭은 0.72em 이므로 전진폭이 그보다 커야 한다.
    assert!(
        advance > FONT_SIZE_PX * 0.72,
        "상자 폭(0.72em={:.2}px)보다 전진폭({advance:.2}px)이 좁아 상자끼리 겹친다",
        FONT_SIZE_PX * 0.72,
    );

    // 뒤 `□` 런은 합성 런 오른쪽 끝에서 시작한다 — 종전에는 40px 왼쪽에서 겹쳐 시작했다.
    assert!(
        square.0 >= boxed.0 + boxed.1 - 0.5,
        "`□` 런이 합성 상자 위로 올라탔다: `□` 시작={:.1}, 합성 런 오른쪽 끝={:.1}",
        square.0,
        boxed.0 + boxed.1,
    );
}

/// `needle` 과 정확히 같은 텍스트를 가진 첫 런의 (왼쪽 x, 폭).
fn run_bounds(node: &RenderNode, needle: &str) -> Option<(f64, f64)> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text == needle {
            return Some((node.bbox.x, node.bbox.width));
        }
    }
    node.children
        .iter()
        .find_map(|child| run_bounds(child, needle))
}
