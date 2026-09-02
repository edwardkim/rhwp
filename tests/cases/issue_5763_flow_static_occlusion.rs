//! [#5763] flow 그림 밑에 불투명 채우기가 깔린 쪽은 flow-static 분리 대상이 아니다.
//!
//! flow-static 분리(#516·#3315)는 flow plane 의 `Image`/`RawSvg` 만 canvas **아래** 평면으로
//! 내리고 나머지 본문은 그 **위에** 그린다. 그림을 담은 표 칸의 흰 배경 사각형은 위 평면에
//! 남으므로, 그대로 분리하면 채우기가 그림을 덮어 studio 에서 그림이 빈 흰 상자로 보였다.
//! `FlowStaticOcclusion` 은 paint 순서대로 훑어 그런 쪽을 짚어낸다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::paint::{FlowStaticOcclusion, PaintOp, PaintReplayPlane};
use rhwp::renderer::render_tree::{BoundingBox, ImageNode, RectangleNode};
use rhwp::renderer::ShapeStyle;

fn filled_rect(x: f64, y: f64, w: f64, h: f64) -> PaintOp {
    let style = ShapeStyle {
        fill_color: Some(0x00FF_FFFF),
        ..ShapeStyle::default()
    };
    PaintOp::rectangle(
        BoundingBox::new(x, y, w, h),
        RectangleNode::new(0.0, style, None),
    )
}

fn stroked_rect(x: f64, y: f64, w: f64, h: f64) -> PaintOp {
    let style = ShapeStyle {
        stroke_color: Some(0x0000_0000),
        stroke_width: 1.0,
        ..ShapeStyle::default()
    };
    PaintOp::rectangle(
        BoundingBox::new(x, y, w, h),
        RectangleNode::new(0.0, style, None),
    )
}

fn flow_image(x: f64, y: f64, w: f64, h: f64) -> PaintOp {
    PaintOp::image(
        BoundingBox::new(x, y, w, h),
        ImageNode::new(1, Some(vec![0x89, b'P', b'N', b'G'])),
        None,
    )
}

#[test]
fn issue_5763_opaque_flow_fill_under_image_blocks_static_split() {
    let mut occlusion = FlowStaticOcclusion::default();
    occlusion.observe(
        PaintReplayPlane::Flow,
        &filled_rect(10.0, 10.0, 100.0, 50.0),
    );
    occlusion.observe(PaintReplayPlane::Flow, &flow_image(12.0, 12.0, 90.0, 40.0));
    assert!(
        occlusion.occluded(),
        "그림 밑에 깔린 불투명 채우기는 flow-static 분리를 막아야 한다"
    );
}

#[test]
fn issue_5763_non_overlapping_or_unfilled_shapes_keep_static_split() {
    let mut apart = FlowStaticOcclusion::default();
    apart.observe(PaintReplayPlane::Flow, &filled_rect(10.0, 10.0, 20.0, 20.0));
    apart.observe(
        PaintReplayPlane::Flow,
        &flow_image(200.0, 200.0, 50.0, 50.0),
    );
    assert!(!apart.occluded(), "겹치지 않는 채우기는 그림을 못 덮는다");

    let mut stroke_only = FlowStaticOcclusion::default();
    stroke_only.observe(
        PaintReplayPlane::Flow,
        &stroked_rect(10.0, 10.0, 100.0, 50.0),
    );
    stroke_only.observe(PaintReplayPlane::Flow, &flow_image(12.0, 12.0, 90.0, 40.0));
    assert!(
        !stroke_only.occluded(),
        "테두리만 있는 도형은 밑을 가리지 않는다"
    );
}

#[test]
fn issue_5763_fill_after_image_or_on_other_plane_keeps_static_split() {
    // 그림 뒤에 오는 채우기는 flow-dynamic 에서도 그림 위에 그려진다 — 분리와 무관하다.
    let mut after = FlowStaticOcclusion::default();
    after.observe(PaintReplayPlane::Flow, &flow_image(12.0, 12.0, 90.0, 40.0));
    after.observe(
        PaintReplayPlane::Flow,
        &filled_rect(10.0, 10.0, 100.0, 50.0),
    );
    assert!(!after.occluded());

    // 다른 plane 의 채우기는 별도 overlay layer 로 합성되므로 세지 않는다.
    let mut other_plane = FlowStaticOcclusion::default();
    other_plane.observe(
        PaintReplayPlane::BehindText,
        &filled_rect(10.0, 10.0, 100.0, 50.0),
    );
    other_plane.observe(PaintReplayPlane::Flow, &flow_image(12.0, 12.0, 90.0, 40.0));
    assert!(!other_plane.occluded());
}
