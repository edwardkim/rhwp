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

#[test]
fn web_canvas_structural_labels_are_explicit_paint_ops() {
    assert!(
        WEB_CANVAS_SOURCE.contains("PaintOp::ControlLabel { bbox, label }"),
        "WebCanvas should translate producer-owned control labels"
    );
    assert!(
        !WEB_CANVAS_SOURCE.contains("fn should_render_group_label("),
        "WebCanvas must not derive labels from structural groups"
    );
    assert!(
        !WEB_CANVAS_SOURCE.contains("group_label_matches_replay_plane"),
        "explicit label ops follow normal replay-plane dispatch"
    );
}

#[test]
fn web_canvas_decodes_bitmap_bytes_before_html_image_fallback() {
    assert!(
        WEB_CANVAS_SOURCE.contains("fn decode_image_to_canvas(data: &[u8])"),
        "WebCanvas should have a synchronous bitmap decode path"
    );
    assert!(
        WEB_CANVAS_SOURCE.contains("put_image_data(&image_data, 0.0, 0.0)"),
        "decoded image bytes should be copied into an offscreen canvas"
    );
    assert!(
        WEB_CANVAS_SOURCE.contains("draw_image_with_html_canvas_element_and_dw_and_dh"),
        "full-image drawing should paint the decoded canvas before HtmlImage fallback"
    );
    assert!(
        WEB_CANVAS_SOURCE.contains(
            "draw_image_with_html_canvas_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh"
        ),
        "cropped-image drawing should paint the decoded canvas before HtmlImage fallback"
    );
}
