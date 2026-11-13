use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SOURCE: &[u8] =
    include_bytes!("../../mydocs/pr/assets/coanchored-table-pagination/float-pagination.hwp");

fn top_level_tables<'a>(node: &'a RenderNode, tables: &mut Vec<&'a RenderNode>) {
    if matches!(&node.node_type, RenderNodeType::Table(_)) {
        tables.push(node);
        return;
    }
    for child in &node.children {
        top_level_tables(child, tables);
    }
}

fn text(node: &RenderNode, result: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        result.push_str(run.display_or_text());
    }
    for child in &node.children {
        text(child, result);
    }
}

#[test]
fn coanchored_title_and_body_stay_on_the_first_page() {
    let core = DocumentCore::from_bytes(SOURCE).unwrap();
    assert_eq!(core.page_count(), 2);

    let first = core.build_page_render_tree(0).unwrap();
    let second = core.build_page_render_tree(1).unwrap();
    let mut first_tables = Vec::new();
    let mut second_tables = Vec::new();
    top_level_tables(&first.root, &mut first_tables);
    top_level_tables(&second.root, &mut second_tables);
    let [title, body] = first_tables.as_slice() else {
        panic!("Expected the title and body tables on page 1");
    };
    let [next] = second_tables.as_slice() else {
        panic!("Expected only the following table on page 2");
    };
    assert!(title.bbox.y + title.bbox.height <= body.bbox.y);
    for (table, expected) in [
        (title, "Title"),
        (body, "BodyEnd of bodyBody end"),
        (next, "Next page"),
    ] {
        let mut actual = String::new();
        text(table, &mut actual);
        assert_eq!(actual, expected);
    }
}
