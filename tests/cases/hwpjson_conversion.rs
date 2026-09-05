//! 한글 클립보드 문서모델(hwpjson) → HWPX 조각 변환 계약.
//!
//! 공개 API `hwpjson_to_hwpx_parts` 만 쓴다. 공개 최소 클립보드 표본은 항상 저장소에서 읽어,
//! 변환 경로가 환경 변수나 개인 파일에 의존하지 않도록 한다.

use rhwp::document_core::hwpjson::hwpjson_to_hwpx_parts;

const SAMPLE: &str = include_str!("../../samples/hwpjson/clipboard-model.json");

/// 여는 태그 개수 — `<hp:tbl ` / `<hp:tbl>` 만 세고 `<hp:tblXxx` 는 세지 않는다.
fn count_open(xml: &str, tag: &str) -> usize {
    xml.matches(&format!("<{tag} ")).count() + xml.matches(&format!("<{tag}>")).count()
}

#[test]
fn empty_input_is_error() {
    assert!(hwpjson_to_hwpx_parts("").is_err());
    assert!(hwpjson_to_hwpx_parts("[]").is_err());
}

/// 표가 전혀 없는 최소 모델도 구조가 성립해야 한다(부분 모델 방어).
#[test]
fn minimal_model_still_emits_skeleton() {
    let parts = hwpjson_to_hwpx_parts("{}").expect("최소 모델 변환 실패");
    assert!(parts.header_xml.starts_with("<?xml"));
    assert!(parts.header_xml.contains("<hh:refList>"));
    assert!(parts.header_xml.ends_with("</hh:head>"));
    assert!(parts.section_xml.contains("<hs:sec"));
    assert!(parts.section_xml.ends_with("</hs:sec>"));
    assert!(parts.bins.is_empty());
    // 글꼴표는 언어 7개 슬롯을 항상 낸다
    assert!(parts.header_xml.contains("<hh:fontfaces itemCnt=\"7\">"));
}

/// 공개 최소 클립보드 표본은 환경과 무관하게 실제 변환 경로를 통과한다.
#[test]
fn checked_in_clipboard_model_converts() {
    let parts = hwpjson_to_hwpx_parts(SAMPLE).expect("공개 표본 변환 실패");

    assert_eq!(count_open(&parts.section_xml, "hp:p"), 1, "문단 수");
    assert!(parts.section_xml.contains("공개 클립보드 표본"));
    assert!(parts.bins.is_empty());
}

#[test]
fn unsupported_control_and_invalid_image_are_errors() {
    let unsupported = r#"{
        "ro": {
            "hp": "p0",
            "p0": {"id": 0, "ru": [{"cp": "", "ch": [{"ci": 1, "co": "unknown"}]}]}
        },
        "cs": {"unknown": {}}
    }"#;
    let unsupported_error = hwpjson_to_hwpx_parts(unsupported).unwrap_err().to_string();
    assert!(unsupported_error.contains("지원하지 않는 hwpjson control 종류"));

    let invalid_image = r#"{"bi": [{"sr": "missing.png", "ty": "image/png"}]}"#;
    let image_error = hwpjson_to_hwpx_parts(invalid_image)
        .unwrap_err()
        .to_string();
    assert!(image_error.contains("hwpjson 이미지 원본이 없다"));

    let corrupt_image = r#"{
        "bi": [{"sr": "broken.png", "ty": "image/png"}],
        "bidt": {"broken.png": "not-base64"}
    }"#;
    let corrupt_error = hwpjson_to_hwpx_parts(corrupt_image)
        .unwrap_err()
        .to_string();
    assert!(corrupt_error.contains("hwpjson 이미지 base64가 손상됐다"));
}
