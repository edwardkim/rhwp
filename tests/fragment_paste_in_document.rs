//! Stage 2 통합 테스트 — `paste_hwpx_fragment_in_document_native` (wasm bridge underlying)
//!
//! native 통합에서 `HwpDocument::from_bytes` 로 실제 HWPX 를 로드한 뒤
//! 통합 paste 함수를 호출해 raw XML + IR 의 일관성을 검증한다.
//!
//! `pasteHwpxFragmentInDocument` (wasm-bindgen) 자체는 native 에서도 호출 가능하지만
//! `JsValue` 의존성을 회피하기 위해 underlying `paste_hwpx_fragment_in_document_native`
//! 를 직접 호출한다.

use rhwp::document_core::{DocumentCore, SourceDefinitions};

fn bundled_hwpx() -> &'static [u8] {
    include_bytes!("../saved/04-blank_hwpx_empty.hwpx")
}

#[test]
fn integration_paste_paragraphs_in_loaded_hwpx() {
    let mut doc = DocumentCore::from_bytes(bundled_hwpx()).expect("from_bytes");

    let raw_section_before_len = doc.get_source_section_xml(0).expect("section 0 raw").len();
    let header_before_len = doc.get_source_header_xml().len();

    let fragment = r#"<hp:p paraPrIDRef="0" styleIDRef="0"><hp:run charPrIDRef="0"><hp:t>integration-marker-A1</hp:t></hp:run></hp:p>"#;
    let source = SourceDefinitions::default();

    let result = doc
        .paste_hwpx_fragment_in_document_native(0, 0, fragment, &source)
        .expect("paste ok");

    assert_eq!(result.inserted_para_count, 1);

    let raw_after = doc.get_source_section_xml(0).expect("section 0 raw after");
    assert!(raw_after.len() > raw_section_before_len, "section grew");
    assert!(
        raw_after.contains("integration-marker-A1"),
        "fragment text appears in raw section"
    );

    // header 는 source 가 비어 있으므로 변동 0 (ID remap 없음)
    let header_after_len = doc.get_source_header_xml().len();
    assert_eq!(
        header_before_len, header_after_len,
        "header should not grow when source has no new defs"
    );
}

#[test]
fn integration_paste_table_in_loaded_hwpx_recomputes_addrs() {
    let mut doc = DocumentCore::from_bytes(bundled_hwpx()).expect("from_bytes");

    // 2x2 표 fragment — recompute_table_addrs 가 colSpan/rowSpan 점유 그리드로 보정
    let fragment = "<hp:p paraPrIDRef=\"0\"><hp:tbl rowCnt=\"99\" colCnt=\"2\">\
<hp:tr>\
<hp:tc rowAddr=\"99\" colAddr=\"99\" rowSpan=\"1\" colSpan=\"1\"><hp:subList><hp:p/></hp:subList></hp:tc>\
<hp:tc rowAddr=\"99\" colAddr=\"99\" rowSpan=\"1\" colSpan=\"1\"><hp:subList><hp:p/></hp:subList></hp:tc>\
</hp:tr>\
<hp:tr>\
<hp:tc rowAddr=\"99\" colAddr=\"99\" rowSpan=\"1\" colSpan=\"1\"><hp:subList><hp:p/></hp:subList></hp:tc>\
<hp:tc rowAddr=\"99\" colAddr=\"99\" rowSpan=\"1\" colSpan=\"1\"><hp:subList><hp:p/></hp:subList></hp:tc>\
</hp:tr>\
</hp:tbl></hp:p>";
    let source = SourceDefinitions::default();

    doc.paste_hwpx_fragment_in_document_native(0, 0, fragment, &source)
        .expect("paste table ok");

    let raw_after = doc.get_source_section_xml(0).expect("section 0 raw after");
    assert!(raw_after.contains("rowCnt=\"2\""), "rowCnt 99→2 corrected");
    assert!(
        !raw_after.contains("rowAddr=\"99\""),
        "all rowAddr=99 corrected"
    );
    assert!(
        raw_after.contains("rowAddr=\"0\" colAddr=\"0\""),
        "(0,0) cell"
    );
    assert!(
        raw_after.contains("rowAddr=\"1\" colAddr=\"1\""),
        "(1,1) cell"
    );
}

#[test]
fn integration_two_consecutive_pastes_reuse_ids() {
    let mut doc = DocumentCore::from_bytes(bundled_hwpx()).expect("from_bytes");

    let fragment = r#"<hp:p paraPrIDRef="9" styleIDRef="0"><hp:run charPrIDRef="9"><hp:t>reuse-marker</hp:t></hp:run></hp:p>"#;
    let source = SourceDefinitions {
        char_prs: "<hh:charPr id=\"9\" height=\"4242\"/>".into(),
        para_prs: "<hh:paraPr id=\"9\" alignTag=\"left\"/>".into(),
        ..Default::default()
    };

    let header_before = doc.get_source_header_xml().len();
    let r1 = doc
        .paste_hwpx_fragment_in_document_native(0, 0, fragment, &source)
        .expect("first paste");
    let header_after_first = doc.get_source_header_xml().len();

    // 첫 paste 는 새 정의 추가 → header 길이 증가
    assert!(header_after_first > header_before);

    let r2 = doc
        .paste_hwpx_fragment_in_document_native(0, 0, fragment, &source)
        .expect("second paste");
    let header_after_second = doc.get_source_header_xml().len();

    // 두 번째 paste 는 동일 정의 재사용 → header 길이 변화 없음 (W5 위험 직접 측정)
    assert_eq!(
        header_after_second, header_after_first,
        "header bloated on second paste — ID reuse failed"
    );
    assert_eq!(
        r1.id_remap.char_pr.get(&9),
        r2.id_remap.char_pr.get(&9),
        "char_pr ID remap target diverged"
    );
    assert_eq!(
        r1.id_remap.para_pr.get(&9),
        r2.id_remap.para_pr.get(&9),
        "para_pr ID remap target diverged"
    );
}
