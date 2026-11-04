//! 관계형: 메타파일 host font는 저장 문서가 아닌 paint 세션 상태다.
//! 주석 갱신: 2026-11-04 04:19:39 KST; 실제 쪽 소속에 따른 자원 조회 대조 보완.
//! 실물 배치나 렌더링 일치율을 고정하는 검사가 아니다.

use rhwp::document_core::DocumentCore;

const FONT: &[u8] = include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf");

#[test]
fn page_font_requests_keep_effects_on_the_page_that_owns_the_text() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "plain").unwrap();
    core.insert_page_break_native(0, 0, 5).unwrap();
    core.insert_text_native(0, 1, 0, "shadow").unwrap();
    let mut doc = core.document().clone();
    assert_eq!(doc.sections[0].paragraphs.len(), 2);
    let mut effect = doc.doc_info.char_shapes[0].clone();
    effect.shadow_type = 2;
    let id = doc.doc_info.char_shapes.len() as u32;
    doc.doc_info.char_shapes.push(effect);
    let second = &mut doc.sections[0].paragraphs[1];
    second.apply_char_shape_range(0, "shadow".chars().count(), id);
    core.set_document(doc);
    core.set_hangul2024_compat(true);
    let source = format!("{:?}", core.document());
    let read = |page| -> serde_json::Value {
        serde_json::from_str(&core.get_page_paint_font_requests_native(page).unwrap()).unwrap()
    };
    let first = read(0);
    let second = read(1);
    assert_eq!(
        first["textEffects"],
        serde_json::json!([]),
        "앞쪽은 뒤쪽 효과 글꼴을 요구하지 않음"
    );
    let glyphs = second["textEffects"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|request| request["text"].as_str().unwrap().chars())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        glyphs,
        "shadow".chars().collect(),
        "실제 효과 문자의 공급은 누락하지 않음"
    );
    assert!(
        core.get_page_paint_font_requests_native(2).is_err(),
        "없는 쪽을 전 문서 조회로 대체하지 않음"
    );
    assert_eq!(
        format!("{:?}", core.document()),
        source,
        "자원 조회는 원본·쪽 소유를 바꾸지 않음"
    );
}

#[test]
fn metafile_font_registration_and_clear_preserve_document_and_saved_bytes() {
    let mut core = DocumentCore::new_empty();
    let document_before = format!("{:?}", core.document());
    let saved_before = rhwp::serializer::serialize_document(core.document()).unwrap();
    let info_before = core.get_document_info();

    // 등록 중복과 해제 뒤의 재등록은 실제 세션 상태가 바뀌었음을 확인한다.
    for _ in 0..2 {
        assert!(core
            .register_metafile_font_native("RHWP Host Fixture", FONT, 0)
            .unwrap());
        assert!(!core
            .register_metafile_font_native("RHWP Host Fixture", FONT, 0)
            .unwrap());
        assert_eq!(format!("{:?}", core.document()), document_before);
        assert_eq!(
            rhwp::serializer::serialize_document(core.document()).unwrap(),
            saved_before,
            "paint 프로그램 등록은 저장 IR과 원시 스트림을 바꾸지 않는다"
        );
        core.clear_metafile_fonts_native();
        assert_eq!(format!("{:?}", core.document()), document_before);
        assert_eq!(
            rhwp::serializer::serialize_document(core.document()).unwrap(),
            saved_before,
            "paint 세션 해제는 저장 내용을 바꾸지 않는다"
        );
        assert_eq!(core.get_document_info(), info_before);
    }
}
