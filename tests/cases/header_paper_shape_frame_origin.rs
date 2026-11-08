//! 머리말 안 용지 기준(`PAPER`) 도형은 머리말 틀의 원점에서 오프셋을 잰다.
//!
//! 한/글은 머리말·꼬리말 안 개체의 오프셋을 그 틀의 원점에서 잰다 — 그림은 #6608 에서
//! 맞췄는데 도형은 물리 용지 (0, 0) 에서 재어, 음수 세로 오프셋 사각형이 용지 위로 잘렸다
//! (국토부 보도자료 머리말 표지: `vertOffset=-1678` HU, 한/글 미리보기에서는 머리말 틀
//! 왼쪽 위에 온전히 보인다).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::shape::{HorzRelTo, VertRelTo};
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const OFFSET_Y_HU: i32 = -1000;

fn header_rect(node: &RenderNode, in_header: bool) -> Option<BoundingBox> {
    let in_header = in_header || matches!(node.node_type, RenderNodeType::Header);
    if in_header && matches!(node.node_type, RenderNodeType::Rectangle(_)) {
        return Some(node.bbox.clone());
    }
    node.children
        .iter()
        .find_map(|child| header_rect(child, in_header))
}

#[test]
fn header_paper_relative_rect_measures_its_offset_from_the_header_frame() {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc.create_header_footer_native(0, true, 0).unwrap();
    // 본문에 만든 사각형을 머리말 문단에 복사해 용지 기준 음수 오프셋을 준다.
    doc.create_shape_control_native(
        0,
        0,
        0,
        8000,
        3000,
        0,
        0,
        false,
        "InFrontOfText",
        "rectangle",
        false,
        false,
        &[],
    )
    .unwrap();
    let mut model = doc.document().clone();
    let body = &mut model.sections[0].paragraphs[0];
    let mut shape = body
        .controls
        .iter()
        .find_map(|c| match c {
            Control::Shape(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap();
    let common = shape.common_mut();
    common.horz_rel_to = HorzRelTo::Paper;
    common.vert_rel_to = VertRelTo::Paper;
    common.horizontal_offset = 0;
    common.vertical_offset = OFFSET_Y_HU as u32;
    let header = body
        .controls
        .iter_mut()
        .find_map(|c| match c {
            Control::Header(h) => Some(h),
            _ => None,
        })
        .unwrap();
    header.paragraphs[0].controls.push(Control::Shape(shape));
    doc.set_document(model);

    let tree = doc.build_page_render_tree(0).unwrap();
    let rect = header_rect(&tree.root, false).expect("머리말 사각형");
    // 빈 문서: 왼쪽 여백 30mm, 위쪽 여백 20mm 에서 머리말 틀이 시작한다.
    let mm = |v: f64| v * 96.0 / 25.4;
    let expected_y = mm(20.0) + f64::from(OFFSET_Y_HU) * 96.0 / 7200.0;
    assert!(
        (rect.x - mm(30.0)).abs() < 0.5 && (rect.y - expected_y).abs() < 0.5,
        "사각형은 머리말 틀 ({:.1}, {expected_y:.1}) 기준이어야 한다: {rect:?}",
        mm(30.0)
    );
}
