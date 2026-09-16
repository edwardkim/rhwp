//! 내용에 따라 커진 TAC 표 다음 줄은 실제 표 흐름을 이어받는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use std::path::Path;

fn collect(node: &RenderNode, nodes: &mut Vec<RenderNode>) {
    nodes.push(node.clone());
    for child in &node.children {
        collect(child, nodes);
    }
}

fn render(name: &str) -> Vec<RenderNode> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/stored-table-text-tail")
        .join(name);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("fixture"))
        .expect("parse synthetic document");
    assert_eq!(core.page_count(), 1, "{name}");
    let tree = core.build_page_render_tree(0).expect("render");
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    nodes
}

fn text(nodes: &[RenderNode], expected: &str) -> BoundingBox {
    let matches: Vec<_> = nodes
        .iter()
        .filter(
            |node| matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text == expected),
        )
        .collect();
    assert_eq!(matches.len(), 1, "exactly one {expected}");
    matches[0].bbox
}

#[test]
fn stored_text_tail_follows_the_measured_table_and_preserves_the_line_gap() {
    for profile in ["native", "pure"] {
        for count in [2, 8] {
            let mut positions = Vec::new();
            for gap in [0, 600] {
                let name = format!("{profile}-{count}-{gap}.hwpx");
                let nodes = render(&name);
                let tables: Vec<_> = nodes
                    .iter()
                    .filter(|node| matches!(node.node_type, RenderNodeType::Table { .. }))
                    .collect();
                assert_eq!(tables.len(), 1, "{name}");
                let table = tables[0].bbox;
                let footer = text(&nodes, "Footer");
                for index in 1..=count {
                    let cell_text = text(&nodes, &format!("Cell {index}"));
                    assert!(cell_text.y >= table.y - 0.5, "{name}");
                    assert!(
                        cell_text.y + cell_text.height <= table.y + table.height + 0.5,
                        "{name} cell {index} must remain visible"
                    );
                }
                assert!(
                    footer.y >= table.y + table.height - 0.5,
                    "{name}: footer {} precedes table bottom {}",
                    footer.y,
                    table.y + table.height
                );
                if count == 8 {
                    assert!(table.height > 80.5, "fixture must grow beyond 6000 HU");
                }
                positions.push(footer.y);
            }
            // 600 HWPUNIT at 96 dpi is 8 px, independently of table growth.
            assert!((positions[1] - positions[0] - 8.0).abs() < 0.5);
        }
    }
}
