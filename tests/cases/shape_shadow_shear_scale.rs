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
