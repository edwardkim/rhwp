//! 한글 클립보드 문서모델(hwpjson) → HWPX 조각 변환 계약.
//!
//! 공개 API `hwpjson_to_hwpx_parts` 만 쓴다. 실제 클립보드 표본이 있으면 개수까지 대조하고,
//! 없으면 그 시험은 조용히 통과한다(표본은 저장소에 싣지 않는다).

use rhwp::document_core::hwpjson::hwpjson_to_hwpx_parts;

/// 표본 경로. 없으면 개수 대조 시험은 건너뛴다. `HWPJSON_SAMPLE` 로 덮어쓸 수 있다.
const SAMPLE: &str = "samples/hwpjson/clipboard-model.json";

fn sample() -> Option<String> {
    if let Ok(p) = std::env::var("HWPJSON_SAMPLE") {
        return std::fs::read_to_string(p).ok();
    }
    std::fs::read_to_string(SAMPLE).ok()
}

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

/// 실제 클립보드 표본을 원본 HWPX 대조로 확정한 개수와 맞춘다.
#[test]
fn sample_model_conversion_counts() {
    let Some(json) = sample() else { return };
    let parts = hwpjson_to_hwpx_parts(&json).expect("표본 변환 실패");

    // 본문 문단(표 칸 안 문단 포함) 259개
    assert_eq!(count_open(&parts.section_xml, "hp:p"), 259, "문단 수");
    // 표 15개 · 셀 63개
    assert_eq!(count_open(&parts.section_xml, "hp:tbl"), 15, "표 수");
    assert_eq!(count_open(&parts.section_xml, "hp:tc"), 63, "표 칸 수");
    // 그림 8개 — pic 요소와 BinData 항목 수가 같아야 한다
    assert_eq!(count_open(&parts.section_xml, "hp:pic"), 8, "그림 수");
    assert_eq!(parts.bins.len(), 8, "BinData 항목 수");
    // 도형(글상자) 1개
    assert_eq!(count_open(&parts.section_xml, "hp:rect"), 1, "도형 수");

    // header 정의표 개수 — 모델 표 크기와 같아야 한다
    assert!(parts
        .header_xml
        .contains("<hh:charProperties itemCnt=\"202\">"));
    assert!(parts
        .header_xml
        .contains("<hh:paraProperties itemCnt=\"97\">"));
    assert!(parts.header_xml.contains("<hh:borderFills itemCnt=\"20\">"));
}
