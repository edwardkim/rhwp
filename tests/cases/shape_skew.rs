//! 한/글 개체 속성의 가로·세로 기울이기. 한/글 12.30은 배율 행렬에 -tan 전단을, 이동 행렬과
//! 개체 offset에 기울인 모양의 외접 상자 왼쪽 위를 적고, 개체 크기를 외접 상자로 둔다.
//! 표본은 한/글 12.30에서 사각형을 가로 20도, 타원을 세로 20도 기울여 저장한 HWPX다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::paint::{GroupKind, LayerBuilder, LayerNode, LayerNodeKind, RenderProfile};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn fixture() -> DocumentCore {
    let bytes = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/shape_skew/hancom_skew.hwpx"),
    )
    .expect("fixture");
    DocumentCore::from_bytes(&bytes).expect("open")
}

/// 사각형·타원 노드의 그린 상자(너비, 높이)
fn shape_boxes(node: &RenderNode, out: &mut Vec<(f64, f64)>) {
    // 쪽 배경 같은 큰 사각형은 빼고 표본 도형만 본다.
    if matches!(
        node.node_type,
        RenderNodeType::Rectangle(_) | RenderNodeType::Ellipse(_)
    ) && node.bbox.width < 300.0
    {
        out.push((node.bbox.width, node.bbox.height));
    }
    node.children
        .iter()
        .for_each(|child| shape_boxes(child, out));
}

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

const HU: f64 = 96.0 / 7200.0;
const TAN20: f64 = 0.363_970_234_266_202_4;

#[test]
fn hancom_skewed_shapes_draw_at_their_original_size_and_lean() {
    let core = fixture();
    let tree = core.build_page_render_tree(0).expect("page");
    // 표본의 사각형 둘·타원은 모두 원래 크기 12000×8000으로 그린다(기울인 것도 외접 상자로 늘리지 않는다).
    let mut boxes = Vec::new();
    shape_boxes(&tree.root, &mut boxes);
    assert_eq!(boxes.len(), 3, "{boxes:?}");
    for (w, h) in &boxes {
        assert!(
            (w - 12000.0 * HU).abs() < 0.01 && (h - 8000.0 * HU).abs() < 0.01,
            "{boxes:?}"
        );
    }
    // 가로 기울이기는 위 변을 오른쪽으로(x' = x − tan·y + …), 세로 기울이기는 왼쪽 변을 아래로 민다.
    let layers = LayerBuilder::new(RenderProfile::Screen).build(&tree);
    let mut found = Vec::new();
    transforms(&layers.root, &mut found);
    let horizontal = found
        .iter()
        .any(|m| (m[2] + TAN20).abs() < 1e-4 && m[1].abs() < 1e-9 && m[0] == 1.0);
    let vertical = found
        .iter()
        .any(|m| (m[1] + TAN20).abs() < 1e-4 && m[2].abs() < 1e-9 && m[3] == 1.0);
    assert!(horizontal && vertical, "{found:?}");
}

#[test]
fn skew_setter_writes_hancom_matrices_and_survives_reopen() {
    let mut core = fixture();
    let rect = core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|c| {
            matches!(c, rhwp::model::control::Control::Shape(s)
                if s.common().horizontal_offset == 5000 && s.common().vertical_offset == 5000)
        })
        .expect("기울이지 않은 사각형");
    core.set_shape_properties_native(0, 0, rect, r#"{"skewX":20}"#)
        .expect("skew");
    let props = core.get_shape_properties_native(0, 0, rect).expect("props");
    assert!(props.contains("\"skewX\":20,"), "{props}");
    assert!(
        props.contains("\"width\":14912,"),
        "외접 상자 너비: {props}"
    );
    for bytes in [
        core.export_hwpx_native().expect("hwpx"),
        core.export_hwp_native().expect("hwp"),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).expect("reopen");
        let props = reopened
            .get_shape_properties_native(0, 0, rect)
            .expect("props");
        assert!(
            props.contains("\"skewX\":20,") && props.contains("\"skewY\":0,"),
            "{props}"
        );
    }
    core.set_shape_properties_native(0, 0, rect, r#"{"skewX":0}"#)
        .expect("unskew");
    let props = core.get_shape_properties_native(0, 0, rect).expect("props");
    assert!(
        props.contains("\"width\":12000,") && props.contains("\"skewX\":0,"),
        "{props}"
    );
}
