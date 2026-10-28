//! #6111의 저장 바깥 상자 원점과 후속 표의 본문 포함 대조군.
//! API의 절대 위치 검사는 #7731로 이관했다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

#[test]
fn saved_outer_box_anchor_keeps_following_tables_inside_the_body() {
    // 저장 흐름 원점과 가시 제목 상자를 같게 고정한 단언은 #7806에 보관했다.
    // 실제 후속 표의 존재와 본문 내부 포함은 정상 대조군으로 유지한다.
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
