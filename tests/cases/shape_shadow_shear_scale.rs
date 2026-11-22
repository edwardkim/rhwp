//! 도형 그림자 중 작게·크게·기울임은 한/글 12.30처럼 본체를 줄이고 늘리거나 눕힌 실루엣으로 그린다.
//! 값은 한/글 12.30 화면에서 12000×8000 HWPUNIT 사각형의 그림자 외곽을 잰 것이다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::paint::{GroupKind, LayerBuilder, LayerNode, LayerNodeKind, RenderProfile};
use rhwp::renderer::render_tree::{
    BoundingBox, PageRenderTree, RectangleNode, RenderNode, RenderNodeType,
};
use rhwp::renderer::{ShadowStyle, ShapeStyle};

fn transforms(node: &LayerNode, out: &mut Vec<[f64; 6]>) {
    match &node.kind {
        LayerNodeKind::Group {
            children,
            group_kind,
            ..
        } => {
            if let GroupKind::Group(group) = group_kind {
                out.extend(group.paint_transform);
            }
            children.iter().for_each(|c| transforms(c, out));
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            transforms(child, out)
        }
        LayerNodeKind::Leaf { .. } => {}
    }
}

/// 본체 상자(쪽 px)와 그림자 종류로 그린 그림자 행렬을 꺼낸다.
fn shadow_matrix(kind: u32, bbox: BoundingBox) -> Option<[f64; 6]> {
    let style = ShapeStyle {
        fill_color: Some(0xFFFFFF),
        stroke_color: Some(0),
        stroke_width: 1.0,
        shadow: Some(ShadowStyle {
            shadow_type: kind,
            color: 0,
            offset_x: 0.0,
            offset_y: 0.0,
            alpha: 0,
        }),
        ..ShapeStyle::default()
    };
    let rect = RenderNode::new(
        2,
        RenderNodeType::Rectangle(RectangleNode::new(0.0, style, None)),
        bbox,
    );
    let mut tree = PageRenderTree::new(0, 800.0, 1100.0);
    tree.root.children.push(rect);
    let layers = LayerBuilder::new(RenderProfile::Screen).build(&tree);
    let mut found = Vec::new();
    transforms(&layers.root, &mut found);
    found.into_iter().next()
}

fn apply(m: [f64; 6], (x, y): (f64, f64)) -> (f64, f64) {
    (m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5])
}

fn near(a: (f64, f64), b: (f64, f64)) -> bool {
    (a.0 - b.0).abs() < 0.01 && (a.1 - b.1).abs() < 0.01
}

const HU: f64 = 96.0 / 7200.0;

#[test]
fn scale_shadows_shrink_from_top_left_and_grow_from_bottom_right() {
    // 12000×8000 HWPUNIT 상자 = 160×106.67px
    let body = BoundingBox::new(100.0, 100.0, 12000.0 * HU, 8000.0 * HU);
    let (r, b) = (body.x + body.width, body.y + body.height);
    let narrow = shadow_matrix(13, body).expect("작게 그림자를 그려야 한다");
    assert!(near(
        apply(narrow, (body.x, body.y)),
        (100.0 - 8.0, 100.0 - 8.0)
    ));
    assert!(near(
        apply(narrow, (r, b)),
        (100.0 - 8.0 + 9000.0 * HU, 100.0 - 8.0 + 6000.0 * HU)
    ));
    let enlarge = shadow_matrix(14, body).expect("크게 그림자를 그려야 한다");
    assert!(near(apply(enlarge, (r, b)), (r - 8.0, b - 8.0)));
    assert!(near(
        apply(enlarge, (body.x, body.y)),
        (r - 8.0 - 15000.0 * HU, b - 8.0 - 10000.0 * HU)
    ));
}

#[test]
fn shear_shadows_lie_down_from_the_bottom_edge() {
    let body = BoundingBox::new(100.0, 100.0, 12000.0 * HU, 8000.0 * HU);
    let b = body.y + body.height;
    // 뒤쪽(왼쪽 위): 아래 변은 그대로, 위 변은 반 높이로 내려와 4500만큼 왼쪽으로.
    let back = shadow_matrix(5, body).expect("기울임 그림자를 그려야 한다");
    assert!(near(apply(back, (body.x, b)), (body.x, b)));
    assert!(near(
        apply(back, (body.x, body.y)),
        (body.x - 4500.0 * HU, body.y + 4000.0 * HU)
    ));
    // 앞쪽(오른쪽 아래): 위 변이 아래로 뒤집혀 반 높이 아래에서 4500만큼 오른쪽으로.
    let front = shadow_matrix(8, body).expect("기울임 그림자를 그려야 한다");
    assert!(near(apply(front, (body.x, b)), (body.x, b)));
    assert!(near(
        apply(front, (body.x, body.y)),
        (body.x + 4500.0 * HU, b + 4000.0 * HU)
    ));
}

#[test]
fn preset_shadow_boxes_match_hancom_screen() {
    // 한컴의 두 사각형에 해당하는 그림자 외곽을 실제 그리기 행렬에서 읽는다.
    let presets = [
        (1, 12000.0, 8000.0, [-600.0, -600.0, 12000.0, 8000.0]),
        (13, 12000.0, 8000.0, [-600.0, -600.0, 9000.0, 6000.0]),
        (14, 12000.0, 8000.0, [-3600.0, -2600.0, 15000.0, 10000.0]),
        (5, 12000.0, 8000.0, [-4500.0, 4000.0, 16500.0, 4000.0]),
        (6, 12000.0, 8000.0, [0.0, 4000.0, 16500.0, 4000.0]),
        (7, 12000.0, 8000.0, [-4500.0, 8000.0, 16500.0, 4000.0]),
        (8, 6000.0, 12000.0, [0.0, 12000.0, 12750.0, 6000.0]),
    ];
    for (kind, width, height, expected) in presets {
        let (w, h) = (width * HU, height * HU);
        let m = shadow_matrix(kind, BoundingBox::new(0.0, 0.0, w, h)).unwrap();
        let corners = [(0.0, 0.0), (w, 0.0), (0.0, h), (w, h)].map(|p| apply(m, p));
        let left = corners.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
        let right = corners
            .iter()
            .map(|p| p.0)
            .fold(f64::NEG_INFINITY, f64::max);
        let top = corners.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        let bottom = corners
            .iter()
            .map(|p| p.1)
            .fold(f64::NEG_INFINITY, f64::max);
        let actual = [left, top, right - left, bottom - top];
        for (value, hu) in actual.into_iter().zip(expected) {
            assert!((value - hu * HU).abs() < 0.01, "그림자 {kind}: {actual:?}");
        }
    }
    assert!(shadow_matrix(9, BoundingBox::new(0.0, 0.0, 1.0, 1.0)).is_none());
}

#[test]
fn front_shear_mirrors_the_body() {
    let body = BoundingBox::new(0.0, 0.0, 12000.0 * HU, 8000.0 * HU);
    let m = shadow_matrix(7, body).unwrap();
    assert!(near(apply(m, (0.0, 0.0)), (-4500.0 * HU, 12000.0 * HU)));
}

#[test]
fn body_origin_moves_past_left_and_top_shadow() {
    use rhwp::document_core::DocumentCore;

    fn body_frame(node: &RenderNode) -> Option<BoundingBox> {
        if let RenderNodeType::Rectangle(rect) = &node.node_type {
            if rect.style.fill_color == Some(0x123456) {
                return Some(node.bbox);
            }
        }
        node.children.iter().find_map(body_frame)
    }

    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let created: serde_json::Value = serde_json::from_str(
        &core
            .create_shape_control_native(
                0,
                0,
                0,
                12000,
                8000,
                5000,
                6000,
                false,
                "InFrontOfText",
                "rectangle",
                false,
                false,
                &[],
            )
            .unwrap(),
    )
    .unwrap();
    let para = created["paraIdx"].as_u64().unwrap() as usize;
    let ctrl = created["controlIdx"].as_u64().unwrap() as usize;
    core.set_shape_properties_native(
        0,
        para,
        ctrl,
        r#"{"fillType":"solid","fillBgColor":1193046}"#,
    )
    .unwrap();
    let base = body_frame(&core.build_page_render_tree(0).unwrap().root).unwrap();
    for (kind, x, y, expected_x, expected_y) in [
        (1, 0, 0, 600.0, 600.0),
        (4, 0, 0, 0.0, 0.0),
        (14, 0, 0, 3600.0, 2600.0),
        (7, 0, 0, 4500.0, 0.0),
        (1, 100, -200, 500.0, 800.0),
    ] {
        let properties = serde_json::json!({"shadowType":kind,"shadowOffsetX":x,"shadowOffsetY":y});
        core.set_shape_properties_native(0, para, ctrl, &properties.to_string())
            .unwrap();
        let frame = body_frame(&core.build_page_render_tree(0).unwrap().root).unwrap();
        assert!(
            near(
                (frame.x - base.x, frame.y - base.y),
                (expected_x * HU, expected_y * HU)
            ),
            "그림자 {kind}, 본체 {frame:?}"
        );
    }
}
