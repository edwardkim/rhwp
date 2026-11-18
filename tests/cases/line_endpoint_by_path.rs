//! 표 셀·글상자·머리말 안 직선의 끝점을 경로로 옮긴다(`moveLineEndpointByPath`).
//!
//! 한컴 12.30은 머리말을 고치며 고른 머리말 직선에도 끝점 손잡이를 두고 끌어 옮긴다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::ShapeObject;
use rhwp::wasm_api::HwpDocument;

fn blank() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc
}

fn shapes(paragraphs: &[Paragraph]) -> Vec<&ShapeObject> {
    paragraphs
        .iter()
        .flat_map(|p| &p.controls)
        .filter_map(|c| match c {
            Control::Shape(s) => Some(s.as_ref()),
            _ => None,
        })
        .collect()
}

/// 복사한 개체를 `path`가 가리키는 문단(표 셀·머리말) 첫 자리에 붙인다.
fn paste_copied(doc: &mut HwpDocument, host: usize, path: &[(usize, usize, usize)]) {
    let pasted = doc
        .paste_internal_in_cell_by_path_native(0, host, path, 0)
        .unwrap();
    assert!(pasted.contains("\"ok\":true"), "{pasted}");
}

#[test]
fn line_endpoints_move_in_a_header() {
    // 한컴 12.30은 머리말을 고치며 고른 머리말 직선의 끝점을 끌어 옮긴다.
    let mut doc = blank();
    let line = doc
        .create_shape_control_native(
            0,
            0,
            0,
            8000,
            3000,
            5000,
            6000,
            false,
            "InFrontOfText",
            "line",
            false,
            false,
            &[],
        )
        .unwrap();
    let line: serde_json::Value = serde_json::from_str(&line).unwrap();
    let (para, ctrl) = (
        line["paraIdx"].as_u64().unwrap() as usize,
        line["controlIdx"].as_u64().unwrap() as usize,
    );
    doc.copy_control_native(0, para, &[], ctrl).unwrap();
    let header: serde_json::Value =
        serde_json::from_str(&doc.create_header_footer_native(0, true, 0).unwrap()).unwrap();
    let host = header["paraIndex"].as_u64().unwrap() as usize;
    let header_ctrl = header["controlIndex"].as_u64().unwrap() as usize;
    paste_copied(&mut doc, host, &[(header_ctrl, 0, 0)]);
    let path = format!(r#"[{{"controlIndex":{header_ctrl},"cellIndex":0,"cellParaIndex":0}}]"#);
    // 끝점을 왼쪽 아래로 끌어 선의 방향을 바꾼다.
    assert_eq!(
        doc.move_line_endpoint_by_path_native(0, host, &path, 0, 9000, 1000, 2000, 5000)
            .unwrap(),
        r#"{"ok":true}"#
    );
    let header_line = |core: &DocumentCore| -> rhwp::model::shape::LineShape {
        let header = core.document().sections[0]
            .paragraphs
            .iter()
            .flat_map(|p| &p.controls)
            .find_map(|c| match c {
                Control::Header(h) => Some(h),
                _ => None,
            })
            .expect("머리말이 없다");
        match shapes(&header.paragraphs)[0] {
            ShapeObject::Line(l) => l.clone(),
            _ => panic!("머리말 직선이 아니다"),
        }
    };
    let moved = header_line(&doc);
    assert_eq!(
        (
            moved.common.horizontal_offset,
            moved.common.vertical_offset,
            moved.common.width,
            moved.common.height
        ),
        (2000, 1000, 7000, 4000)
    );
    let (start, end) = if moved.started_right_or_bottom {
        (moved.end, moved.start)
    } else {
        (moved.start, moved.end)
    };
    assert_eq!((start.x, start.y, end.x, end.y), (7000, 0, 0, 4000));
    for bytes in [
        doc.export_hwp_native().unwrap(),
        doc.export_hwpx_native().unwrap(),
    ] {
        let reopened = header_line(&DocumentCore::from_bytes(&bytes).unwrap());
        assert_eq!(
            (reopened.common.width, reopened.common.height),
            (7000, 4000)
        );
    }
    // 본문 경로가 아닌 개체는 직선이어야 한다.
    assert!(doc
        .move_line_endpoint_by_path_native(0, host, &path, 9, 0, 0, 1, 1)
        .is_err());
}
