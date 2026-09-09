//! #5874: compatibility PDF synthesizes italic only for the actual upright face.

use rhwp::renderer::pdf::{parse_svg_with_synthetic_italic, svg2pdf_to_chunk};
use std::sync::Arc;
use usvg::fontdb::{Database, FaceInfo, Language, Source, Stretch, Style, Weight, ID};

const FONT: &[u8] = include_bytes!("../fixtures/fonts/RHWPExactKerningSmoke.ttf");
const FALLBACK_FONT: &[u8] = include_bytes!("../fixtures/fonts/RHWPBitmapSvgGlyphSmoke.ttf");

fn face(db: &mut Database, name: &str, style: Style, bytes: &[u8]) -> ID {
    db.push_face_info(FaceInfo {
        id: ID::dummy(),
        source: Source::Binary(Arc::new(bytes.to_vec())),
        index: 0,
        families: vec![(name.to_owned(), Language::English_UnitedStates)],
        post_script_name: name.to_owned(),
        style,
        weight: Weight::NORMAL,
        stretch: Stretch::Normal,
        monospaced: false,
    })
}

fn options(style: Style) -> usvg::Options<'static> {
    let mut db = Database::new();
    face(&mut db, "Fixture", style, FONT);
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

fn glyph_transforms(tree: &usvg::Tree) -> Vec<usvg::Transform> {
    fn walk(group: &usvg::Group, result: &mut Vec<usvg::Transform>) {
        for node in group.children() {
            match node {
                usvg::Node::Group(group) => walk(group, result),
                usvg::Node::Text(text) => {
                    for span in text.layouted() {
                        for glyph in &span.positioned_glyphs {
                            result.push(text.abs_transform().pre_concat(glyph.transform()));
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let mut result = Vec::new();
    walk(tree.root(), &mut result);
    result
}

fn close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.0001, "{actual} != {expected}");
}

#[test]
fn upright_fallback_is_sheared_without_moving_baseline_or_advances() {
    let options = options(Style::Normal);
    for attributes in [
        r#"x="12" y="40""#,
        r#"transform="translate(12,40) scale(0.8,1)""#,
        r#"x="12" y="40" text-anchor="middle" dominant-baseline="central""#,
        r#"x="12" y="40" transform="rotate(15 12 40)""#,
    ] {
        let input = svg(&format!(
            r#"<text {attributes} font-style="italic">HAT</text>"#
        ));
        let before = usvg::Tree::from_str(&input, &options).unwrap();
        let (after, report) = parse_svg_with_synthetic_italic(&input, &options).unwrap();
        assert_eq!(report.synthesized_texts, 1, "{attributes}");
        assert_eq!(report.unsupported_texts, 0);
        let old = glyph_transforms(&before);
        let new = glyph_transforms(&after);
        assert_eq!(old.len(), 3);
        assert_eq!(new.len(), old.len());
        for (old, new) in old.iter().zip(&new) {
            close(new.tx, old.tx);
            close(new.ty, old.ty);
            close(new.sx, old.sx);
            close(new.ky, old.ky);
            close(new.kx, old.kx - 0.25 * old.sx);
            close(new.sy, old.sy - 0.25 * old.ky);
        }
        let (chunk, _) = svg2pdf_to_chunk(&after, true).unwrap();
        assert!(
            String::from_utf8_lossy(chunk.as_bytes()).contains("/Subtype /Type0"),
            "synthesis must keep an embedded searchable font, not convert all text to paths"
        );
    }
}

#[test]
fn real_italic_and_oblique_faces_are_not_synthesized_twice() {
    for (style, request) in [(Style::Italic, "italic"), (Style::Oblique, "oblique")] {
        let options = options(style);
        let input = svg(&format!(
            r#"<text x="12" y="40" font-style="{request}">HAT</text>"#
        ));
        let before = usvg::Tree::from_str(&input, &options).unwrap();
        let (after, report) = parse_svg_with_synthetic_italic(&input, &options).unwrap();
        assert_eq!(report.synthesized_texts, 0);
        assert_eq!(report.unsupported_texts, 0);
        assert_eq!(glyph_transforms(&after), glyph_transforms(&before));
    }
}

#[test]
fn normal_text_and_existing_ids_are_preserved() {
    let options = options(Style::Normal);
    for body in [
        r#"<text id="normal" x="12" y="40">HAT</text>"#,
        r#"<text id="normal" x="12" y="40" data-label="italic">HAT</text>"#,
    ] {
        let input = svg(body);
        let before = usvg::Tree::from_str(&input, &options).unwrap();
        let (after, report) = parse_svg_with_synthetic_italic(&input, &options).unwrap();
        assert_eq!(report.synthesized_texts, 0);
        assert_eq!(report.unsupported_texts, 0);
        assert!(after.node_by_id("normal").is_some());
        assert_eq!(glyph_transforms(&after), glyph_transforms(&before));
    }
}

#[test]
fn actual_glyph_fallback_not_requested_family_controls_synthesis() {
    let mut options = options(Style::Italic);
    face(
        Arc::make_mut(&mut options.fontdb),
        "Fallback",
        Style::Normal,
        FALLBACK_FONT,
    );
    let input = svg("<text x='12' y='40' font-style='italic'>\u{e100}</text>");
    let (tree, report) = parse_svg_with_synthetic_italic(&input, &options).unwrap();
    assert_eq!(glyph_transforms(&tree).len(), 1);
    assert_eq!(report.synthesized_texts, 1);
    assert_eq!(report.unsupported_texts, 0);

    let input = svg("<text x='12' y='40' font-style='italic'>H\u{e100}</text>");
    let (tree, report) = parse_svg_with_synthetic_italic(&input, &options).unwrap();
    assert_eq!(glyph_transforms(&tree).len(), 2);
    assert_eq!(report.synthesized_texts, 0);
    assert_eq!(
        report.unsupported_texts, 1,
        "mixed real italic and upright must warn"
    );
}

#[test]
fn complex_text_is_reported_instead_of_silently_distorted() {
    let options = options(Style::Normal);
    for body in [
        r#"<text x="12" y="40" font-style="italic">H<tspan font-style="normal">A</tspan></text>"#,
        r#"<text x="12" y="40" font-style="italic">H<tspan dy="10">A</tspan></text>"#,
        r#"<text x="12" y="40" font-style="italic" writing-mode="tb">HAT</text>"#,
        r#"<text x="12" y="40" font-style="italic" rotate="10">HAT</text>"#,
        r#"<text x="12" y="40" style="font-style:italic;transform:translate(2px)">HAT</text>"#,
    ] {
        let input = svg(body);
        let before = usvg::Tree::from_str(&input, &options).unwrap();
        let (after, report) = parse_svg_with_synthetic_italic(&input, &options).unwrap();
        assert_eq!(report.synthesized_texts, 0, "{body}");
        assert_eq!(report.unsupported_texts, 1, "{body}");
        assert_eq!(glyph_transforms(&after), glyph_transforms(&before));
    }
}

#[test]
fn inherited_style_id_collisions_and_single_quoted_transforms_work() {
    let options = options(Style::Normal);
    let input = svg(concat!(
        "<text id='rhwp-pdf-italic-0' x='12' y='20'>H</text>",
        "<g font-style='italic'><text transform='translate(12 40)'>A</text>",
        "<text id='kept' x='60' y='40'>T</text></g>",
    ));
    let (after, report) = parse_svg_with_synthetic_italic(&input, &options).unwrap();
    assert_eq!(report.synthesized_texts, 2);
    assert_eq!(report.unsupported_texts, 0);
    assert!(after.node_by_id("rhwp-pdf-italic-0").is_some());
    assert!(after.node_by_id("kept").is_some());
    assert_eq!(glyph_transforms(&after).len(), 3);
}

#[test]
fn invalid_svg_is_still_an_error() {
    assert!(
        parse_svg_with_synthetic_italic("<svg font-style='italic'", &options(Style::Normal))
            .is_err()
    );
}
