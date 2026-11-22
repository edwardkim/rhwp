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

/// 그림자 행렬과 외곽 상자(한/글 12.30 화면 값).
mod shadow_geometry {
    use rhwp::model::shape::shadow::{body_origin_hu, matrix, transformed_box};

    fn shadow_box(kind: u32, w: f64, h: f64) -> [f64; 4] {
        transformed_box(
            matrix(kind, [0.0, 0.0, w, h], 1.0, (0.0, 0.0)).unwrap(),
            [0.0, 0.0, w, h],
        )
    }

    /// 한/글 12.30이 12000×8000 사각형에 그린 그림자 외곽(HWPUNIT, 본체 왼쪽 위 기준).
    #[test]
    fn preset_shadow_boxes_match_hancom_screen() {
        assert_eq!(
            shadow_box(1, 12000.0, 8000.0),
            [-600.0, -600.0, 12000.0, 8000.0]
        );
        assert_eq!(
            shadow_box(13, 12000.0, 8000.0),
            [-600.0, -600.0, 9000.0, 6000.0]
        );
        assert_eq!(
            shadow_box(14, 12000.0, 8000.0),
            [-3600.0, -2600.0, 15000.0, 10000.0]
        );
        // 뒤쪽 기울임: 아래 변은 본체와 같고 위 변은 반 높이에서 4500만큼 밀린다.
        assert_eq!(
            shadow_box(5, 12000.0, 8000.0),
            [-4500.0, 4000.0, 16500.0, 4000.0]
        );
        assert_eq!(
            shadow_box(6, 12000.0, 8000.0),
            [0.0, 4000.0, 16500.0, 4000.0]
        );
        // 앞쪽 기울임: 본체 아래 변에서 아래로 반 높이.
        assert_eq!(
            shadow_box(7, 12000.0, 8000.0),
            [-4500.0, 8000.0, 16500.0, 4000.0]
        );
        assert_eq!(
            shadow_box(8, 6000.0, 12000.0),
            [0.0, 12000.0, 12750.0, 6000.0]
        );
        assert_eq!(matrix(9, [0.0, 0.0, 1.0, 1.0], 1.0, (0.0, 0.0)), None);
    }

    /// 앞쪽 기울임은 상하로 뒤집힌다: 직각삼각형의 위 꼭짓점이 그림자의 아래 먼 끝으로 간다.
    #[test]
    fn front_shear_mirrors_the_body() {
        let m = matrix(7, [0.0, 0.0, 12000.0, 8000.0], 1.0, (0.0, 0.0)).unwrap();
        let apex = (
            m[0] * 0.0 + m[2] * 0.0 + m[4],
            m[1] * 0.0 + m[3] * 0.0 + m[5],
        );
        assert_eq!(apex, (-4500.0, 12000.0));
    }

    #[test]
    fn body_origin_moves_past_left_and_top_shadow() {
        assert_eq!(
            body_origin_hu(1, 0, 0, 12000.0, 8000.0),
            Some((600.0, 600.0))
        );
        assert_eq!(body_origin_hu(4, 0, 0, 12000.0, 8000.0), Some((0.0, 0.0)));
        assert_eq!(
            body_origin_hu(14, 0, 0, 12000.0, 8000.0),
            Some((3600.0, 2600.0))
        );
        assert_eq!(
            body_origin_hu(7, 0, 0, 12000.0, 8000.0),
            Some((4500.0, 0.0))
        );
        assert_eq!(
            body_origin_hu(1, 100, -200, 12000.0, 8000.0),
            Some((500.0, 800.0))
        );
    }
}
