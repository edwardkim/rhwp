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

/// 누름틀 한 개를 담은 모델 — `end_ci` 로 필드 끝의 ci 를 바꾼다.
/// 한/글 2020 은 끝을 `\tclk`(157510763)로, 11.0.0.8808 은 시작과 같은 `%clk`(627272811)로 적는다.
fn clickhere_model(end_ci: i64) -> String {
    format!(
        r#"{{
        "ro": {{
            "hp": "p0",
            "p0": {{"id": 0, "ru": [{{"cp": "", "ch": [
                {{"cc": 3, "ci": 627272811, "co": "B"}},
                {{"t": "TGT_DPT"}},
                {{"cc": 4, "ci": {end_ci}, "co": "E"}},
                {{"t": " 끝"}}
            ]}}]}}
        }},
        "cs": {{
            "B": {{"cc": 3, "ci": 627272811,
                   "ps": {{"na": 539, "pa": [{{"na": 16384, "ty": 1, "va": "TGT_DPT"}}]}},
                   "fb": {{"id": 1250575263, "ty": 627272811, "ed": true, "la": 0, "di": true,
                          "pa": {{"fi": false, "pr": 9,
                                 "co": "Clickhere:set:49:Direction:wstring:5:가 나:다 HelpState:wstring:2:도움"}},
                          "zo": 4294967295, "fi": 627272811}}}},
            "E": {{"fo": "B", "cc": 4, "ci": {end_ci}, "zo": 4294967295}}
        }}
    }}"#
    )
}

/// 누름틀 시작·끝이 HWPX 필드 쌍으로 옮겨지고, 필드 끝의 두 표기(ci) 모두 끝으로 읽힌다.
#[test]
fn clickhere_field_pair_survives_both_end_notations() {
    for end_ci in [157_510_763_i64, 627_272_811] {
        let parts = hwpjson_to_hwpx_parts(&clickhere_model(end_ci))
            .unwrap_or_else(|e| panic!("end ci={end_ci} 변환 실패: {e}"));
        let xml = &parts.section_xml;
        assert_eq!(
            count_open(xml, "hp:fieldBegin"),
            1,
            "end ci={end_ci}: 시작은 하나 {xml}"
        );
        assert!(
            xml.contains(r#"type="CLICK_HERE" name="TGT_DPT" editable="1" dirty="1""#),
            "end ci={end_ci}: {xml}"
        );
        assert!(xml.contains(r#"fieldid="627272811""#), "{xml}");
        // Command 항목 값은 글자 수로 자른다(공백·콜론 포함)
        assert!(
            xml.contains(r#"<hp:stringParam name="Direction">가 나:다</hp:stringParam>"#),
            "{xml}"
        );
        assert!(
            xml.contains(r#"<hp:stringParam name="HelpState">도움</hp:stringParam>"#),
            "{xml}"
        );
        assert!(
            xml.contains(r#"<hp:integerParam name="Prop">9</hp:integerParam>"#),
            "{xml}"
        );
        assert!(
            xml.contains(r#"<hp:fieldEnd beginIDRef="1250575263" fieldid="627272811"/>"#),
            "end ci={end_ci}: {xml}"
        );
        let begin = xml.find("<hp:fieldBegin").unwrap();
        let text = xml.find("TGT_DPT</hp:t>").expect("필드 글자");
        let end = xml.find("<hp:fieldEnd").unwrap();
        assert!(begin < text && text < end, "시작·글자·끝 순서 {xml}");
    }
}

/// 글자 안 탭(`co` 없는 `{cc:9}`)은 오류가 아니라 HWPX 탭이 된다.
#[test]
fn inline_tab_becomes_hwpx_tab() {
    let model = r#"{
        "ro": {
            "hp": "p0",
            "p0": {"id": 0, "ru": [{"cp": "", "ch": [
                {"t": "가"}, {"cc": 9, "wi": 1820, "le": 0, "ty": 1}, {"t": "나"}
            ]}]}
        }
    }"#;
    let parts = hwpjson_to_hwpx_parts(model).expect("탭 모델 변환 실패");
    let xml = &parts.section_xml;
    assert!(
        xml.contains(r#"<hp:t><hp:tab width="1820" leader="0" type="1"/></hp:t>"#),
        "{xml}"
    );
    let a = xml.find("가</hp:t>").unwrap();
    let tab = xml.find("<hp:tab ").unwrap();
    let b = xml.find("나</hp:t>").unwrap();
    assert!(a < tab && tab < b, "글자·탭 순서 {xml}");
}
