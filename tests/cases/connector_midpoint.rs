//! 관계형(제어점·끝점·인쇄 경로), 고정형(독립 인쇄의 ¼타원 접선 비율).
//! 주석 갱신 2026-11-24T01:49:33+09:00 / 직전 파일 변경 6f4ded68c7391bf0497930230ef9aeba1c8ed29e, 2026-11-23T22:19:57+09:00.
//! 2026-11-24, Windows 한컴 Print 대비 96 DPI 2px raw 실루엣. 꺾임 0개·1개 원본은 각 1쪽 Native/fresh WASM 100.00%, 나머지 10개 연결선도 전쪽 100.00%다.
//! 전쪽 측정과 원본은 `mydocs/pr/assets/semanticist21-20261124-rework/final/`에 기록한다.
//!
//! 관계형 검사. 꺾인·곡선 연결선의 가운데 점을 옮긴 뒤 제어점·크기·위치·저장 결과를 입력과 대조한다.
//! 주석 갱신 2026-11-23T16:00:00+09:00.
//! 곡선 모양과 셀 높이는 한/글 12.30(Mac) 인쇄 PDF에서 잰 규칙과 대조한다. 픽셀 값은 옮기지 않았다.
//!
//! 한/글 인쇄는 가운데 점이 있는 곡선 연결선을 시작점→가운데 점→끝점의 4분의 1 타원 두 개로 그린다
//! (안쪽 제어점이 각 타원의 모서리). 셀 안 종이 기준 글 앞 연결선·도형은 칸을 키우지 않아,
//! 칸은 글자 줄 높이 그대로이고 표 뒤 본문도 제자리에 놓인다.
//!
//! 한/글 12.30(Mac)은 고른 꺾인·곡선 연결선의 가운데에 노란 마름모를 두고, 끌면 안쪽 제어점 둘을
//! 가운데 선에 수직인 한 축으로만 옮긴다. 끝점·크기·위치는 그대로이고 끝점 상자 밖으로도 옮긴다.
//! 이은 도형을 옮기면 연결선을 다시 이어 가운데 점이 처음 자리로 돌아간다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::shape::{ConnectorData, ShapeObject};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::renderer::PathCommand;
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

fn find<'a>(node: &'a RenderNode, hit: &dyn Fn(&RenderNode) -> bool) -> Option<&'a RenderNode> {
    if hit(node) {
        return Some(node);
    }
    node.children.iter().find_map(|child| find(child, hit))
}

#[test]
fn moved_arc_midpoint_is_on_two_quarter_ellipses() {
    let (mut doc, para, ctrl, _) = connected("connector-arc", 3);
    doc.move_connector_midpoint_native(0, para, ctrl, 3000, 0)
        .unwrap();
    let (conn, [x, y, _, _]) = connector(&doc, para, ctrl);
    let px = |hu: i32| f64::from(hu) / 75.0;
    let [_, a, b, _] = conn.control_points.as_slice() else {
        panic!("제어점이 넷이 아니다");
    };
    let mid = (
        px(x as i32 + (a.x + b.x) / 2),
        px(y as i32 + (a.y + b.y) / 2),
    );
    for core in [
        DocumentCore::from_bytes(&doc.export_hwp_native().unwrap()).unwrap(),
        DocumentCore::from_bytes(&doc.export_hwpx_native().unwrap()).unwrap(),
    ] {
        let tree = core.build_page_render_tree(0).unwrap();
        let node = find(&tree.root, &|n| {
            matches!(n.node_type, RenderNodeType::Path(_))
        })
        .expect("연결선 경로가 없다");
        let RenderNodeType::Path(path) = &node.node_type else {
            unreachable!()
        };
        let [PathCommand::MoveTo(sx, sy), PathCommand::CurveTo(_, _, c2x, _, mx, my), PathCommand::CurveTo(d1x, _, _, _, _, _)] =
            path.commands.as_slice()
        else {
            panic!("두 구간 곡선이 아니다: {:?}", path.commands);
        };
        // 곡선은 옮긴 가운데 점을 지나고, 거기서 가운데 선(세로) 방향으로 꺾임 없이 이어진다.
        assert!(
            (mx - mid.0).abs() < 0.01 && (my - mid.1).abs() < 0.01,
            "{:?}",
            path.commands
        );
        assert!((c2x - mx).abs() < 1e-9 && (d1x - mx).abs() < 1e-9);
        // 각 구간은 끝점과 가운데 점을 잇는 4분의 1 타원이다(중심은 모서리의 맞은편).
        let PathCommand::CurveTo(c1x, c1y, c2x, c2y, _, _) = path.commands[1] else {
            unreachable!()
        };
        let t = 0.5_f64;
        let bez = |p0: f64, p1: f64, p2: f64, p3: f64| {
            (1.0 - t).powi(3) * p0
                + 3.0 * (1.0 - t).powi(2) * t * p1
                + 3.0 * (1.0 - t) * t * t * p2
                + t.powi(3) * p3
        };
        let (qx, qy) = (bez(*sx, c1x, c2x, *mx), bez(*sy, c1y, c2y, *my));
        let on_ellipse = ((qx - sx) / (mx - sx)).powi(2) + ((qy - my) / (my - sy)).powi(2);
        assert!((on_ellipse - 1.0).abs() < 1e-3, "{on_ellipse}");
    }
}

/// 위 사각형·가운데 사각형·곡선 연결선을 1×1 표 칸에 옮긴 문서(`with_shapes`가 거짓이면 지우기만 한다)와
/// 표 뒤 본문 문단.
fn cell_document(with_shapes: bool) -> HwpDocument {
    let (mut doc, ..) = connected("connector-arc", 3);
    let made: serde_json::Value =
        serde_json::from_str(&doc.create_table_native(0, 0, 0, 1, 1).unwrap()).unwrap();
    let table_para = made["paraIdx"].as_u64().unwrap() as usize;
    let table_ctrl = made["controlIdx"].as_u64().unwrap() as usize;
    let shapes: Vec<usize> = doc.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .enumerate()
        .filter(|(_, c)| matches!(c, Control::Shape(_)))
        .map(|(i, _)| i)
        .collect();
    // 위·가운데 사각형과 연결선. 아래 사각형은 본문에 남긴다.
    let moved = [shapes[0], shapes[1], shapes[3]];
    if with_shapes {
        for &ci in &moved {
            doc.copy_control_native(0, 0, &[], ci).unwrap();
            doc.paste_internal_in_cell_by_path_native(0, table_para, &[(table_ctrl, 0, 0)], 0)
                .unwrap();
        }
    }
    for &ci in moved.iter().rev() {
        doc.delete_shape_control_native(0, 0, ci).unwrap();
    }
    let last = doc.document().sections[0].paragraphs.len() - 1;
    doc.insert_text_native(0, last, 0, "표 뒤 본문").unwrap();
    doc
}

#[test]
fn floating_connector_in_a_cell_keeps_the_row_and_following_text() {
    let place = |core: &DocumentCore| {
        let tree = core.build_page_render_tree(0).unwrap();
        let table = find(&tree.root, &|n| {
            matches!(n.node_type, RenderNodeType::Table(_))
        })
        .expect("표가 없다");
        let text = find(
            &tree.root,
            &|n| matches!(&n.node_type, RenderNodeType::TextRun(run) if run.text.contains('뒤')),
        )
        .expect("표 뒤 본문이 없다");
        (table.bbox.height, text.bbox.y)
    };
    let (with, empty) = (cell_document(true), cell_document(false));
    let tree = with.build_page_render_tree(0).unwrap();
    assert!(
        find(&tree.root, &|n| matches!(
            n.node_type,
            RenderNodeType::Path(_)
        ))
        .is_some(),
        "셀 안 연결선이 그려지지 않는다"
    );
    // 한/글 인쇄는 칸을 키우지 않은 채 연결선·도형을 칸 밖까지 그린다. 칸 클립 아래에 두면
    // 한 줄 높이 칸에 잘려 사라진다.
    fn path_under_cell_clip(node: &RenderNode, clipped: bool) -> bool {
        let clipped =
            clipped || matches!(&node.node_type, RenderNodeType::TableCell(cell) if cell.clip);
        (clipped && matches!(node.node_type, RenderNodeType::Path(_)))
            || node
                .children
                .iter()
                .any(|child| path_under_cell_clip(child, clipped))
    }
    assert!(
        !path_under_cell_clip(&tree.root, false),
        "셀 안 연결선이 칸 클립에 잘린다"
    );
    // 연결선·도형이 든 칸과 빈 칸의 표 높이·뒤 본문 자리가 같다(편집 직후, HWP·HWPX로 다시 연 문서).
    assert_eq!(place(&with), place(&empty), "편집 직후");
    for hwpx in [false, true] {
        let reopen = |doc: &HwpDocument| {
            let bytes = if hwpx {
                doc.export_hwpx_native()
            } else {
                doc.export_hwp_native()
            };
            DocumentCore::from_bytes(&bytes.unwrap()).unwrap()
        };
        assert_eq!(place(&reopen(&with)), place(&reopen(&empty)), "HWPX {hwpx}");
    }
}

#[test]
fn arcs_with_zero_or_one_bend_preserve_printed_paths_and_endpoints() {
    for (file, bend) in [("one-elbow.hwpx", true), ("no-elbow.hwpx", false)] {
        let bytes = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("samples/semanticist21-20261124-rework")
                .join(file),
        )
        .unwrap();
        let original = DocumentCore::from_bytes(&bytes).unwrap();
        let serialized = [
            bytes,
            original.export_hwp_native().unwrap(),
            original.export_hwpx_native().unwrap(),
        ];
        for data in serialized {
            let doc = DocumentCore::from_bytes(&data).unwrap();
            let tree = doc.build_page_render_tree(0).unwrap();
            let node=find(&tree.root,&|n|matches!(&n.node_type,RenderNodeType::Path(p) if p.connector_endpoints.is_some())).expect("원본 연결선");
            let RenderNodeType::Path(path) = &node.node_type else {
                unreachable!()
            };
            let (sx, sy, ex, ey) = path.connector_endpoints.unwrap();
            assert!(sx != ex && sy != ey, "양 끝점이 보존되어야 한다");
            if bend {
                let [PathCommand::MoveTo(x, y), PathCommand::CurveTo(c1x, c1y, c2x, c2y, end_x, end_y)] =
                    path.commands.as_slice()
                else {
                    panic!("한컴 인쇄의 단일 ¼타원")
                };
                assert_eq!((*x, *y, *end_x, *end_y), (sx, sy, ex, ey));
                assert!((c1x - sx).abs() < 0.001 && (c2y - ey).abs() < 0.001);
                // 한컴 PDF에서 추출한 양 끝 접선 길이 / 각 축의 반지름.
                assert!(((c1y - sy) / (ey - sy) - 0.553).abs() < 0.003);
                assert!(((ex - c2x) / (ex - sx) - 0.552).abs() < 0.003);
                assert!(path.style.stroke_color.is_some());
            } else {
                assert!(matches!(
                    path.commands.as_slice(),
                    [PathCommand::MoveTo(_, _)]
                ));
                assert!(path.style.stroke_color.is_none() && path.line_style.is_none());
            }
        }
    }
}
