//! 관계형과 고정형. 저장·재열기의 도형 비율과 전단을 한컴의 외접 크기·행렬값과 비교한다.
//! 주석 갱신 2026-11-24T09:49:08+09:00. 직전 파일 변경 99b85699b86b8ffd6257561370854062f87afe1b 2026-11-22T03:06:03+09:00.
//! 2026-11-24 KST, Windows 한컴 Print PDF와 96 DPI, 2px raw 실루엣을 측정했다. 각 입력은 1쪽이다.
//! 원본 Native 99.97% / fresh WASM 99.77%, 너비 확대 99.97% / 99.77%,
//! 복원 99.97% / 99.77%. 원본·크기 편집 대조군 전쪽 최저는 각각 99.97% / 99.77%.
//! 근거: mydocs/pr/assets/semanticist21-9288-9314-20261124/corrected/shape-pages.tsv,
//! pdf/pr9314-review/hancom-width-16000-2024.pdf 및 같은 폴더의 hancom-skew-original-2024.pdf.
//!
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

fn skewed_rectangle(core: &DocumentCore) -> (usize, &rhwp::model::shape::ShapeObject) {
    core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .enumerate()
        .find_map(|(index, control)| match control {
            rhwp::model::control::Control::Shape(shape)
                if matches!(
                    shape.as_ref(),
                    rhwp::model::shape::ShapeObject::Rectangle(_)
                ) && shape.shape_attr().render_b < 0.0 =>
            {
                Some((index, shape.as_ref()))
            }
            _ => None,
        })
        .expect("가로로 기울인 사각형")
}

#[test]
fn resized_skew_frame_matches_the_hancom_edit_after_each_save() {
    let reference = |bytes: &[u8]| DocumentCore::from_bytes(bytes).expect("한컴으로 편집한 대조군");
    for (binary, wider_bytes, restored_bytes) in [
        (
            false,
            include_bytes!("../fixtures/shape_skew/hancom_wider.hwpx").as_slice(),
            include_bytes!("../fixtures/shape_skew/hancom_restored.hwpx").as_slice(),
        ),
        (
            true,
            include_bytes!("../fixtures/shape_skew/hancom_wider.hwp").as_slice(),
            include_bytes!("../fixtures/shape_skew/hancom_restored.hwp").as_slice(),
        ),
    ] {
        let wider = reference(wider_bytes);
        let restored = reference(restored_bytes);
        let mut core = fixture();
        for oracle in [&wider, &restored] {
            let (idx, _) = skewed_rectangle(&core);
            let (_, target) = skewed_rectangle(oracle);
            let width = target.common().width;
            core.set_shape_properties_native(0, 0, idx, &format!(r#"{{"width":{width}}}"#))
                .unwrap();
            let bytes = if binary {
                core.export_hwp_native().unwrap()
            } else {
                core.export_hwpx_native().unwrap()
            };
            core = DocumentCore::from_bytes(&bytes).unwrap();
            let (_, actual) = skewed_rectangle(&core);
            assert_eq!(
                (actual.common().width, actual.common().height),
                (width, target.common().height)
            );
            let got = actual.shape_attr();
            let want = target.shape_attr();
            assert_eq!(got.current_width, want.current_width);
            if !binary {
                assert_eq!(got.current_width_was_zero, want.current_width_was_zero);
                assert_eq!(got.current_height_was_zero, want.current_height_was_zero);
            }
            for (a, b) in [
                (got.render_sx, want.render_sx),
                (got.render_b, want.render_b),
                (got.render_c, want.render_c),
                (got.render_sy, want.render_sy),
                (got.render_tx, want.render_tx),
                (got.render_ty, want.render_ty),
            ] {
                // 한컴 HWPX의 행렬 계수는 소수점 여섯 자리, native HWP는 실수 그대로다.
                assert!((a - b).abs() < 0.001, "저장 행렬 {a} / 한컴 {b}");
            }
        }
    }
}

#[test]
fn rejected_skew_edit_preserves_other_properties_in_both_formats() {
    let mut core = fixture();
    let rect = core.document().sections[0].paragraphs[0].controls.iter().position(|c|
        matches!(c,rhwp::model::control::Control::Shape(s) if matches!(s.as_ref(),rhwp::model::shape::ShapeObject::Rectangle(_)) && s.shape_attr().render_b == 0.0)
    ).unwrap();
    core.set_shape_properties_native(0, 0, rect, r#"{"rotationAngle":30}"#)
        .unwrap();
    let props = |doc: &DocumentCore| {
        let value: serde_json::Value =
            serde_json::from_str(&doc.get_shape_properties_native(0, 0, rect).unwrap()).unwrap();
        [
            value["width"].clone(),
            value["rotationAngle"].clone(),
            value["skewX"].clone(),
        ]
    };
    let before = props(&core);
    assert!(core
        .set_shape_properties_native(0, 0, rect, r#"{"width":14600,"skewX":20}"#)
        .is_err());
    assert_eq!(props(&core), before);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        assert_eq!(props(&DocumentCore::from_bytes(&bytes).unwrap()), before);
    }
}
