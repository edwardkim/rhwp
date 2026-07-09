//! Issue #1197: paper/page anchored Picture/Table/Shape objects must share one
//! z-order axis regardless of object type.
//!
//! The regression is visible when a low z-order BehindText table is followed by
//! a BehindText full-page image and then a higher z-order table. The current SVG
//! plane pass classifies only ImageNode as BehindText, so it renders the image
//! before every table. That leaves the low z-order table text on top of the
//! image, which is the overlap reported in #1197.

use rhwp::model::shape::TextWrap;
use rhwp::renderer::render_tree::{
    BoundingBox, ImageNode, PageRenderTree, RawSvgNode, RenderLayerInfo, RenderNode,
    RenderNodeType, TableNode,
};
use rhwp::renderer::svg::SvgRenderer;

fn marker_node(id: u32, marker: &str) -> RenderNode {
    RenderNode::new(
        id,
        RenderNodeType::RawSvg(RawSvgNode::new(format!("<!--{marker}-->\n"))),
        BoundingBox::new(0.0, 0.0, 1.0, 1.0),
    )
}

fn table_with_label(tree: &mut PageRenderTree, label: &str, para_index: usize) -> RenderNode {
    let id = tree.next_id();
    let mut table = RenderNode::new(
        id,
        RenderNodeType::Table(TableNode {
            row_count: 1,
            col_count: 1,
            border_fill_id: 0,
            section_index: Some(0),
            para_index: Some(para_index),
            control_index: Some(0),
        }),
        BoundingBox::new(40.0, 40.0, 180.0, 40.0),
    );

    let text_id = tree.next_id();
    table.children.push(marker_node(text_id, label));
    table.set_layer(RenderLayerInfo::new(
        Some(TextWrap::BehindText),
        para_index as i32,
        para_index as u32,
    ));
    table
}

fn behind_text_image(tree: &mut PageRenderTree) -> RenderNode {
    let mut image = ImageNode::new(1, None);
    image.text_wrap = Some(TextWrap::BehindText);
    image.external_path = Some("Z11_IMAGE".to_string());
    RenderNode::new(
        tree.next_id(),
        RenderNodeType::Image(image),
        BoundingBox::new(0.0, 0.0, 260.0, 180.0),
    )
    .with_layer(RenderLayerInfo::new(Some(TextWrap::BehindText), 11, 11))
}

fn master_page_marker(tree: &mut PageRenderTree) -> RenderNode {
    let mut master = RenderNode::new(
        tree.next_id(),
        RenderNodeType::MasterPage,
        BoundingBox::new(0.0, 0.0, 300.0, 220.0),
    );
    master
        .children
        .push(marker_node(tree.next_id(), "MASTER_PAGE_BACKGROUND"));
    master
}

fn render_synthetic_z_order_svg() -> String {
    let mut tree = PageRenderTree::new(0, 300.0, 220.0);

    // This order models the intended object z-order:
    // z=1 low table, z=11 full-page image, z=12 final table, z=69 front shape.
    let image = behind_text_image(&mut tree);
    let low_table = table_with_label(&mut tree, "Z01_LOW_TABLE", 1);
    let final_table = table_with_label(&mut tree, "Z12_FINAL_TABLE", 12);
    let master_page = master_page_marker(&mut tree);
    let front_shape = marker_node(tree.next_id(), "Z69_FRONT_SHAPE")
        .with_layer(RenderLayerInfo::new(Some(TextWrap::InFrontOfText), 69, 69));

    // Insert the full-page image before the low table to prove that layer
    // metadata, not accidental insertion order, decides SVG replay order.
    tree.root.children.push(image);
    tree.root.children.push(low_table);
    tree.root.children.push(final_table);
    // Insert the master page late to prove it is still rendered below
    // BehindText paper/page objects in the SVG single-stream output.
    tree.root.children.push(master_page);
    tree.root.children.push(front_shape);

    let mut renderer = SvgRenderer::new();
    renderer.render_tree(&tree);
    renderer.output().to_string()
}

fn offset(svg: &str, needle: &str) -> usize {
    svg.find(needle)
        .unwrap_or_else(|| panic!("missing marker {needle} in SVG:\n{svg}"))
}

#[test]
fn paper_anchored_tables_and_images_follow_one_z_order_axis() {
    let svg = render_synthetic_z_order_svg();

    let master_page = offset(&svg, "MASTER_PAGE_BACKGROUND");
    let low_table = offset(&svg, "Z01_LOW_TABLE");
    let image = offset(&svg, "Z11_IMAGE");
    let final_table = offset(&svg, "Z12_FINAL_TABLE");
    let front_shape = offset(&svg, "Z69_FRONT_SHAPE");

    assert!(
        master_page < low_table,
        "master page content must be below BehindText paper objects. \
         Current SVG order puts low table offset {low_table} before master page offset {master_page}."
    );
    assert!(
        low_table < image,
        "low z-order BehindText table must be below the full-page image. \
         Current SVG order puts image offset {image} before low table offset {low_table}."
    );
    assert!(
        image < final_table,
        "full-page image must be below the higher z-order final table"
    );
    assert!(
        final_table < front_shape,
        "final table must be below the InFrontOfText/front shape"
    );
}
