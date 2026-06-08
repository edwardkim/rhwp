const WEB_CANVAS_SOURCE: &str = include_str!("../src/renderer/web_canvas.rs");

#[test]
fn web_canvas_layer_leaf_replay_does_not_rebuild_render_nodes() {
    let body = rust_fn_body(WEB_CANVAS_SOURCE, "fn render_layer_node")
        .expect("render_layer_node should exist");

    assert!(
        body.contains("self.render_paint_op(op)"),
        "WebCanvas layer leaf replay should dispatch PaintOp payloads directly"
    );
    assert!(
        !body.contains("RenderNode::new"),
        "WebCanvas layer leaf replay must not rebuild temporary RenderNode wrappers"
    );
}

fn rust_fn_body<'a>(source: &'a str, signature: &str) -> Option<&'a str> {
    let start = source.find(signature)?;
    let open = source[start..].find('{')? + start;
    let mut depth = 0usize;
    for (offset, ch) in source[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(&source[open..open + offset + 1]);
                }
            }
            _ => {}
        }
    }
    None
}
