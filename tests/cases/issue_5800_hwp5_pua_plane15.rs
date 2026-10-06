//! [#5800] HWP5 원시 한컴 사용자 기호의 **평면-15 표시 키 정규화** 계약.
//!
//! 같은 글자를 HWP5 는 BMP 단일 유닛 `0xA000 | X` 로, HWPX 는 평면 15 보충 PUA
//! `U+F0000 | X` 로 싣는다. 표시 매핑표는 평면 15 키만 갖고 있어서, 정규화가 없으면
//! HWP5 문서의 `0xA832` 가 유니코드 U+A832(실로티 나그리 `꠲`)로 그려진다.
//!
//! 정규화는 **표에 값이 있을 때만** 한다 — 값이 없는 코드포인트는 원문을 유지해
//! 미등록 PUA 축(#5599)의 관측 표면을 바꾸지 않는다.

use rhwp::renderer::composer::{expand_pua_render_text, pua_to_display_text, pua_to_text_surface};
use rhwp::wasm_api::HwpDocument;
use std::fs;
use std::path::Path;

/// `tools/fixture_generators/build_issue_5800_fixture.rs` 가 만든 최소 재현 문서.
const FIXTURE: &str = "samples/issue5800-hancom-symbol.hwp";

fn fixture_bytes() -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

#[test]
fn hwp5_raw_symbols_reach_the_display_table() {
    // 검증된 한컴 표시표(`renderer::hancom_pua`) 쪽 값.
    assert_eq!(
        expand_pua_render_text("\u{A832}"),
        "═",
        "0xA832 는 U+F0832 와 같은 글자 — 이중 가로 괘선으로 그려야 함",
    );
    assert_eq!(
        expand_pua_render_text("도장란 \u{A12B}"),
        "도장란 (인)",
        "0xA12B 는 U+F012B 와 같은 글자 — 도장란 `(인)`",
    );
    assert_eq!(
        expand_pua_render_text("\u{A289}\u{A28A}"),
        "①②",
        "0xA289/0xA28A 는 U+F0289/U+F028A 와 같은 글자 — 원문자",
    );
    // 글머리표 표시표(`renderer::layout::map_pua_bullet_char`) 쪽 값도 같이 탄다.
    assert_eq!(
        expand_pua_render_text("\u{A80F}"),
        "━",
        "0xA80F 는 U+F080F 와 같은 글자 — 굵은 가로선",
    );

    // 렌더러 단건 조회와 텍스트 표면도 같은 답을 낸다.
    assert_eq!(pua_to_display_text('\u{A832}').as_deref(), Some("═"));
    assert_eq!(pua_to_text_surface("\u{A832}"), "═");
}

#[test]
fn normalization_only_applies_to_measured_symbols_with_a_table_value() {
    // (1) 한글 실측 값 집합 밖 — `08103` 의 0xA813 은 한글도 평면 15 로 옮기지 않는다.
    assert_eq!(
        expand_pua_render_text("\u{A813}"),
        "\u{A813}",
        "실측 값 집합 밖의 U+A813 을 기호로 오인하면 안 됨",
    );
    // (2) 값 집합 안이지만 평면-15 대응값이 표에 없는 경우 — 원문 유지(#5599 축).
    assert_eq!(
        expand_pua_render_text("\u{A80A}"),
        "\u{A80A}",
        "표에 값이 없으면 의미를 지어내지 말고 원문을 유지해야 함",
    );
    // (3) HWPX 경로(평면 15 원문)는 종전과 동일하다.
    assert_eq!(expand_pua_render_text("\u{F0832}"), "═");
}

#[test]
fn fixture_ir_keeps_raw_units_but_svg_never_paints_them() {
    let bytes = fixture_bytes();
    let document = rhwp::parser::parse_hwp(&bytes).expect("parse issue5800 fixture");
    let ir: String = document.sections[0]
        .paragraphs
        .iter()
        .map(|p| p.text.as_str())
        .collect();
    assert_eq!(
        ir.matches('\u{A832}').count(),
        84,
        "IR 은 HWP5 원시 값을 그대로 보존해야 함(정규화는 표시 경로의 몫)",
    );

    let svg = HwpDocument::from_bytes(&bytes)
        .expect("load issue5800 fixture")
        .render_page_svg_native(0)
        .expect("render issue5800 fixture page 1");
    assert!(svg.contains('═'), "SVG 는 이중 가로 괘선을 그려야 함",);
    assert!(
        !svg.contains('\u{A832}'),
        "SVG 에 원시 U+A832(`꠲`)가 남으면 안 됨",
    );
    assert!(
        !svg.contains('\u{A12B}') && !svg.contains('\u{A289}'),
        "도장란/원문자 자리에도 원시 코드포인트가 남으면 안 됨",
    );
}
