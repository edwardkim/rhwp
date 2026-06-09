const WEB_CANVAS_SOURCE: &str = include_str!("../src/renderer/web_canvas.rs");

#[test]
fn web_canvas_layer_leaf_replay_does_not_rebuild_render_nodes() {
    assert!(
        WEB_CANVAS_SOURCE.contains("fn render_layer_node("),
        "render_layer_node should exist"
    );
    assert!(
        WEB_CANVAS_SOURCE.contains("self.render_paint_op(op)"),
        "WebCanvas layer leaf replay should dispatch PaintOp payloads directly"
    );
    assert!(
        !WEB_CANVAS_SOURCE.contains("RenderNode::new"),
        "WebCanvas layer replay must not rebuild temporary RenderNode wrappers"
    );
}
