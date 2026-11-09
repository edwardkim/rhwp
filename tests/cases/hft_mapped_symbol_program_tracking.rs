//! 검사 유형: 관계형(기호 프로그램·본문 꾸밈 분리), 고정형(독립 Print 474쪽의 마지막 기호 원점 ±2 CSSpx).
//! 주석 갱신: 2026-11-09T13:33:56+09:00.
//! 실물 474쪽: 2026-11-09 B4 Native/fresh WASM raw 99.88894%, 96 DPI·compat2022·2px.
//! 독립 Print pdf/issue8371/hwp3-sample10-2020.pdf, SHA310e3251e910ac8c7c6f34d0e50d85c4a0ecbbadd65e69fe4f5071370931a918.
//! 두 경로의 같은 쪽 review·overlay를 직접 판독했다. 합성 검사는 독립 기호 프로그램의 소유 관계다.
//! 원 계약: #5793 → PR #5829 → 통합 #5844. 본문 HFT의 굵기 전진은 보존하고, 별도 기호 프로그램에 재사용하지 않는다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    renderer::{
        layout::{EmbeddedTextMeasurer, TextMeasurer},
        render_tree::{RenderNode, RenderNodeType, TextRunNode},
        TextStyle,
    },
    DocumentCore,
};
fn style(face: &str, bold: bool) -> TextStyle {
    TextStyle {
        font_family: "HY Junggothic".into(),
        metric_font_family: Some(face.into()),
        source_font_is_hft: true,
        font_size: 20.,
        bold,
        ..Default::default()
    }
}
fn advance(s: &TextStyle, text: &str) -> f64 {
    let p = EmbeddedTextMeasurer.compute_char_positions(text, s);
    p[p.len() - 1]
}
#[test]
fn implicit_double_rule_does_not_inherit_the_body_hft_bold_tracking() {
    for face in ["한양중고딕 (HFT)", "한양신명조 (HFT)"] {
        let normal = style(face, false);
        let bold = style(face, true);
        for symbol in [
            '\u{F0827}',
            '\u{F09E}',
            '\u{F09F}',
            '\u{F0A0}',
            '\u{F0854}',
            '\u{F0855}',
            '\u{F02FC}',
        ] {
            for (ratio, spacing) in [(1.0, 0.0), (0.8, -0.5), (1.2, 1.0)] {
                let mut ordinary = normal.clone();
                ordinary.ratio = ratio;
                ordinary.letter_spacing = spacing;
                let mut decorated = ordinary.clone();
                decorated.bold = true;
                let text = symbol.to_string();
                assert!(
                    (advance(&decorated, &text) - advance(&ordinary, &text)).abs() <= 1. / 75.,
                    "별도 기호 프로그램의 전진에는 본문 꾸밈을 더하지 않는다: {face} {symbol:?} ratio={ratio} tracking={spacing}"
                );
            }
        }
        assert!(
            advance(&bold, "A") > advance(&normal, "A"),
            "원래 본문 HFT 굵기 전진을 제거하지 않는다"
        );
    }
}
#[test]
fn bmp_and_plane15_source_rule_share_the_same_symbol_program() {
    let s = style("한양중고딕 (HFT)", true);
    assert!((advance(&s, "\u{A827}") - advance(&s, "\u{F0827}")).abs() <= 1. / 75.);
    assert!(
        (advance(&s, "\u{A827}") - advance(&style("한양중고딕 (HFT)", false), "\u{F0827}")).abs()
            <= 1. / 75.,
        "원시 HWP5 별칭에도 같은 기호 전진을 사용한다"
    );
}
#[test]
fn literal_unicode_rule_keeps_its_own_program_contract() {
    let s = style("한양중고딕 (HFT)", true);
    assert!(
        advance(&s, "═") > advance(&style("한양중고딕 (HFT)", false), "═"),
        "Unicode 원문에는 기존 본문 꾸밈 계약을 유지한다"
    );
    assert!(
        advance(&s, "A") > advance(&style("한양중고딕 (HFT)", false), "A"),
        "원래 HFT 본문 굵기 전진을 보존한다"
    );
}
#[test]
fn source_page474_keeps_the_independent_print_last_symbol_origin() {
    fn find<'a>(n: &'a RenderNode) -> Option<(&'a RenderNode, &'a TextRunNode)> {
        if let RenderNodeType::TextRun(r) = &n.node_type {
            if r.section_index == Some(0) && r.para_index == Some(17095) {
                return Some((n, r));
            }
        }
        n.children.iter().find_map(find)
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-sample10.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(false);
    let tree = core.build_page_render_tree(473).unwrap();
    let (n, r) = find(&tree.root).unwrap();
    assert_eq!(r.text, "==========".to_string() + &"\u{F0827}".repeat(25));
    assert_eq!(
        r.text,
        core.document().sections[0].paragraphs[17095].text,
        "원문·문자 순서·누락·중복을 보존한다"
    );
    let p = r
        .layout_positions
        .clone()
        .unwrap_or_else(|| EmbeddedTextMeasurer.compute_char_positions(&r.text, &r.style));
    assert_eq!(p.len(), 36);
    let last = n.bbox.x + p[34];
    // 독립 PDF 마지막 Haansoft Batang 원점279.808349609375pt를96DPI로 환산했다.
    assert!(
        (last - 373.0777994791667).abs() <= 2.,
        "별도 이중선 프로그램의 마지막 원점: {last}"
    );
}
