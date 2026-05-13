//! Stage 4 통합 테스트 — `paste_fragment_into_section` 진입점을 native build에서
//! 직접 호출해 paragraphs/table 두 형태의 fragment paste를 검증한다.
//!
//! `paste_hwpx_fragment_raw` (wasm_bindgen 함수) 자체는 native에서도 호출 가능하지만
//! `JsValue` 의존성을 회피하기 위해 native 통합 테스트는 underlying
//! `crate::document_core::paste_fragment_into_section` 을 직접 호출한다.

use rhwp::document_core::{paste_fragment_into_section, SourceDefinitions};

fn empty_section() -> String {
    "<hs:sec><hp:p paraPrIDRef=\"0\" styleIDRef=\"0\"><hp:run charPrIDRef=\"0\"><hp:t>x</hp:t></hp:run></hp:p></hs:sec>"
        .to_string()
}

fn empty_header() -> String {
    "<hh:head>\
<hh:charPropertyList><hh:charPr id=\"0\" height=\"1000\"/></hh:charPropertyList>\
<hh:paraPropertyList><hh:paraPr id=\"0\"/></hh:paraPropertyList>\
<hh:styleList><hh:style id=\"0\" name=\"def\"/></hh:styleList>\
<hh:borderFillList></hh:borderFillList>\
</hh:head>"
        .to_string()
}

#[test]
fn integration_paste_paragraphs_fragment() {
    let section = empty_section();
    let mut header = empty_header();
    let fragment = r#"<hp:p paraPrIDRef="9" styleIDRef="0"><hp:run charPrIDRef="9"><hp:t>integration paragraph</hp:t></hp:run></hp:p>"#;
    let source = SourceDefinitions {
        char_prs: "<hh:charPr id=\"9\" height=\"3000\"/>".into(),
        para_prs: "<hh:paraPr id=\"9\" alignTag=\"left\"/>".into(),
        ..Default::default()
    };
    let result =
        paste_fragment_into_section(&section, &mut header, 0, fragment, &source).unwrap();
    assert_eq!(result.inserted_para_count, 1);
    assert!(result.new_section_xml.contains("integration paragraph"));
    let new_char_pr = result.id_remap.char_pr.get(&9).copied().unwrap();
    assert_ne!(new_char_pr, 9);
    assert!(header.contains("height=\"3000\""));
    assert!(!result.new_section_xml.contains("charPrIDRef=\"9\""));
    assert!(result
        .new_section_xml
        .contains(&format!("charPrIDRef=\"{new_char_pr}\"")));
}

#[test]
fn integration_paste_table_fragment_recomputes_addrs() {
    let section = empty_section();
    let mut header = empty_header();
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
    let result =
        paste_fragment_into_section(&section, &mut header, 0, fragment, &source).unwrap();
    assert_eq!(result.inserted_para_count, 1);
    assert!(result.new_section_xml.contains("rowCnt=\"2\""));
    assert!(!result.new_section_xml.contains("rowAddr=\"99\""));
    assert!(!result.new_section_xml.contains("colAddr=\"99\""));
    assert!(result
        .new_section_xml
        .contains("rowAddr=\"0\" colAddr=\"0\""));
    assert!(result
        .new_section_xml
        .contains("rowAddr=\"0\" colAddr=\"1\""));
    assert!(result
        .new_section_xml
        .contains("rowAddr=\"1\" colAddr=\"0\""));
    assert!(result
        .new_section_xml
        .contains("rowAddr=\"1\" colAddr=\"1\""));
}

#[test]
fn integration_paste_same_fragment_twice_reuses_ids() {
    let section = empty_section();
    let mut header = empty_header();
    let fragment = r#"<hp:p paraPrIDRef="0"><hp:run charPrIDRef="9"><hp:t>x</hp:t></hp:run></hp:p>"#;
    let source = SourceDefinitions {
        char_prs: "<hh:charPr id=\"9\" height=\"4242\"/>".into(),
        ..Default::default()
    };
    let header_baseline_len = header.len();
    let r1 = paste_fragment_into_section(&section, &mut header, 0, fragment, &source).unwrap();
    let header_after_first = header.len();
    let r2 = paste_fragment_into_section(&r1.new_section_xml, &mut header, 0, fragment, &source)
        .unwrap();
    let header_after_second = header.len();

    assert!(header_after_first > header_baseline_len);
    assert_eq!(
        header_after_second, header_after_first,
        "header bloated on second paste — ID reuse failed"
    );
    assert_eq!(
        r1.id_remap.char_pr.get(&9),
        r2.id_remap.char_pr.get(&9),
        "ID remap diverged across calls"
    );
}
