//! Issue #7495: HWP의 '면 색 없음' 채우기를 속성 조회가 흰색 단색으로 돌려주던 결함.
//!
//! 한컴은 '면 색 없음'을 단색 채우기 + 면 색 `0xFFFFFFFF` 로 저장한다. 렌더러는 이를 칠하지
//! 않고, HWPX `faceColor="none"` 은 파서가 `FillType::None` 으로 읽는다(#1172). 속성 JSON 만
//! `solid`·`#ffffff` 로 답해 진짜 흰색과 구분되지 않았고, HWPX 저장 후에는 답이 바뀌었다.
//! 진짜 흰색(`0x00FFFFFF`)은 그대로 `solid`·`#ffffff` 여야 한다.

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn load(rel: &str) -> HwpDocument {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    HwpDocument::from_bytes(&bytes).expect("parse")
}

fn blank() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc
}

fn reopen_all(doc: HwpDocument) -> Vec<(&'static str, HwpDocument)> {
    let hwp = HwpDocument::from_bytes(&doc.export_hwp_native().unwrap()).unwrap();
    let hwpx = HwpDocument::from_bytes(&doc.export_hwpx_native().unwrap()).unwrap();
    vec![
        ("편집 중", doc),
        ("HWP 저장 후", hwp),
        ("HWPX 저장 후", hwpx),
    ]
}

/// `(fillType, fillColor)`
fn fill(json: &str) -> (String, String) {
    let v: Value = serde_json::from_str(json).unwrap_or_else(|e| panic!("{e}: {json}"));
    (
        v["fillType"].as_str().unwrap().to_string(),
        v["fillColor"].as_str().unwrap().to_string(),
    )
}

fn char_fill(doc: &HwpDocument, para: usize, offset: usize) -> (String, String) {
    fill(&doc.get_char_properties_at_native(0, para, offset).unwrap())
}

fn para_fill(doc: &HwpDocument, para: usize) -> (String, String) {
    fill(&doc.get_para_properties_at_native(0, para).unwrap())
}

#[test]
fn blank_document_has_no_fill_in_every_format() {
    // 새 문서의 글자·문단 모양은 면 색 0xFFFFFFFF 인 borderFill 2 를 쓴다.
    for (label, doc) in reopen_all(blank()) {
        assert_eq!(char_fill(&doc, 0, 0).0, "none", "{label}: 글자");
        assert_eq!(para_fill(&doc, 0).0, "none", "{label}: 문단");
    }
}

#[test]
fn hancom_hwp_no_fill_matches_hwpx() {
    for rel in ["samples/para-001.hwp", "samples/hwpx/para-001.hwpx"] {
        let doc = load(rel);
        assert_eq!(char_fill(&doc, 0, 0).0, "none", "{rel}: 글자");
        assert_eq!(para_fill(&doc, 0).0, "none", "{rel}: 문단");
    }
}

#[test]
fn hancom_hwp_cell_fill() {
    // 첫 문단의 표(컨트롤 5) 셀 0: 면 색 0xFFFFFFFF, 무늬 없음
    let doc = load("samples/table-complex.hwp");
    let json = doc.get_cell_properties(0, 0, 5, 0).unwrap();
    assert_eq!(fill(&json).0, "none", "면 색 없음");

    // 면 색 0xFFFFFFFF 에 무늬(3)가 있는 셀: 렌더러가 무늬를 그리므로 채우기가 있다.
    let doc = load("samples/한글문서파일형식_5.0_revision1.3.hwp");
    let json = doc.get_cell_properties(3, 133, 3, 0).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["fillType"], "solid", "면 색 없음 + 무늬");
    assert_eq!(v["patternType"], 3, "면 색 없음 + 무늬");
}

#[test]
fn real_white_stays_solid_white() {
    let white = ("solid".to_string(), "#ffffff".to_string());

    // 한컴이 저장한 흰색 셀: 면 색 0x00FFFFFF
    let doc = load("samples/issue1937_rowbreak_footnote_overpagination.hwp");
    let json = doc.get_cell_properties(0, 37, 0, 3).unwrap();
    assert_eq!(fill(&json), white, "한컴 흰색 셀");

    // API 로 지정한 흰색 배경
    let mut doc = blank();
    doc.insert_text_native(0, 0, 0, "가나다").unwrap();
    let props = r##"{"fillType":"solid","fillColor":"#ffffff"}"##;
    doc.apply_para_format_native(0, 0, props).unwrap();
    doc.apply_char_format_native(0, 0, 0, 3, props).unwrap();
    for (label, doc) in reopen_all(doc) {
        assert_eq!(char_fill(&doc, 0, 1), white, "{label}: 글자");
        assert_eq!(para_fill(&doc, 0), white, "{label}: 문단");
    }
}
