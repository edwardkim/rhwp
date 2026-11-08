//! 새 표의 배치가 형식만 바꾸어 저장·다시 열기에서 달라지면 안 된다.
//! #7265의 한컴 정본 여백은 유지하면서 편집 중/HWP/HWPX 원점을 맞춰야 한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use serde_json::Value;

fn table_top(node: &RenderNode) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        Some(node.bbox.y)
    } else {
        node.children.iter().find_map(table_top)
    }
}

fn top(core: &DocumentCore) -> f64 {
    table_top(&core.build_page_render_tree(0).unwrap().root).expect("표 테두리")
}

fn generated_table(with_body: bool) -> (DocumentCore, usize, usize) {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    if with_body {
        core.insert_text_native(0, 0, 0, "앞").unwrap();
        core.split_paragraph_native(0, 0, 1, None).unwrap();
    }
    let created: Value = serde_json::from_str(
        &core
            .create_table_native(0, usize::from(with_body), 0, 2, 2)
            .unwrap(),
    )
    .unwrap();
    let para = created["paraIdx"].as_u64().unwrap() as usize;
    let control = created["controlIdx"].as_u64().unwrap() as usize;
    (core, para, control)
}

fn assert_save_origin(core: &DocumentCore) {
    let before = top(core);
    let hwp = DocumentCore::from_bytes(&core.export_hwp_with_adapter().unwrap()).unwrap();
    let hwpx = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
    let hwp_top = top(&hwp);
    let hwpx_top = top(&hwpx);
    assert_eq!(hwp.page_count(), core.page_count());
    assert_eq!(hwpx.page_count(), core.page_count());
    assert!(
        (hwp_top - before).abs() < 0.2 && (hwpx_top - before).abs() < 0.2,
        "표 원점이 저장 형식에 따라 달라짐: 편집 중={before:.2}, HWP={hwp_top:.2}, HWPX={hwpx_top:.2}",
    );
}

#[test]
fn generated_table_padding_origin_survives_hwpx_save() {
    let (mut core, para, control) = generated_table(false);
    core.set_cell_properties_native(
        0,
        para,
        control,
        0,
        r#"{"width":12000,"height":9000,"applyInnerMargin":true,"paddingLeft":750,"paddingRight":1500,"paddingTop":750,"paddingBottom":1500,"verticalAlign":0}"#,
    )
    .unwrap();
    assert_save_origin(&core);
}

#[test]
fn generated_table_offset_origin_survives_hwpx_save() {
    let (mut core, para, control) = generated_table(true);
    core.set_table_properties_native(
        0,
        para,
        control,
        r#"{"treatAsChar":false,"horzRelTo":"Para","horzAlign":"Left","vertRelTo":"Para","vertAlign":"Top","restrictInPage":false,"allowOverlap":false,"horzOffset":850,"vertOffset":1700}"#,
    )
    .unwrap();
    assert_save_origin(&core);
}
