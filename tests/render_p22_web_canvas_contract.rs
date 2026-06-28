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
fn web_canvas_control_code_group_labels_follow_active_replay_plane() {
    assert!(
        WEB_CANVAS_SOURCE.contains("fn should_render_group_label("),
        "WebCanvas should gate group labels separately from PaintOp replay"
    );
    assert!(
        WEB_CANVAS_SOURCE
            .contains("group_label_matches_replay_plane(self.active_replay_plane, layer)"),
        "group labels should be filtered against the active replay plane"
    );
    assert!(
        WEB_CANVAS_SOURCE.contains("if self.should_render_group_label(active_layer)"),
        "layer group labels should use the replay-plane gate"
    );
}

#[test]
fn web_canvas_kopub_light_uses_canvas_specific_font_guard() {
    assert!(
        WEB_CANVAS_SOURCE.contains("fn canvas_css_font_weight("),
        "WebCanvas should own the browser-canvas font-weight override"
    );
    assert!(
        WEB_CANVAS_SOURCE.contains("is_kopub_dotum_light_face(&style.font_family)"),
        "KoPub Dotum Light should not inherit the generic 300-weight hint in canvas"
    );
    assert!(
        WEB_CANVAS_SOURCE.contains(
            "'Malgun Gothic','맑은 고딕','Apple SD Gothic Neo','Noto Sans KR ExtraLight'"
        ),
        "KoPub Dotum Light canvas fallback should prefer system Korean sans before ExtraLight"
    );
}
