//! 관계형: 실제 윤곽 프로그램 공급은 원본 HFT의 굵기 전진 소유를 바꾸지 않는다.
//! Native/fresh WASM 일치율: 공개 CFF fixture의 합성 계약이므로 비해당.
//! 실제 반례는30026-safety-reform-toc.hwp 2쪽이다. 독립 Print와 같은 조건에서
//! 고정 devel5077b216 Native99.99715%, 후보63c39702 Native99.89494%였다.
//! 설치 프로그램 공급 전후와 명시적 자연 전진 선택을 구분하며 사설 글꼴에 의존하지 않는다.
//! 주석 갱신: 2026-11-08T03:07:41+09:00. 실제 좌표는 고정하지 않고 독립 Print의 HFT 굵기 계약을 검사한다.
//! 주석 갱신: 2026-11-08T19:43:46.094614+09:00. 직전 파일 변경: 2026-11-08T06:19:56+09:00, 7f76cee60e38fceb63feacaff706cf984b19c427.
//! 자연 전진 선택과 윤곽 보존을 분리한 합성 계약이며 설치 글꼴·실물 좌표에 의존하지 않는다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::renderer::{
    layout::{EmbeddedTextMeasurer, TextMeasurer},
    native_pua_fonts::NativePuaFonts,
    TextStyle,
};
fn style(size: f64) -> TextStyle {
    let data = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/fonts/RHWPHftOutline.otf"
    ));
    let mut fonts = NativePuaFonts::default();
    fonts
        .register_otf("Unmapped HFT fixture", true, false, data, 0)
        .unwrap();
    TextStyle {
        font_family: "Unmapped HFT fixture".into(),
        font_size: size,
        source_font_is_hft: true,
        native_otf_metrics: fonts.get_otf("Unmapped HFT fixture", true, false),
        ..Default::default()
    }
}
fn width(text: &str, style: &TextStyle) -> f64 {
    let positions = EmbeddedTextMeasurer.compute_char_positions(text, style);
    positions.last().unwrap() - positions[0]
}
#[test]
fn hft_bold_decoration_survives_outline_supply() {
    // 독립 HFT Print: 윤곽 자체의 전진과 별도로 굵은 한글에0.05em이 추가된다.
    // fixture의 hmtx는0.9em이므로 윤곽 폭을1em으로 덮어쓰는 보정도 검출한다.
    for size in [20.0, 24.0] {
        let mut supplied = style(size);
        let regular = width("가", &supplied);
        supplied.bold = true;
        let decorated = width("가", &supplied);
        let hu = 96.0 / 7200.0;
        assert!((regular - size * 0.9).abs() <= hu * 3.0);
        assert!(
            (decorated - regular - size * 0.05).abs() <= hu * 3.0,
            "HFT 꾸밈 전진 소유: regular={regular}, bold={decorated}, size={size}"
        );
    }
}
#[test]
fn explicit_natural_outline_does_not_inherit_hft_decoration() {
    let mut selected = style(24.0);
    selected.preserve_glyph_ink = true;
    selected.natural_program_metrics = true;
    let regular = width("가", &selected);
    selected.bold = true;
    assert_eq!(
        width("가", &selected),
        regular,
        "자연 전진을 선택한 세션은 원본 HFT의 추가 전진을 상속하지 않는다"
    );
}
#[test]
fn unmarked_programs_and_spaces_do_not_gain_hft_decoration() {
    let mut unmarked = style(24.0);
    unmarked.source_font_is_hft = false;
    let regular = width("가", &unmarked);
    unmarked.bold = true;
    assert_eq!(width("가", &unmarked), regular);
    let mut anonymous = style(24.0);
    anonymous.font_family.clear();
    let regular = width("가", &anonymous);
    anonymous.bold = true;
    assert_eq!(
        width("가", &anonymous),
        regular,
        "원본 family 주소가 없는 스타일은 공급 프로그램만으로 HFT 계약을 추정하지 않는다"
    );
    let mut hft = style(24.0);
    let space = width(" ", &hft);
    hft.bold = true;
    assert_eq!(width(" ", &hft), space);
}
