//! 고정형: 합성 셀의 문단 범위·schema JSON 계약. 실제 배치·좌표를 고정하지 않는다.
//! 주석 갱신: 2026-11-03T20:16:37+09:00; 직전 파일 커밋: 2026-10-26T00:03:47+09:00 75481c0ed6364ccda48ca32173ec9151a90e6635.
//! 시각 측정 비해당: 합성 노드에 지정한 필드의 직렬화만 검사한다.
use rhwp::document_core::DocumentCore;
use rhwp::paint::{LayerBuilder, RenderProfile};
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType, TableCellNode};

#[test]
fn cell_source_paragraph_range_survives_layer_conversion_and_json() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    for range in [None, Some((2, 5))] {
        let mut tree = core.build_page_render_tree(0).unwrap();
        tree.root.children.push(RenderNode::new(
            999,
            RenderNodeType::TableCell(TableCellNode {
                col: 0,
                row: 0,
                col_span: 1,
                row_span: 1,
                border_fill_id: 0,
                text_direction: 0,
                clip: true,
                page_fragment: false,
                closed_reflow_frame: false,
                model_cell_index: Some(4),
                model_para_range: range,
            }),
            BoundingBox::default(),
        ));
        let layer = LayerBuilder::new(RenderProfile::Screen).build(&tree);
        assert_eq!(
            layer.to_json().contains("\"modelParaRange\":[2,5]"),
            range.is_some()
        );
        assert!(layer.to_json().contains("\"modelCellIndex\":4"));
        let json: serde_json::Value = serde_json::from_str(&layer.to_json()).unwrap();
        assert_eq!(json["schemaMinorVersion"], 35);
    }
}
