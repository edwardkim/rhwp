//! [#7507] 글자가 없는 문단에 건 글자 모양은 그 문단에 남아야 한다.
//!
//! 빈 문단의 글자 모양 런(위치 0)은 이어 입력할 글자와 저장되는 문단 끝의 모양이다.
//! 범위는 캐럿 위치(`0..0`)와 hwpctl `CharShape`의 문단 전체(`0..65535`)를 모두 본다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const BOLD_20PT: &str = r#"{"bold":true,"fontSize":2000}"#;

fn bold_and_size(json: &str) -> (bool, i64) {
    let v: Value = serde_json::from_str(json).unwrap();
    (
        v["bold"].as_bool().unwrap(),
        v["fontSize"].as_i64().unwrap(),
    )
}

fn body_props(doc: &HwpDocument, para: usize, offset: usize) -> (bool, i64) {
    bold_and_size(&doc.get_char_properties_at_native(0, para, offset).unwrap())
}

/// 문단 0 `A`, 문단 1 빈 문단.
fn body_with_empty_paragraph() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc.insert_text_native(0, 0, 0, "A").unwrap();
    doc.split_paragraph_native(0, 0, 1, None).unwrap();
    doc
}

#[test]
fn body_empty_paragraph_keeps_char_format() {
    for end in [0, 65535] {
        let mut doc = body_with_empty_paragraph();
        let plain = body_props(&doc, 0, 0);
        assert!(!plain.0 && plain.1 != 2000, "기본 모양이 이미 굵게·20pt다");
        let height = doc.document().sections[0].paragraphs[1].line_segs[0].line_height;
        let shape_id = |doc: &HwpDocument| {
            doc.document().sections[0].paragraphs[1].char_shapes[0].char_shape_id
        };
        let original = shape_id(&doc);

        doc.apply_char_format_native(0, 1, 0, end, BOLD_20PT)
            .unwrap();
        assert_eq!(body_props(&doc, 1, 0), (true, 2000), "0..{end}: 조회");
        assert!(
            doc.document().sections[0].paragraphs[1].line_segs[0].line_height > height,
            "0..{end}: 글자 크기만큼 빈 줄이 높아져야 한다"
        );
        assert_eq!(body_props(&doc, 0, 0), plain, "0..{end}: 앞 문단은 그대로");

        let saved = HwpDocument::from_bytes(&doc.export_hwp_native().unwrap()).unwrap();
        assert_eq!(
            body_props(&saved, 1, 0),
            (true, 2000),
            "0..{end}: HWP 재열기"
        );

        // 되돌리기·다시 하기는 모양 ID 를 그대로 다시 건다.
        let applied = shape_id(&doc);
        doc.set_char_shape_id_native(0, 1, 0, end, original)
            .unwrap();
        assert_eq!(body_props(&doc, 1, 0), plain, "0..{end}: 되돌리기");
        doc.set_char_shape_id_native(0, 1, 0, end, applied).unwrap();

        doc.insert_text_native(0, 1, 0, "가").unwrap();
        assert_eq!(
            body_props(&doc, 1, 0),
            (true, 2000),
            "0..{end}: 이어 입력한 글자"
        );
    }
}

#[test]
fn cell_empty_paragraph_keeps_char_format() {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    let created: Value =
        serde_json::from_str(&doc.create_table_native(0, 0, 0, 1, 1).unwrap()).unwrap();
    let para = created["paraIdx"].as_u64().unwrap() as usize;
    let ctrl = created["controlIdx"].as_u64().unwrap() as usize;
    let cell_props = |doc: &HwpDocument| {
        bold_and_size(
            &doc.get_cell_char_properties_at_native(0, para, ctrl, 0, 0, 0)
                .unwrap(),
        )
    };

    doc.apply_char_format_in_cell_native(0, para, ctrl, 0, 0, 0, 0, BOLD_20PT)
        .unwrap();
    assert_eq!(cell_props(&doc), (true, 2000));

    doc.insert_text_in_cell_native(0, para, ctrl, 0, 0, 0, "가")
        .unwrap();
    assert_eq!(cell_props(&doc), (true, 2000), "이어 입력한 글자");
}
