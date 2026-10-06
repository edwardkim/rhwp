//! #6936: faux bold must preserve face/positions and emit searchable glyphs once.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::renderer::pdf::{parse_svg_with_synthetic_italic, prepare_svg_with_synthetic_bold};
use std::sync::Arc;
use usvg::fontdb::{Database, FaceInfo, Language, Source, Stretch, Style, Weight, ID};

const FONT: &[u8] = include_bytes!("../fixtures/fonts/RHWPExactKerningSmoke.ttf");
const FALLBACK: &[u8] = include_bytes!("../fixtures/fonts/RHWPBitmapSvgGlyphSmoke.ttf");

fn face(db: &mut Database, name: &str, weight: Weight, bytes: &[u8]) -> ID {
    db.push_face_info(FaceInfo {
        id: ID::dummy(),
        source: Source::Binary(Arc::new(bytes.to_vec())),
        index: 0,
        families: vec![(name.into(), Language::English_UnitedStates)],
        post_script_name: name.into(),
        style: Style::Normal,
        weight,
        stretch: Stretch::Normal,
        monospaced: false,
    })
}
fn options(bold: bool) -> usvg::Options<'static> {
    let mut db = Database::new();
    face(&mut db, "Fixture", Weight::NORMAL, FONT);
    if bold {
        face(&mut db, "Fixture", Weight::BOLD, FONT);
    }
    db.set_serif_family("Fixture");
    usvg::Options {
        fontdb: Arc::new(db),
        ..Default::default()
    }
}
fn svg(body: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100"><g font-family="Fixture" font-size="20">{body}</g></svg>"#
    )
}
fn glyphs(tree: &usvg::Tree) -> Vec<(ID, u16, usvg::Transform, String)> {
    fn walk(g: &usvg::Group, out: &mut Vec<(ID, u16, usvg::Transform, String)>) {
        for n in g.children() {
            match n {
                usvg::Node::Group(g) => walk(g, out),
                usvg::Node::Text(t) => {
                    for s in t.layouted() {
                        for g in &s.positioned_glyphs {
                            out.push((
                                g.font,
                                g.id.0,
                                t.abs_transform().pre_concat(g.transform()),
                                g.text.clone(),
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let mut out = vec![];
    walk(tree.root(), &mut out);
    out
}
fn pdf(tree: &usvg::Tree) -> Vec<u8> {
    svg2pdf::to_pdf(
        tree,
        svg2pdf::ConversionOptions {
            compress: false,
            ..Default::default()
        },
        Default::default(),
    )
    .unwrap()
}
fn operators(bytes: &[u8], op: &str) -> usize {
    String::from_utf8_lossy(bytes)
        .lines()
        .filter(|line| line.ends_with(op))
        .count()
}
#[test]
fn bold_regular_face_keeps_glyph_positions_and_emits_once() {
    let options = options(false);
    for attrs in [
        r#"x="12" y="40""#,
        r#"transform="translate(12 40) scale(.8 1)""#,
        r#"x="12" y="40" transform="rotate(15 12 40)""#,
    ] {
        let input = svg(&format!(r#"<text {attrs} font-weight="bold">HAT</text>"#));
        let old = usvg::Tree::from_str(&input, &options).unwrap();
        let (prepared, report) = prepare_svg_with_synthetic_bold(&input, &options).unwrap();
        assert_eq!(report.synthesized_texts, 1);
        assert_eq!(report.unsupported_texts, 0);
        let new = usvg::Tree::from_str(&prepared, &options).unwrap();
        assert_eq!(glyphs(&old), glyphs(&new));
        let bytes = pdf(&new);
        assert_eq!(
            operators(&bytes, " Tj"),
            3,
            "fill/stroke must not duplicate text"
        );
        assert_eq!(operators(&bytes, "2 Tr"), 1, "{attrs}");
        assert_eq!(bytes, pdf(&new), "PDF emission remains deterministic");
    }
}
#[test]
fn real_bold_and_normal_faces_are_byte_unchanged() {
    for (has_bold, weight) in [(true, "bold"), (false, "normal"), (false, "500")] {
        let opts = options(has_bold);
        let input = svg(&format!(
            r#"<text x="12" y="40" font-weight="{weight}">HAT</text>"#
        ));
        let (prepared, report) = prepare_svg_with_synthetic_bold(&input, &opts).unwrap();
        assert_eq!(prepared, input);
        assert_eq!(report.synthesized_texts, 0);
        assert_eq!(report.unsupported_texts, 0);
    }
}
#[test]
fn actual_glyph_fallback_controls_synthesis() {
    let mut opts = options(true);
    face(
        Arc::make_mut(&mut opts.fontdb),
        "Fallback",
        Weight::NORMAL,
        FALLBACK,
    );
    let input = svg("<text x='12' y='40' font-weight='700'>\u{e100}</text>");
    let (prepared, report) = prepare_svg_with_synthetic_bold(&input, &opts).unwrap();
    assert_eq!(report.synthesized_texts, 1);
    let tree = usvg::Tree::from_str(&prepared, &opts).unwrap();
    assert_eq!(operators(&pdf(&tree), " Tj"), 1);
    let input = svg("<text x='12' y='40' font-weight='bold'>H\u{e100}</text>");
    let (prepared, report) = prepare_svg_with_synthetic_bold(&input, &opts).unwrap();
    assert_eq!(prepared, input);
    assert_eq!(
        report.unsupported_texts, 1,
        "never thicken an actual bold fallback twice"
    );
}
#[test]
fn inherited_bold_opacity_and_italic_compose() {
    let opts = options(false);
    let input=svg("<g font-weight='bold'><text id='kept' x='12' y='40' style='fill:#123456;fill-opacity:.5' font-style='italic'>HAT</text></g>");
    let (prepared, report) = prepare_svg_with_synthetic_bold(&input, &opts).unwrap();
    assert_eq!(report.synthesized_texts, 1);
    let (tree, italic) = parse_svg_with_synthetic_italic(&prepared, &opts).unwrap();
    assert_eq!(italic.synthesized_texts, 1);
    assert!(tree.node_by_id("kept").is_some());
    let bytes = pdf(&tree);
    assert_eq!(operators(&bytes, " Tj"), 3);
    assert_eq!(operators(&bytes, "2 Tr"), 1);
    let raw = String::from_utf8_lossy(&bytes);
    assert!(raw.contains("/CA 0.5") && raw.contains("/ca 0.5"));
}
#[test]
fn explicit_stroke_has_one_text_show_and_preserves_reverse_paint_order() {
    let opts = options(false);
    for (attributes,shows,combined) in [
        ("fill='black' stroke='black'",3,1),
        ("fill='red' stroke='blue'",3,1),
        ("fill='black' stroke='black' paint-order='stroke fill'",3,1),
        ("fill='red' stroke='blue' paint-order='stroke fill'",6,0),
        ("fill='black' stroke='black' fill-opacity='.5' stroke-opacity='.5' paint-order='stroke fill'",6,0),
        ("fill='none' stroke='black'",3,0),
    ] {
        let input=svg(&format!("<text x='12' y='40' stroke-width='.6' {attributes}>HAT</text>"));
        let tree=usvg::Tree::from_str(&input,&opts).unwrap();
        let bytes=pdf(&tree);
        assert_eq!(operators(&bytes," Tj"),shows,"{attributes}");
        assert_eq!(operators(&bytes,"2 Tr"),combined,"{attributes}");
    }
}
#[test]
fn mixed_weights_warn_and_existing_stroke_is_preserved() {
    let opts = options(false);
    let input = svg(
        "<text x='12' y='40' font-weight='bold'>H<tspan font-weight='normal'>AT</tspan></text>",
    );
    let (prepared, report) = prepare_svg_with_synthetic_bold(&input, &opts).unwrap();
    assert_eq!(prepared, input);
    assert_eq!(report.unsupported_texts, 1);
    let input = svg("<text x='12' y='40' font-weight='bold' stroke='red'>HAT</text>");
    let (prepared, report) = prepare_svg_with_synthetic_bold(&input, &opts).unwrap();
    assert_eq!(prepared, input);
    assert_eq!(report.synthesized_texts, 0);
}

#[test]
fn child_stroke_override_is_not_falsely_reported_as_synthesized() {
    let opts = options(false);
    let input =
        svg("<text x='12' y='40' font-weight='bold'><tspan stroke='none'>HAT</tspan></text>");
    let (prepared, report) = prepare_svg_with_synthetic_bold(&input, &opts).unwrap();
    assert_eq!(prepared, input);
    assert_eq!(report.synthesized_texts, 0);
    assert_eq!(report.unsupported_texts, 1);
}

#[test]
fn real_bold_complex_paint_does_not_warn() {
    let opts = options(true);
    let input = svg("<defs><linearGradient id='g'><stop stop-color='red'/><stop offset='1' stop-color='blue'/></linearGradient></defs><text x='12' y='40' font-weight='bold' fill='url(#g)'>HAT</text>");
    let (prepared, report) = prepare_svg_with_synthetic_bold(&input, &opts).unwrap();
    assert_eq!(prepared, input);
    assert_eq!(report.unsupported_texts, 0);
}

#[test]
fn hancom_fixture_preserves_new_gulim_and_bold_flags() {
    use rhwp::document_core::DocumentCore;
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn collect(node: &RenderNode, runs: &mut Vec<(String, String, bool)>) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            runs.push((
                run.display_or_text().to_string(),
                run.style.font_family.clone(),
                run.style.bold,
            ));
        }
        for child in &node.children {
            collect(child, runs);
        }
    }
    for input in [
        include_bytes!("../../samples/issue6936/bold-faces.hwp").as_slice(),
        include_bytes!("../../samples/issue6936/bold-faces.hwpx").as_slice(),
    ] {
        let core = DocumentCore::from_bytes(input).unwrap();
        assert_eq!(core.page_count(), 1);
        let tree = core.build_page_render_tree(0).unwrap();
        let mut runs = Vec::new();
        collect(&tree.root, &mut runs);
        for bold in [false, true] {
            assert!(
                runs.iter()
                    .any(|(text, family, weight)| text.contains("새굴림")
                        && family == "New Gulim"
                        && *weight == bold),
                "{runs:?}"
            );
        }
    }
}
