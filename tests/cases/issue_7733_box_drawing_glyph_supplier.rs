//! 검사 유형: 관계형 — 원본 프로그램의 결측과 실제 공급자의 자연 전진·paint를 대조한다.
//! 주석 갱신: 2026-11-18 06:25 KST.
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

#[test]
fn original_hanyang_hft_right_bar_keeps_its_independent_full_em_advance() {
    // 독립 한컴 2020 Print의 T11 /Widths=1000, FontMatrix=.001이다.
    // 표시용 프로그램의 결측 때문에 이 원본 폭을 반각으로 추정하면 잉크가 눌린다.
    let mut source = style("HY Sinmyeongjo");
    source.source_font_is_ttf = false;
    source.source_font_is_hft = true;
    source.metric_font_family = Some("한양신명조 (HFT)".into());
    source.original_glyph_metric_family = Some(std::sync::Arc::new("한양신명조".to_owned()));
    for size in [16.0, 32.0] {
        source.font_size = size;
        let width = EmbeddedTextMeasurer.estimate_text_width("╺", &source);
        assert!(
            (width - size).abs() <= 96.0 / 7200.0,
            "원본 Type3 전각 폭과 같은 크기의 전진: 크기={size}, 전진={width}"
        );
    }
}

#[test]
fn another_hft_source_does_not_inherit_the_hanyang_right_bar_contract() {
    let mut source = style("BatangChe");
    source.source_font_is_ttf = false;
    source.source_font_is_hft = true;
    let ordinary = EmbeddedTextMeasurer.estimate_text_width("╺", &source);
    source.original_glyph_metric_family = Some(std::sync::Arc::new("산세리프".to_owned()));
    assert_eq!(
        EmbeddedTextMeasurer.estimate_text_width("╺", &source),
        ordinary,
        "같은 표식이라는 이유로 다른 원본 프로그램의 폭을 인증하지 않는다"
    );
}
