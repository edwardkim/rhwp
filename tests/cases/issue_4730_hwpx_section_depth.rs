//! [#4730] HWPX 섹션 파서 재귀 깊이 가드 — 기본 스택에서 XmlError 로 거부.
//!
//! `parse_container` 자기재귀와 문단↔표↔셀 상호재귀에 상한이 없으면 악성
//! `.hwpx` 하나가 SIGSEGV(catch_unwind 로 못 잡는 하드 크래시)를 낸다.
//! 가드는 큰 본문 프레임을 쌓기 전에 동작해야 한다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::parser::hwpx::section::parse_hwpx_section;
use rhwp::parser::hwpx::HwpxError;

fn nested_table_section_xml(depth: usize) -> String {
    let mut xml = String::from(
        r#"<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph" xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section"><hp:p paraPrIDRef="0" styleIDRef="0">"#,
    );
    for _ in 0..depth {
        xml.push_str("<hp:tbl><hp:tr><hp:tc><hp:p>");
    }
    for _ in 0..depth {
        xml.push_str("</hp:p></hp:tc></hp:tr></hp:tbl>");
    }
    xml.push_str("</hp:p></hs:sec>");
    xml
}

fn nested_container_section_xml(depth: usize) -> String {
    let mut xml = String::from(
        r#"<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph" xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section"><hp:p paraPrIDRef="0" styleIDRef="0">"#,
    );
    for _ in 0..depth {
        xml.push_str("<hp:container>");
    }
    for _ in 0..depth {
        xml.push_str("</hp:container>");
    }
    xml.push_str("</hp:p></hs:sec>");
    xml
}

fn assert_xml_nesting_error(
    result: Result<rhwp::model::document::Section, HwpxError>,
    needle: &str,
) {
    match result {
        Err(HwpxError::XmlError(msg)) => {
            assert!(
                msg.contains(needle),
                "XmlError 메시지에 `{needle}` 가 없다: {msg}"
            );
        }
        other => panic!("상한 초과가 XmlError 로 거부되지 않았다: {other:?}"),
    }
}

#[test]
fn table_nesting_beyond_limit_is_rejected_on_default_stack() {
    // 한 겹마다 문단·표·셀 큰 프레임이 겹친다. 상한(16)을 넘긴 입력이
    // 기본 테스트 스레드에서 크래시하지 않고 XmlError 여야 한다.
    assert_xml_nesting_error(
        parse_hwpx_section(&nested_table_section_xml(17)),
        "section nesting exceeds",
    );
}

#[test]
fn container_nesting_hostile_depth_is_xml_error_on_default_stack() {
    // 여는 태그 ~14바이트라 수만 겹도 입력 상한에 안 걸린다.
    // 가드 없이 parse_hwpx_section 에 넣으면 SIGSEGV. 가드 후엔 XmlError.
    assert_xml_nesting_error(
        parse_hwpx_section(&nested_container_section_xml(10_000)),
        "container nesting exceeds",
    );
}
