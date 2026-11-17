//! 검사 유형: 관계형 — 원본 프로그램의 결측과 실제 공급자의 자연 전진·paint를 대조한다.
//! 주석 갱신: 2026-11-18 05:26 KST.
//! 화학 92쪽 보정 전 Native 98.49337% / fresh WASM 98.48877%(96 DPI·Print·2px raw).
//! 독립 한컴 2020 Hancom PDF Print의 U+257A는 Haansoft Batang을 사용한다.
//! BatangChe TTC에는 없고, 공급자의 원폭은 1024/1024em이다. 원점 이동으로 맞추지 않는다.
//! 개인 글꼴 파일을 읽지 않는다. 명시적 HFT와 원 프로그램의 실제 hit는 보존한다.

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

fn paint_family(style: &TextStyle) -> String {
    let mut renderer = SvgRenderer::new();
    renderer.draw_text_positioned("╺", 120.0, 152.64, style, None);
    let svg = format!("<svg>{}</svg>", renderer.output());
    let xml = roxmltree::Document::parse(&svg).unwrap();
    xml.descendants()
        .find(|n| n.has_tag_name("text") && n.text() == Some("╺"))
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
fn missing_batangche_right_bar_uses_the_independent_supplier_advance() {
    for family in ["바탕체", "BatangChe"] {
        let source = style(family);
        let advance = EmbeddedTextMeasurer.estimate_text_width("╺", &source);
        assert!(
            (advance - source.font_size).abs() <= 96.0 / 7200.0,
            "원 프로그램의 결측은 독립 Print 공급자의 한 em을 쓴다: {family}, {advance}"
        );
    }
}

#[test]
fn supplier_program_has_its_verified_full_em_advance() {
    let supplied = style("Haansoft Batang");
    let advance = EmbeddedTextMeasurer.estimate_text_width("╺", &supplied);
    assert!(
        (advance - supplied.font_size).abs() <= 96.0 / 7200.0,
        "실제 cmap/hmtx가 확인한 공급자의 원폭: {advance}"
    );
}

#[test]
fn print_uses_the_same_program_that_supplied_the_measured_glyph() {
    assert_eq!(paint_family(&style("BatangChe")), "Haansoft Batang");
}

#[test]
fn explicit_hft_and_existing_supplier_keep_their_own_programs() {
    let mut hft = style("BatangChe");
    hft.source_font_is_ttf = false;
    hft.source_font_is_hft = true;
    assert_eq!(paint_family(&hft), "BatangChe");
    assert_eq!(paint_family(&style("Haansoft Batang")), "Haansoft Batang");
}

#[test]
fn directly_supplied_program_hit_precedes_the_reference_programs_missing_glyph() {
    use rhwp::renderer::native_pua_fonts::NativePuaFonts;
    let mut programs = NativePuaFonts::default();
    programs
        .register_otf(
            "BatangChe",
            false,
            false,
            include_bytes!("../fixtures/issue7733_box_marker/program-present.ttf"),
            0,
        )
        .unwrap();
    let mut supplied = style("BatangChe");
    supplied.preserve_glyph_ink = true;
    supplied.native_otf_metrics = programs.get_otf("BatangChe", false, false);
    assert!(supplied.native_otf_metrics.as_ref().unwrap().has_glyph('╺'));
    assert_eq!(
        paint_family(&supplied),
        "BatangChe",
        "실제 제공 프로그램의 hit를 같은 이름의 기준 TTC 결측으로 대체하지 않는다"
    );
}
