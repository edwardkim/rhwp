//! [Issue #6127] 한컴 사각 안 숫자(U+F02B0~F02C4)를 평문에서 raw 통과시키면
//! 함초롬 확장 글꼴이 없는 소비자(브라우저·PyMuPDF·PDF 뷰어)에서 빈칸이 된다 —
//! 2599643 "② 입항회수" 칸의 "200"(네모 2·0·0) 소실. web_canvas 는 이미 평문
//! 벡터 합성(draw_boxed_pua_number)을 갖고 있었고, SVG·Skia 백엔드에 같은
//! 폴백이 없던 패리티 결손이다. U+F02B0(네모 0)은 기존 범위(F02B1~) 밖이라
//! CharOverlap 합성(#4158)에서도 빠져 있었다.
//!
//! 수정: SVG·Skia 평문 경로에 같은 기하(상자 0.72em·숫자 0.5em)의 벡터 합성
//! 추가 + boxed_pua_number 범위를 F02B0(0)까지 확장 + 텍스트 표면 ⓪ 매핑.
//!
//! 결함 상태에서는 SVG 에 raw PUA 3자가 남아 어서션이 실패한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6127/2599643_vessel_pass_application.hwp";

#[test]
fn issue_6127_boxed_numbers_render_as_vector_synthesis() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    let svg = core.render_page_svg_native(0).expect("page 1 svg");

    let raw = svg
        .chars()
        .filter(|c| (0xF02B0..=0xF02C4).contains(&(*c as u32)))
        .count();
    assert_eq!(
        raw, 0,
        "SVG 에 raw 사각 안 숫자 PUA 가 남았다 — 글꼴 부재 환경에서 빈칸이 된다 ({raw}자)"
    );
    // 합성 사각형 3개(2·0·0)가 있어야 한다 — 20pt(26.7px)의 0.72em ≈ 19.2px 상자.
    let boxes = svg
        .match_indices("width=\"19.20\" height=\"19.20\"")
        .count();
    assert!(
        boxes >= 3,
        "네모 안 숫자 2·0·0 의 합성 사각형이 3개여야 한다: {boxes}"
    );
}
