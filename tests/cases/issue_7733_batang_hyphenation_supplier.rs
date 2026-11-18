//! 검사 유형: 관계형 — 원본 바탕의 U+2027 결측을 독립 Print 공급자와 자연 전진으로 보호한다.
//! 주석 갱신: 2026-11-18 07:01 KST.
//! 화학 27쪽 보정 전 Native 99.25221%, fresh WASM 99.37167%(96 DPI·PDFium·2px raw).
//! 독립 한컴 2020 Hancom PDF Print의 xref 28, glyph 7은 178/1024em이다.
//! 원본 TTF 바탕의 cmap 결측과 HFT·다른 프로그램의 별개 계약을 구분한다.
//! 개인 글꼴을 열거나 설치 상태에 의존하지 않으며 절대 페이지 좌표를 고정하지 않는다.

use rhwp::renderer::{
    layout::{EmbeddedTextMeasurer, TextMeasurer},
    svg::SvgRenderer,
    Renderer, TextStyle,
};

fn style(family: &str) -> TextStyle {
    TextStyle {
        font_family: family.to_owned(),
        font_size: 32.0,
        source_font_is_ttf: true,
        ..Default::default()
    }
}
fn family(source: &TextStyle) -> String {
    let mut renderer = SvgRenderer::new();
    renderer.draw_text_positioned("‧", 120.0, 152.64, source, None);
    let xml = format!("<svg>{}</svg>", renderer.output());
    let doc = roxmltree::Document::parse(&xml).unwrap();
    doc.descendants()
        .find(|n| n.has_tag_name("text") && n.text() == Some("‧"))
        .unwrap()
        .attribute("font-family")
        .unwrap()
        .split(',')
        .next()
        .unwrap()
        .trim_matches(['\'', '"'])
        .to_owned()
}
#[test]
fn batang_missing_hyphenation_point_uses_the_independent_supplier_advance() {
    for name in ["바탕", "Batang"] {
        let source = style(name);
        // 전체 폭 추정 API는 정수 px로 반올림한다. 실제 글줄이 소비하는
        // 글자별 경계에서 4 HU 장치 격자의 반 칸(2 HU) 오차만 허용한다.
        let actual = EmbeddedTextMeasurer.compute_char_positions("‧", &source)[1];
        assert!(
            (actual - source.font_size * 178.0 / 1024.0).abs() <= 2.0 * 96.0 / 7200.0,
            "독립 공급자 전진: {name}, {actual}"
        );
    }
}
#[test]
fn paint_uses_the_same_supplier_as_measurement() {
    assert_eq!(family(&style("Batang")), "Haansoft Batang");
}
#[test]
fn hft_source_does_not_borrow_the_ttf_programs_gap() {
    let mut source = style("Batang");
    source.source_font_is_ttf = false;
    source.source_font_is_hft = true;
    assert_eq!(family(&source), "Batang");
}
