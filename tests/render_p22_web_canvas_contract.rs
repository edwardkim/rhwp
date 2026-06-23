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

#[test]
fn web_canvas_all_filter_replays_logical_planes_in_order() {
    assert!(
        WEB_CANVAS_SOURCE.contains("active_replay_plane: Option<PaintReplayPlane>"),
        "WebCanvas should track the currently replayed plane"
    );
    assert!(
        WEB_CANVAS_SOURCE.contains("for replay_plane in PaintReplayPlane::ORDERED"),
        "LayerFilter::All should replay planes in logical paint order"
    );
    assert!(
        WEB_CANVAS_SOURCE.contains("self.active_replay_plane = Some(replay_plane)"),
        "WebCanvas should filter each tree pass to the active replay plane"
    );
}
