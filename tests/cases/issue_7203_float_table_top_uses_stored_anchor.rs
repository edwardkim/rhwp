//! #6111의 저장 바깥 상자 원점과 후속 표의 본문 포함 대조군.
//! API의 절대 위치 검사는 #7731로 이관했다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

#[test]
fn saved_outer_box_anchor_keeps_following_tables_inside_the_body() {
    // The stored 1x1 anchor ladder advances by declaration + both margins.
    // It denotes a flow origin, not an anchor after the upper margin. Mixing
    // those origins inflated the following table reservation by 19.5px.
    let bytes = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/issue6111/56345_regulatory_impact_analysis.hwp"),
    )
    .unwrap();
    let core = rhwp::document_core::DocumentCore::from_bytes(&bytes).unwrap();
    let tree = core.build_page_render_tree(19).unwrap();
    fn check(node: &RenderNode, body_bottom: Option<f64>, found: &mut bool) {
        let bottom = if matches!(node.node_type, RenderNodeType::Column(_)) {
            Some(node.bbox.y + node.bbox.height)
        } else {
            body_bottom
        };
        if let RenderNodeType::Table(meta) = &node.node_type {
            if let Some(vpos) = match meta.para_index {
                Some(357) => Some(43556),
                Some(358) => Some(45988),
                _ => None,
            } {
                let expected = 75.6 + f64::from(vpos) * 96.0 / 7200.0;
                assert!(
                    (node.bbox.y - expected).abs() < 0.05,
                    "stored flow origin: actual={}, expected={expected}",
                    node.bbox.y
                );
            }
            if meta.para_index == Some(359) {
                *found = true;
                let table_bottom = node.bbox.y + node.bbox.height;
                assert!(
                    table_bottom <= bottom.expect("column") + 0.5,
                    "measured table bottom {table_bottom} exceeds body {bottom:?}"
                );
            }
        }
        for child in &node.children {
            check(child, bottom, found);
        }
    }
    let mut found = false;
    check(&tree.root, None, &mut found);
    assert!(found, "target picture/caption table must remain present");
}
