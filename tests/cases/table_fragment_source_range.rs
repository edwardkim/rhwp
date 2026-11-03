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
        assert_eq!(json["schemaMinorVersion"], 30);
    }
}
