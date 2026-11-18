//! 검사 유형: 관계형 — HY견고딕의 결측 U+2027을 독립 Print 공급자와 연결한다.
//! 주석 갱신: 2026-11-18 11:36 KST.
//! 화학 56쪽 보정 전 Native 99.66074%, fresh WASM 99.70125%(96 DPI·PDFium·2px raw).
//! H2GTRE.TTF에는 U+2027이 없고 독립 한컴 2020 Hancom PDF Print의
//! Haansoft Batang(xref 28, glyph 7)은 178/1024em이다. 개인 글꼴 파일은 열지 않는다.
//! TTF 원본과 HFT 원본의 별개 계약을 구분하며 다른 기호로 범위를 넓히지 않는다.

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

fn first_paint_family(source: &TextStyle) -> String {
    let mut renderer = SvgRenderer::new();
    renderer.draw_text_positioned("‧", 0.0, 0.0, source, None);
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
fn missing_point_advance_uses_the_independent_print_supplier() {
    for family in ["HY견고딕", "HYGothic-Extra"] {
        for size in [12.0, 26.666666666666668, 32.0] {
            let mut source = style(family);
            source.font_size = size;
            let advance = EmbeddedTextMeasurer.compute_char_positions("‧", &source)[1];
            assert!(
                (advance - size * 178.0 / 1024.0).abs() <= 2.0 * 96.0 / 7200.0,
                "독립 공급자의 자연 전진: {family}, {size}, {advance}"
            );
        }
    }
}

#[test]
fn paint_and_measurement_use_the_same_missing_point_supplier() {
    assert_eq!(first_paint_family(&style("HY견고딕")), "Haansoft Batang");
}

#[test]
fn hft_source_does_not_borrow_the_ttf_supplier_contract() {
    let mut source = style("HY견고딕");
    source.source_font_is_ttf = false;
    source.source_font_is_hft = true;
    assert_eq!(first_paint_family(&source), "HY견고딕");
}
