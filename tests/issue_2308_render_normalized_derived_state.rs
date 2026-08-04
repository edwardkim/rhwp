//! Issue #2308 functional regression for the #2195 sparse width overlay.
//!
//! Page-count pins do not catch a width-scale consumer that drifts only the split
//! height of a nested 1×1 table. The authoritative pre-refactor geometry is pinned
//! on the two fragments that exposed a missing scale in
//! `nested_table_mixed_fragment_heights`.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

fn nested_one_by_one_tables(node: &RenderNode, table_depth: usize, out: &mut Vec<(f64, f64)>) {
    let next_depth = if let RenderNodeType::Table(table) = &node.node_type {
        if table_depth >= 1 && table.row_count == 1 && table.col_count == 1 {
            out.push((node.bbox.y, node.bbox.height));
        }
        table_depth + 1
    } else {
        table_depth
    };
    for child in &node.children {
        nested_one_by_one_tables(child, next_depth, out);
    }
}

#[test]
fn issue_2308_sparse_width_overlay_keeps_nested_fragment_geometry() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/76076_regulatory_analysis.hwp");
    let bytes = fs::read(path).expect("read #2195 authority fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse #2195 authority fixture");

    let expected = [(32, 351.1, 636.8), (33, 282.2, 404.3)];
    for (page, expected_y, expected_height) in expected {
        let tree = core
            .build_page_render_tree(page)
            .unwrap_or_else(|error| panic!("render page {}: {error}", page + 1));
        let mut fragments = Vec::new();
        nested_one_by_one_tables(&tree.root, 0, &mut fragments);
        assert!(
            fragments.iter().any(|(y, height)| {
                (y - expected_y).abs() <= 0.2 && (height - expected_height).abs() <= 0.2
            }),
            "page {} nested fragment must preserve pre-overlay geometry \
             y={expected_y:.1} h={expected_height:.1}; got {fragments:?}",
            page + 1
        );
    }
}
