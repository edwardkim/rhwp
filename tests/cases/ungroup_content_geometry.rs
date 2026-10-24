//! 묶음 해제는 이웃 필드와 자식의 최종 그리기 위치를 바꾸지 않아야 한다.
//! 공개 편집 API로 만든 HWP를 다시 연 뒤 검사하며 조판 정답 좌표를 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::shape::ShapeObject;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::renderer::PathCommand;
use serde_json::{json, Value};

fn fixture(rotation: u16, field: bool) -> DocumentCore {
    let mut doc = DocumentCore::new_empty();
    doc.create_blank_document_native().unwrap();
    doc.insert_text_native(0, 0, 0, "앞쪽 가나다 뒤쪽").unwrap();
    if field {
        doc.insert_click_here_field_at(0, 0, 5, "안내문", "메모", "남을 필드", true)
            .unwrap();
    }
    let mut targets = Vec::new();
    for i in 0..2 {
        let result: Value = serde_json::from_str(
            &doc.create_shape_control_native(
                0,
                0,
                2,
                4500,
                3000,
                15000 + i * 6000,
                15000,
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
        let ci = result["controlIdx"].as_u64().unwrap() as usize;
        doc.set_shape_properties_native(
            0,
            0,
            ci,
            &json!({"fillBgColor": if i == 0 { 255 } else { 65280 },
                    "rotationAngle": if i == 0 { rotation } else { 0 }})
            .to_string(),
        )
        .unwrap();
        targets.push((0, ci));
    }
    doc.group_shapes_native(0, &targets).unwrap();
    DocumentCore::from_bytes(&doc.export_hwp_native().unwrap()).unwrap()
}

fn group_index(doc: &DocumentCore) -> usize {
    doc.document().sections[0].paragraphs[0]
        .controls.iter().position(|control| {
            matches!(control, Control::Shape(shape) if matches!(shape.as_ref(), ShapeObject::Group(_)))
        }).expect("저장본에 묶음 한 개")
}

fn fields(doc: &DocumentCore) -> Vec<Value> {
    let fields: Vec<Value> = serde_json::from_str(&doc.get_field_list_json()).unwrap();
    fields
        .into_iter()
        .map(|field| {
            json!({
                "id": field["fieldId"], "name": field["name"], "guide": field["guide"],
                "value": field["value"], "start": field["startCharIdx"], "end": field["endCharIdx"],
                "editable": field["editableInForm"],
            })
        })
        .collect()
}

// 빨간 사각형은 그룹 안에서는 Path, 밖에서는 회전 transform을 가진 Rectangle일 수 있다.
// 노드 종류나 SVG 문자열 대신 paint가 소비하는 꼭짓점의 최종 좌표를 비교한다.
fn red_bounds(doc: &DocumentCore) -> [f64; 4] {
    fn visit(node: &RenderNode, out: &mut Vec<(f64, f64)>) {
        let (points, transform) = match &node.node_type {
            RenderNodeType::Rectangle(rect) if rect.style.fill_color == Some(255) => {
                let b = node.bbox;
                (
                    vec![
                        (b.x, b.y),
                        (b.x + b.width, b.y),
                        (b.x + b.width, b.y + b.height),
                        (b.x, b.y + b.height),
                    ],
                    rect.transform,
                )
            }
            RenderNodeType::Path(path) if path.style.fill_color == Some(255) => {
                let points = path
                    .commands
                    .iter()
                    .filter_map(|command| match command {
                        PathCommand::MoveTo(x, y) | PathCommand::LineTo(x, y) => Some((*x, *y)),
                        PathCommand::ClosePath => None,
                        _ => panic!("합성 사각형에 곡선이 없어야 한다"),
                    })
                    .collect();
                (points, path.transform)
            }
            _ => {
                for child in &node.children {
                    visit(child, out);
                }
                return;
            }
        };
        assert!(!transform.horz_flip && !transform.vert_flip);
        let cx = node.bbox.x + node.bbox.width / 2.0;
        let cy = node.bbox.y + node.bbox.height / 2.0;
        let (sin, cos) = transform.rotation.to_radians().sin_cos();
        out.extend(points.into_iter().map(|(x, y)| {
            (
                cx + (x - cx) * cos - (y - cy) * sin,
                cy + (x - cx) * sin + (y - cy) * cos,
            )
        }));
    }
    let tree = doc.build_page_render_tree(0).unwrap();
    let mut points = Vec::new();
    visit(&tree.root, &mut points);
    assert_eq!(
        points.len(),
        4,
        "실제 출력에 빨간 사각형 하나가 있어야 한다"
    );
    points.iter().fold(
        [
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        ],
        |mut bounds, &(x, y)| {
            bounds[0] = bounds[0].min(x);
            bounds[1] = bounds[1].min(y);
            bounds[2] = bounds[2].max(x);
            bounds[3] = bounds[3].max(y);
            bounds
        },
    )
}

fn check_geometry(rotation: u16) {
    let mut doc = fixture(rotation, false);
    let before = red_bounds(&doc);
    let text = doc.document().sections[0].paragraphs[0].text.clone();
    let snap = doc.save_snapshot_native();
    doc.ungroup_shape_native(0, 0, group_index(&doc)).unwrap();
    let live = red_bounds(&doc);
    let hwp = DocumentCore::from_bytes(&doc.export_hwp_native().unwrap()).unwrap();
    let hwpx = DocumentCore::from_bytes(&doc.export_hwpx_native().unwrap()).unwrap();
    let saved = [red_bounds(&hwp), red_bounds(&hwpx)];
    assert_eq!(doc.document().sections[0].paragraphs[0].text, text);
    doc.restore_snapshot_native(snap).unwrap();
    assert_eq!(red_bounds(&doc), before, "snapshot은 그룹 위치를 복원한다");
    for after in [live, saved[0], saved[1]] {
        assert!(
            before.iter().zip(after).all(|(a, b)| (a - b).abs() < 0.05),
            "묶음을 풀어도 자식 위치가 같아야 한다: before={before:?}, after={after:?}"
        );
    }
}

#[test]
fn ungroup_plain_child_keeps_drawn_position_and_roundtrips() {
    check_geometry(0);
}

#[test]
fn ungroup_rotated_child_keeps_drawn_position_and_roundtrips() {
    check_geometry(30);
}

#[test]
fn ungroup_keeps_following_field_live_and_in_both_formats() {
    let mut doc = fixture(0, true);
    let group = group_index(&doc);
    let controls = &doc.document().sections[0].paragraphs[0].controls;
    let field = controls
        .iter()
        .position(|c| matches!(c, Control::Field(_)))
        .unwrap();
    assert!(group < field, "저장본에서 필드가 해제할 그룹 뒤에 있다");
    let before = fields(&doc);
    assert_eq!(before.len(), 1);
    let text = doc.document().sections[0].paragraphs[0].text.clone();
    let snap = doc.save_snapshot_native();
    doc.ungroup_shape_native(0, 0, group).unwrap();
    let live = fields(&doc);
    let hwp = fields(&DocumentCore::from_bytes(&doc.export_hwp_native().unwrap()).unwrap());
    let hwpx = fields(&DocumentCore::from_bytes(&doc.export_hwpx_native().unwrap()).unwrap());
    assert_eq!(doc.document().sections[0].paragraphs[0].text, text);
    doc.restore_snapshot_native(snap).unwrap();
    assert_eq!(fields(&doc), before);
    assert_eq!(
        [live, hwp, hwpx],
        [before.clone(), before.clone(), before],
        "그룹 하나가 자식 둘로 바뀌어도 뒤 필드의 소속·값을 보존해야 한다"
    );
}
