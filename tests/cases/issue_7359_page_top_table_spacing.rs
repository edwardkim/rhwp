//! #7359: the saved HWP5 line ladder keeps the page-top paragraph spacing.
//! Hancom 2020 PDF page 14 places the first sentence near 140px, the table
//! caption near 173px, and the table's first horizontal border at 193px.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp";

fn find_para(node: &RenderNode, para_index: usize, table: bool) -> Option<f64> {
    let matches = match &node.node_type {
        RenderNodeType::Table(meta) if table => meta.para_index == Some(para_index),
        RenderNodeType::TextLine(meta) if !table => meta.para_index == Some(para_index),
        _ => false,
    };
    matches.then_some(node.bbox.y).or_else(|| {
        node.children
            .iter()
            .find_map(|child| find_para(child, para_index, table))
    })
}

#[test]
fn page_top_stored_spacing_aligns_caption_and_table_with_pdf() {
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
        .expect("read source HWP5");
    let core = DocumentCore::from_bytes(&bytes).expect("open source HWP5");
    assert_eq!(core.page_count(), 103);
    let page = core.build_page_render_tree(13).expect("page 14");
    for (actual, expected, label) in [
        (find_para(&page.root, 133, false), 140.28, "heading"),
        (find_para(&page.root, 134, false), 172.55, "table caption"),
        (find_para(&page.root, 134, true), 193.0, "table border"),
    ] {
        let actual = actual.unwrap_or_else(|| panic!("missing {label}"));
        assert!(
            (actual - expected).abs() < 1.5,
            "{label} y={actual:.2}, Hancom PDF y≈{expected:.2}"
        );
    }
}
