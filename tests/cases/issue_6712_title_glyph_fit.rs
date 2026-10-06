//! Narrow Hangul metrics must not overlap when SVG paints a fallback face.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::{svg::SvgRenderer, Renderer, TextStyle};

fn style() -> TextStyle {
    TextStyle {
        font_family: "휴먼굵은팸체".into(),
        font_size: 34.6666666667,
        bold: true,
        ..Default::default()
    }
}

fn draw(style: &TextStyle, text: &str, positions: Option<&[f64]>) -> String {
    let mut svg = SvgRenderer::new();
    svg.begin_page(800.0, 300.0);
    svg.draw_text_positioned(text, 10.0, 100.0, style, positions);
    svg.end_page();
    svg.output().into()
}

fn attr(svg: &str, glyph: &str, attribute: &str) -> Option<f64> {
    let doc = roxmltree::Document::parse(svg).unwrap();
    doc.descendants()
        .find(|n| n.has_tag_name("text") && n.text() == Some(glyph))
        .unwrap()
        .attribute(attribute)
        .map(|v| v.parse().unwrap())
}

#[test]
fn narrow_hangul_fits_the_stored_advance_without_moving_origins() {
    let svg = draw(&style(), "여름", Some(&[0.0, 19.8266666667, 39.6533333334]));
    assert!((attr(&svg, "여", "textLength").unwrap() - 19.8267).abs() < 0.0001);
    assert!((attr(&svg, "름", "x").unwrap() - 29.8266666667).abs() < 0.0001);
    assert!(svg.contains("lengthAdjust=\"spacingAndGlyphs\""));
}

#[test]
fn distribution_spacing_is_not_stretched_into_the_glyph() {
    let mut s = style();
    s.extra_char_spacing = 10.0;
    let svg = draw(&s, "여름", Some(&[0.0, 29.8266666667, 59.6533333334]));
    assert!((attr(&svg, "여", "textLength").unwrap() - 19.8267).abs() < 0.0001);
    assert!((attr(&svg, "름", "x").unwrap() - 39.8266666667).abs() < 0.0001);
}

#[test]
fn full_em_unknown_face_and_explicit_tracking_keep_existing_paint() {
    for s in [
        TextStyle {
            font_family: "맑은 고딕".into(),
            ..style()
        },
        TextStyle {
            font_family: "unknown-6712-face".into(),
            ..style()
        },
        TextStyle {
            letter_spacing: -1.0,
            ..style()
        },
        TextStyle {
            letter_spacing: 1.0,
            ..style()
        },
    ] {
        let svg = draw(&s, "여름", None);
        assert_eq!(attr(&svg, "여", "textLength"), None, "{s:?}");
    }
}

#[test]
fn uniform_sub_em_hangul_keeps_existing_paint() {
    for font_family in ["함초롬바탕", "HCR Batang", "'함초롬바탕',serif"] {
        for bold in [false, true] {
            let s = TextStyle {
                font_family: font_family.into(),
                font_size: 14.6666666667,
                bold,
                ..Default::default()
            };
            for positions in [None, Some([0.0, 14.2266666667, 28.4533333334])] {
                let svg = draw(&s, "여름", positions.as_ref().map(|p| p.as_slice()));
                assert_eq!(attr(&svg, "여", "textLength"), None, "{s:?}");
                assert_eq!(attr(&svg, "름", "textLength"), None, "{s:?}");
            }
        }
    }
}

#[test]
fn jamo_cluster_is_not_fitted_as_a_single_precomposed_syllable() {
    let svg = draw(&style(), "\u{1112}\u{1161}\u{11ab}", None);
    assert!(!svg.contains("textLength="));
}

#[test]
fn shadow_uses_the_same_fit_as_foreground() {
    let mut s = style();
    s.shadow_type = 1;
    s.shadow_offset_x = 2.0;
    let svg = draw(&s, "여", None);
    let doc = roxmltree::Document::parse(&svg).unwrap();
    let nodes: Vec<_> = doc
        .descendants()
        .filter(|n| n.has_tag_name("text"))
        .collect();
    assert_eq!(nodes.len(), 2);
    assert!(nodes[0].attribute("textLength").is_some());
    assert_eq!(
        nodes[0].attribute("textLength"),
        nodes[1].attribute("textLength")
    );
}

#[test]
fn original_newsletter_title_has_fitted_glyphs_on_the_same_two_pages() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6712/한국어_2026년 8호 가정통신문_여름철 영유아 감염병 예방.hwp");
    let bytes = std::fs::read(path).unwrap();
    let doc = rhwp::parser::parse_document(&bytes).unwrap();
    let mut core = DocumentCore::new_empty();
    core.set_document(doc);
    assert_eq!(core.page_count(), 2);
    let svg = core.render_page_svg_native(0).unwrap();
    let xml = roxmltree::Document::parse(&svg).unwrap();
    let title: Vec<_> = xml
        .descendants()
        .filter(|n| {
            n.has_tag_name("text")
                && n.attribute("font-family")
                    .is_some_and(|f| f.contains("휴먼굵은팸체"))
        })
        .collect();
    assert_eq!(
        title.iter().filter_map(|n| n.text()).collect::<String>(),
        "여름철영유아감염병예방"
    );
    for pair in title.windows(2) {
        let x: f64 = pair[0].attribute("x").unwrap().parse().unwrap();
        let length: f64 = pair[0].attribute("textLength").unwrap().parse().unwrap();
        let next: f64 = pair[1].attribute("x").unwrap().parse().unwrap();
        assert!(x + length <= next + 0.001, "{pair:?}");
    }
}
