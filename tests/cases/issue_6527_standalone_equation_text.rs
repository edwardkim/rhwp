use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn compact(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

fn equations(node: &RenderNode, in_line: bool, result: &mut Vec<(String, bool)>) {
    let in_line = in_line || matches!(node.node_type, RenderNodeType::TextLine(_));
    if let RenderNodeType::Equation(eq) = &node.node_type {
        result.push((compact(&eq.script), in_line));
    }
    for child in &node.children {
        equations(child, in_line, result);
    }
}

fn assert_sample_equations(sample: &str, inline: bool) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples")
        .join(sample);
    let bytes = std::fs::read(path).expect("read public sample");
    let core = DocumentCore::from_bytes(&bytes).expect("parse public sample");
    let tree = core.build_page_render_tree(5).expect("page 6 render tree");
    let mut scripts = Vec::new();
    equations(&tree.root, false, &mut scripts);
    assert_eq!(scripts.len(), 2, "price evaluation equations");
    assert_eq!(scripts[0].0.chars().count(), 49);
    assert_eq!(scripts[1].0.chars().count(), 81);
    assert!(scripts.iter().all(|(_, in_line)| *in_line == inline));

    let text = compact(&core.extract_page_text_native(5).expect("page 6 text"));
    let mut previous = text.find("주)입찰가격평점산식").expect("equation heading");
    for (script, _) in &scripts {
        assert_eq!(text.matches(script).count(), 1, "{sample}: {script}");
        let position = text.find(script).expect("equation script");
        assert!(previous < position, "{sample}: equation reading order");
        previous = position;
    }
}

#[test]
fn standalone_equations_are_extracted_once_in_reading_order() {
    assert_sample_equations("hwp3-sample16.hwp", false);
}

#[test]
fn inline_equations_are_extracted_once_in_reading_order() {
    assert_sample_equations("hwp3-sample16-hwp5.hwpx", true);
}
