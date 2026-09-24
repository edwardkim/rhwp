//! #7358 maintainer correction: the last multi-row RowBreak fragment on page 28
//! repeats the table's saved outer top margin. Hancom 2024 PDF p28 has its
//! continuation border at 58.136pt = 77.515px (96 dpi), while the old renderer
//! put it at body top 75.6px. The following table likewise started too high.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/86712_regulatory_analysis.hwp";

fn table<'a>(node: &'a RenderNode, para_index: usize) -> Option<&'a RenderNode> {
    if matches!(
        &node.node_type,
        RenderNodeType::Table(meta) if meta.para_index == Some(para_index)
    ) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| table(child, para_index))
}

#[test]
fn terminal_rowbreak_continuation_repeats_saved_outer_top() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open sample");
    let page = core.build_page_render_tree(27).expect("page 28");
    let continuation = table(&page.root, 172).expect("28-row continuation");
    let following = table(&page.root, 182).expect("following table");

    let expected_top = 75.6 + 141.0 / 75.0;
    assert!(
        (continuation.bbox.y - expected_top).abs() < 0.5,
        "outer top margin must reopen at the page boundary: {:?}, expected {expected_top}",
        continuation.bbox
    );
    // Hancom PDF's following border is at 434.884pt from its 841pt bottom.
    let pdf_following_top = (841.0 - 434.884) * 96.0 / 72.0;
    assert!(
        (following.bbox.y - pdf_following_top).abs() < 1.0,
        "following table must move with the continuation: {:?}, PDF {pdf_following_top}",
        following.bbox
    );
    assert_eq!(core.page_count(), 64);
}
