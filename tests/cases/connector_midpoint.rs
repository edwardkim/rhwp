//! 관계형 검사. 꺾인·곡선 연결선의 가운데 점을 옮긴 뒤 제어점·크기·위치·저장 결과를 입력과 대조한다.
//! 주석 갱신 2026-11-22T20:30:00+09:00.
//! 실물 인쇄 좌표·픽셀을 대상으로 하는 검사는 없다.
//!
//! 한/글 12.30(Mac)은 고른 꺾인·곡선 연결선의 가운데에 노란 마름모를 두고, 끌면 안쪽 제어점 둘을
//! 가운데 선에 수직인 한 축으로만 옮긴다. 끝점·크기·위치는 그대로이고 끝점 상자 밖으로도 옮긴다.
//! 이은 도형을 옮기면 연결선을 다시 이어 가운데 점이 처음 자리로 돌아간다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::shape::{ConnectorData, ShapeObject};
use rhwp::wasm_api::HwpDocument;

/// 위·가운데·아래 사각형과, 위 사각형 오른쪽(1)에서 가운데 사각형 `end_index` 연결점으로 잇는 연결선.
fn connected(kind: &str, end_index: u32) -> (HwpDocument, usize, usize, Vec<u32>) {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    let mut ids = Vec::new();
    for (x, y) in [(5000, 5000), (30000, 22000), (5000, 40000)] {
        let made: serde_json::Value = serde_json::from_str(
            &doc.create_shape_control_native(
                0,
                0,
                0,
                6000,
                4500,
                x,
                y,
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
        let (para, ctrl) = (
            made["paraIdx"].as_u64().unwrap() as usize,
            made["controlIdx"].as_u64().unwrap() as usize,
        );
        let Control::Shape(shape) = &doc.document().sections[0].paragraphs[para].controls[ctrl]
        else {
            panic!("사각형이 아니다");
        };
        ids.push(shape.drawing().unwrap().inst_id);
    }
    let made: serde_json::Value = serde_json::from_str(
        &doc.create_shape_control_native(
            0,
            0,
            0,
            19000,
            17000,
            11000,
            7250,
            false,
            "InFrontOfText",
            kind,
            false,
            false,
            &[],
        )
        .unwrap(),
    )
    .unwrap();
    let (para, ctrl) = (
        made["paraIdx"].as_u64().unwrap() as usize,
        made["controlIdx"].as_u64().unwrap() as usize,
    );
    doc.update_connector_subject_ids(0, para, ctrl, ids[0], 1, ids[1], end_index);
    doc.recalculate_connector_routing(0, para, ctrl, 1, end_index);
    doc.update_connectors_in_section(0);
    (doc, para, ctrl, ids)
}

fn connector(core: &DocumentCore, para: usize, ctrl: usize) -> (ConnectorData, [u32; 4]) {
    let Control::Shape(shape) = &core.document().sections[0].paragraphs[para].controls[ctrl] else {
        panic!("연결선이 아니다");
    };
    let ShapeObject::Line(line) = shape.as_ref() else {
        panic!("연결선이 아니다");
    };
    let c = &line.common;
    (
        line.connector.clone().expect("연결 정보가 없다"),
        [c.horizontal_offset, c.vertical_offset, c.width, c.height],
    )
}

fn inner(conn: &ConnectorData) -> Vec<(i32, i32)> {
    conn.control_points[1..3]
        .iter()
        .map(|p| (p.x, p.y))
        .collect()
}

#[test]
fn midpoint_moves_inner_points_along_its_axis() {
    for kind in ["connector-stroke", "connector-arc"] {
        let (mut doc, para, ctrl, _) = connected(kind, 3);
        let (before, frame) = connector(&doc, para, ctrl);
        let props = doc.get_shape_properties_native(0, para, ctrl).unwrap();
        assert!(
            props.contains(r#""connectorMidAxis":"x""#),
            "{kind}: {props}"
        );
        // 끝점 상자(너비 19000) 밖까지 끈다. 세로 거리는 가운데 선 축이 아니라 쓰지 않는다.
        assert_eq!(
            doc.move_connector_midpoint_native(0, para, ctrl, 30000, 700)
                .unwrap(),
            r#"{"ok":true}"#
        );
        let (moved, moved_frame) = connector(&doc, para, ctrl);
        let expected: Vec<_> = inner(&before)
            .iter()
            .map(|&(x, y)| (x + 30000, y))
            .collect();
        assert_eq!(inner(&moved), expected, "{kind}");
        assert_eq!(moved_frame, frame, "{kind}: 크기·위치는 그대로다");
        assert_eq!(moved.control_points[0].x, before.control_points[0].x);
        assert_eq!(moved.control_points[3].x, before.control_points[3].x);
        for bytes in [
            doc.export_hwp_native().unwrap(),
            doc.export_hwpx_native().unwrap(),
        ] {
            let reopened = DocumentCore::from_bytes(&bytes).unwrap();
            assert_eq!(
                inner(&connector(&reopened, para, ctrl).0),
                expected,
                "{kind}"
            );
        }
    }
}

#[test]
fn unrelated_connector_update_keeps_a_moved_midpoint() {
    // 한/글 저장본처럼 가운데 점을 옮긴 연결선은, 이은 도형이 그대로면 연결선 갱신 뒤에도 그 자리에 있다.
    let (mut doc, para, ctrl, _) = connected("connector-stroke", 3);
    // 한/글이 저장한 것처럼 안쪽 제어점 둘을 오른쪽으로 옮겨 둔다.
    let Control::Shape(shape) = &mut doc.document_mut().sections[0].paragraphs[para].controls[ctrl]
    else {
        panic!("연결선이 아니다");
    };
    let ShapeObject::Line(line) = shape.as_mut() else {
        panic!("연결선이 아니다");
    };
    for point in &mut line.connector.as_mut().unwrap().control_points[1..3] {
        point.x += 5000;
    }
    let moved = inner(&connector(&doc, para, ctrl).0);
    let mut reopened = DocumentCore::from_bytes(&doc.export_hwp_native().unwrap()).unwrap();
    reopened.update_connectors_in_section(0);
    assert_eq!(inner(&connector(&reopened, para, ctrl).0), moved);
}

#[test]
fn moving_a_connected_shape_reroutes_the_connector() {
    let (mut doc, para, ctrl, ids) = connected("connector-stroke", 3);
    let (routed, [_, _, width, _]) = connector(&doc, para, ctrl);
    doc.move_connector_midpoint_native(0, para, ctrl, 5000, 0)
        .unwrap();
    // 가운데 사각형을 오른쪽으로 750 옮기면 끝점이 바뀌어 가운데 선이 새 두 끝점의 가운데로 돌아간다.
    let middle = doc.document_mut().sections[0]
        .paragraphs
        .iter_mut()
        .flat_map(|p| p.controls.iter_mut())
        .find_map(|c| match c {
            Control::Shape(s) if s.drawing().is_some_and(|d| d.inst_id == ids[1]) => Some(s),
            _ => None,
        })
        .expect("가운데 사각형이 없다");
    middle.common_mut().horizontal_offset += 750;
    doc.update_connectors_in_section(0);
    let (rerouted, [_, _, new_width, _]) = connector(&doc, para, ctrl);
    assert_eq!(new_width, width + 750);
    assert_eq!(rerouted.control_points[1].x, (new_width / 2) as i32);
    assert_eq!(rerouted.control_points[1].y, routed.control_points[1].y);
}

#[test]
fn single_bend_connector_has_no_midpoint() {
    // 오른쪽에서 위 연결점으로 잇는 꺾인 연결선은 꺾임이 하나라 한/글도 가운데 마름모를 두지 않는다.
    let (mut doc, para, ctrl, _) = connected("connector-stroke", 0);
    let props = doc.get_shape_properties_native(0, para, ctrl).unwrap();
    assert!(!props.contains("connectorMidAxis"), "{props}");
    let before = connector(&doc, para, ctrl).0.control_points.len();
    assert!(doc
        .move_connector_midpoint_native(0, para, ctrl, 3000, 3000)
        .is_err());
    assert_eq!(connector(&doc, para, ctrl).0.control_points.len(), before);
}
