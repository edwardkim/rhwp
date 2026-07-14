use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &[u8] = include_bytes!("../samples/hml/formatting_table.hml");
const PARAGRAPH_INDEX: usize = 1;
const CONTROL_INDEX: usize = 0;
const GEOMETRY_TOLERANCE: f64 = 1.0;

fn find_text_run(node: &RenderNode, text: &str) -> Option<BoundingBox> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.para_index == Some(PARAGRAPH_INDEX) && run.text == text {
            return Some(node.bbox);
        }
    }

    node.children
        .iter()
        .find_map(|child| find_text_run(child, text))
}

fn find_table(node: &RenderNode) -> Option<BoundingBox> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(PARAGRAPH_INDEX) && table.control_index == Some(CONTROL_INDEX) {
            return Some(node.bbox);
        }
    }

    node.children.iter().find_map(find_table)
}

#[test]
fn formatting_table_middle_anchor_preserves_vertical_text_flow() {
    let document =
        rhwp::wasm_api::HwpDocument::from_bytes(SAMPLE).expect("parse formatting_table.hml");
    let tree = document
        .build_page_render_tree(0)
        .expect("render formatting_table.hml page 1");

    let leading = find_text_run(&tree.root, "abc").expect("distinct leading abc text run");
    let table = find_table(&tree.root).expect("middle-anchored table");
    let trailing = find_text_run(&tree.root, "efg").expect("distinct trailing efg text run");

    assert!(
        leading.y + leading.height <= table.y + GEOMETRY_TOLERANCE,
        "leading text must remain above the middle-anchored table: abc={leading:?}, table={table:?}"
    );
    assert!(
        table.y + table.height <= trailing.y + GEOMETRY_TOLERANCE,
        "trailing text must flow below the middle-anchored table: table={table:?}, efg={trailing:?}"
    );
}
