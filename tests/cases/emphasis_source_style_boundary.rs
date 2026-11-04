//! 독립 한컴 Print에서 확인한 실제 서식 경계와 자동 표시 런 경계를 구분한다.
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const DIFFERENT: &[u8] = include_bytes!(
    "../../mydocs/pr/assets/emphasis_source_style_boundary_20261020/different-styles.hwpx"
);
const EQUAL_IDS: &[u8] = include_bytes!(
    "../../mydocs/pr/assets/emphasis_source_style_boundary_20261020/equal-styles-distinct-ids.hwpx"
);
const SAME_ID: &[u8] = include_bytes!(
    "../../mydocs/pr/assets/emphasis_source_style_boundary_20261020/shared-source-style.hwpx"
);

fn source_text(node: &RenderNode, text: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.section_index == Some(0) && run.para_index == Some(0) {
            text.push_str(&run.text);
        }
    }
    for child in &node.children {
        source_text(child, text);
    }
}

fn check(bytes: &[u8], distinct_ids: bool, colors: &[&str]) {
    let doc = HwpDocument::from_bytes(bytes).unwrap();
    let source = &doc.document().sections[0].paragraphs[0];
    assert_eq!(source.text, "a\u{301}");
    assert_eq!(
        source.char_shape_id_at(0) != source.char_shape_id_at(1),
        distinct_ids
    );
    assert_eq!(doc.page_count(), 1);
    let tree = doc.build_page_render_tree(0).unwrap();
    let mut text = String::new();
    source_text(&tree.root, &mut text);
    assert_eq!(text, source.text, "원문 주소의 누락·중복 없이 표시한다");
    let svg = doc.render_page_svg_native(0).unwrap();
    let xml = roxmltree::Document::parse(&svg).unwrap();
    let marks = xml
        .descendants()
        .filter(|node| node.has_tag_name("text") && node.text() == Some("\u{307}"))
        .collect::<Vec<_>>();
    let actual_colors = marks
        .iter()
        .map(|node| node.attribute("fill").unwrap())
        .collect::<Vec<_>>();
    assert_eq!(actual_colors, colors, "최종 SVG의 원문 구간별 강조점");
    if marks.len() == 2 {
        let origin = |index: usize| {
            marks[index]
                .attribute("transform")
                .unwrap()
                .strip_prefix("translate(")
                .unwrap()
                .split_whitespace()
                .next()
                .unwrap()
                .parse::<f64>()
                .unwrap()
        };
        assert!(origin(0) < origin(1), "독립 PDF와 같은 앞뒤 소유 순서");
    }
}

#[test]
fn distinct_effective_styles_keep_both_source_owned_emphasis_marks() {
    check(DIFFERENT, true, &["#000000", "#ff0000"]);
}

#[test]
fn equal_effective_styles_with_distinct_ids_share_one_emphasis_mark() {
    check(EQUAL_IDS, true, &["#000000"]);
}

#[test]
fn one_source_style_does_not_duplicate_marks_at_a_display_run_boundary() {
    check(SAME_ID, false, &["#000000"]);
}
