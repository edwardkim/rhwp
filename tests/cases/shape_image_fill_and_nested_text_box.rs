//! 도형 속성의 그림 채우기·그림자 투명도와, 표 셀·머리말 안 도형의 글상자 붙이기·떼기.
//!
//! 한컴 12.30 개체 모양 복사는 그림 채우기와 그림자 투명도도 옮긴다. 앱은 `getShapeProperties`의
//! 값을 그대로 `setShapeProperties`에 넘겨 붙이므로 두 값이 오가야 한다.
//! 한컴 도형 안에 글자 넣기는 표 셀·머리말 안 도형에서도 켜진다(`setTextBoxAtByPath`).
#![cfg(not(target_arch = "wasm32"))]
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::ShapeObject;
use rhwp::model::style::FillType;
use rhwp::wasm_api::HwpDocument;

fn blank() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc
}

fn rect(doc: &mut HwpDocument) -> (usize, usize) {
    let result = doc
        .create_shape_control_native(
            0,
            0,
            0,
            8000,
            6000,
            5000,
            6000,
            false,
            "InFrontOfText",
            "rectangle",
            false,
            false,
            &[],
        )
        .unwrap();
    let json: serde_json::Value = serde_json::from_str(&result).unwrap();
    (
        json["paraIdx"].as_u64().unwrap() as usize,
        json["controlIdx"].as_u64().unwrap() as usize,
    )
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

#[test]
fn shape_picture_fill_and_shadow_alpha_are_read_and_written() {
    let mut doc = blank();
    doc.insert_picture_native(
        0,
        0,
        0,
        &[],
        include_bytes!("../../assets/logo/logo-16.png"),
        2000,
        2000,
        16,
        16,
        "png",
        "채울 그림",
        None,
        None,
    )
    .unwrap();
    let bin_id = doc.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .find_map(|c| match c {
            Control::Picture(p) => Some(p.image_attr.bin_data_id),
            _ => None,
        })
        .unwrap();
    let (para, ctrl) = rect(&mut doc);
    let props = format!(
        r#"{{"fillType":"image","imageBinDataId":{bin_id},"imageFillMode":"fitToSize","shadowType":1,"shadowColor":8421504,"shadowOffsetX":283,"shadowOffsetY":283,"shadowAlpha":128}}"#
    );
    doc.set_shape_properties_native(0, para, ctrl, &props)
        .unwrap();

    let read: serde_json::Value =
        serde_json::from_str(&doc.get_shape_properties_native(0, para, ctrl).unwrap()).unwrap();
    assert_eq!(read["fillType"], "image");
    assert_eq!(read["imageBinDataId"], bin_id);
    assert_eq!(read["imageFillMode"], "fitToSize");
    assert_eq!(read["shadowAlpha"], 128);

    // 읽은 값을 다른 도형에 그대로 붙이면 같은 채우기·그림자가 된다(개체 모양 붙이기).
    let (para2, ctrl2) = rect(&mut doc);
    doc.set_shape_properties_native(0, para2, ctrl2, &read.to_string())
        .unwrap();
    let pasted: serde_json::Value =
        serde_json::from_str(&doc.get_shape_properties_native(0, para2, ctrl2).unwrap()).unwrap();
    for key in ["fillType", "imageBinDataId", "imageFillMode", "shadowAlpha"] {
        assert_eq!(pasted[key], read[key], "{key}");
    }

    for bytes in [
        doc.export_hwp_native().unwrap(),
        doc.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        let found = shapes(&reopened.document().sections[0].paragraphs);
        assert_eq!(found.len(), 2);
        for shape in found {
            let d = shape.drawing().unwrap();
            assert_eq!(d.fill.fill_type, FillType::Image);
            assert_eq!(d.fill.image.as_ref().unwrap().bin_data_id, bin_id);
            assert_eq!(d.shadow_alpha, 128);
        }
    }
}

/// 사각형 하나를 만들어 복사해 둔 빈 문서. 붙일 곳(표·머리말)은 그 뒤에 만들어 번호가 밀리지 않게 한다.
fn blank_with_copied_rect() -> HwpDocument {
    let mut doc = blank();
    let (para, ctrl) = rect(&mut doc);
    doc.copy_control_native(0, para, &[], ctrl).unwrap();
    doc
}

/// 복사한 사각형을 `path`가 가리키는 문단(표 셀·머리말) 첫 자리에 붙인다.
fn paste_rect(doc: &mut HwpDocument, host: usize, path: &[(usize, usize, usize)]) {
    let pasted = doc
        .paste_internal_in_cell_by_path_native(0, host, path, 0)
        .unwrap();
    assert!(pasted.contains("\"ok\":true"), "{pasted}");
}

fn text_box_round_trip(doc: &HwpDocument, inner: impl Fn(&DocumentCore) -> Vec<&ShapeObject>) {
    for bytes in [
        doc.export_hwp_native().unwrap(),
        doc.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        let found = inner(&reopened);
        assert_eq!(found.len(), 1);
        assert!(found[0].drawing().unwrap().text_box.is_some());
    }
}

#[test]
fn text_box_attaches_to_a_shape_in_a_table_cell() {
    let mut doc = blank_with_copied_rect();
    let table: serde_json::Value =
        serde_json::from_str(&doc.create_table_native(0, 0, 0, 1, 1).unwrap()).unwrap();
    let host = table["paraIdx"].as_u64().unwrap() as usize;
    let table_ctrl = table["controlIdx"].as_u64().unwrap() as usize;
    paste_rect(&mut doc, host, &[(table_ctrl, 0, 0)]);
    let cell_paras = |core: &DocumentCore| -> Vec<Paragraph> {
        match &core.document().sections[0].paragraphs[host].controls[table_ctrl] {
            Control::Table(t) => t.cells[0].paragraphs.clone(),
            _ => panic!("표가 아니다"),
        }
    };
    let paras = cell_paras(&doc);
    let inner_ctrl = paras[0]
        .controls
        .iter()
        .position(|c| matches!(c, Control::Shape(_)))
        .expect("셀에 붙인 도형이 없다");
    let path = format!(r#"[{{"controlIndex":{table_ctrl},"cellIndex":0,"cellParaIndex":0}}]"#);

    let attached = doc
        .set_text_box_at_by_path_native(0, host, &path, inner_ctrl, true)
        .unwrap();
    assert_eq!(attached, r#"{"ok":true}"#);
    // 이미 있으면 바뀌지 않는다.
    assert_eq!(
        doc.set_text_box_at_by_path_native(0, host, &path, inner_ctrl, true)
            .unwrap(),
        r#"{"ok":false}"#
    );
    text_box_round_trip(&doc, |core| {
        match &core.document().sections[0].paragraphs[host].controls[table_ctrl] {
            Control::Table(t) => shapes(&t.cells[0].paragraphs),
            _ => vec![],
        }
    });

    assert_eq!(
        doc.set_text_box_at_by_path_native(0, host, &path, inner_ctrl, false)
            .unwrap(),
        r#"{"ok":true}"#
    );
    let paras = cell_paras(&doc);
    assert!(shapes(&paras)[0].drawing().unwrap().text_box.is_none());
}

#[test]
fn text_box_attaches_to_a_shape_in_a_header() {
    let mut doc = blank_with_copied_rect();
    let header: serde_json::Value =
        serde_json::from_str(&doc.create_header_footer_native(0, true, 0).unwrap()).unwrap();
    let host = header["paraIndex"].as_u64().unwrap() as usize;
    let header_ctrl = header["controlIndex"].as_u64().unwrap() as usize;
    paste_rect(&mut doc, host, &[(header_ctrl, 0, 0)]);
    let header_shapes = |core: &DocumentCore| -> Vec<ShapeObject> {
        match &core.document().sections[0].paragraphs[host].controls[header_ctrl] {
            Control::Header(h) => shapes(&h.paragraphs).into_iter().cloned().collect(),
            _ => panic!("머리말이 아니다"),
        }
    };
    let inner_ctrl = match &doc.document().sections[0].paragraphs[host].controls[header_ctrl] {
        Control::Header(h) => h.paragraphs[0]
            .controls
            .iter()
            .position(|c| matches!(c, Control::Shape(_)))
            .expect("머리말에 붙인 도형이 없다"),
        _ => panic!("머리말이 아니다"),
    };
    let path = format!(r#"[{{"controlIndex":{header_ctrl},"cellIndex":0,"cellParaIndex":0}}]"#);
    assert_eq!(
        doc.set_text_box_at_by_path_native(0, host, &path, inner_ctrl, true)
            .unwrap(),
        r#"{"ok":true}"#
    );
    assert!(header_shapes(&doc)[0].drawing().unwrap().text_box.is_some());
    for bytes in [
        doc.export_hwp_native().unwrap(),
        doc.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        let found: Vec<ShapeObject> = reopened.document().sections[0]
            .paragraphs
            .iter()
            .flat_map(|p| &p.controls)
            .filter_map(|c| match c {
                Control::Header(h) => Some(shapes(&h.paragraphs).into_iter().cloned()),
                _ => None,
            })
            .flatten()
            .collect();
        assert_eq!(found.len(), 1);
        assert!(found[0].drawing().unwrap().text_box.is_some());
    }
    assert_eq!(
        doc.set_text_box_at_by_path_native(0, host, &path, inner_ctrl, false)
            .unwrap(),
        r#"{"ok":true}"#
    );
    assert!(header_shapes(&doc)[0].drawing().unwrap().text_box.is_none());
}
