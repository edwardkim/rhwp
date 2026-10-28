//! [Issue #6872] HWPX 저장이 각주 numbering 토큰과 빈 장식 문자를 원본대로 보존하는지 고정한다.
//!
//! 두 축 모두 **한글을 정답지로 세워야만** 보인다 — rhwp 파서가 자기가 낸 토큰을 도로
//! 받아주기 때문에(`"ON_PAGE" | "RESTART_PAGE" | "restartPage"`) 자기 왕복 검증은 통과한다.
//!
//! - 축 1: `RESTART_PAGE`/`RESTART_SECTION` 은 한컴이 한 번도 쓰지 않는 토큰이다(원본 HWPX
//!   3,391 파일 실측: `CONTINUOUS` 7,640 · `ON_PAGE` 12 · `RESTART_*` 0). 한글은 이것을
//!   못 알아듣고 각주 번호를 연속으로 매긴다 — 쪽마다 `1)` 이던 것이 `1) 2) 3) …` 이 된다.
//! - 축 2: 원본이 `suffixChar=""` 로 비운 접미를 템플릿 기본값 `)` 로 되돌려, 사용자 기호
//!   각주 `*` 가 `*)` 가 된다. 다만 `'\0'` 하나로는 "원본이 비웠다"와 "IR 미설정"을 못 가르므로
//!   (`#2742` 는 후자에서 템플릿 문자열 유지를 요구한다) `deco_chars_from_source` 로 가른다.
//!
//! `src/` 안 `#[cfg(test)]` 총량은 래칫으로 묶여 있어(`rust-unit-test-tiers`) 통합 시험으로 둔다.
//! 대상 함수(`note_numbering_str`·`render_auto_num_format`)는 private 이라 공개 경로
//! `serialize_hwpx` 로 방출 XML 을 직접 확인한다 — 실제 저장 경로와 같은 축이다.

#![cfg(not(target_arch = "wasm32"))]

use std::io::Read;

use rhwp::model::document::Document;
use rhwp::model::footnote::{FootnoteNumbering, NumberFormat};
use rhwp::serializer::hwpx::serialize_hwpx;

/// 저장한 HWPX 에서 첫 구역 XML 을 꺼낸다.
fn section0_xml(doc: &Document) -> String {
    let bytes = serialize_hwpx(doc).expect("HWPX 직렬화");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("zip 열기");
    let mut f = zip
        .by_name("Contents/section0.xml")
        .expect("Contents/section0.xml");
    let mut s = String::new();
    f.read_to_string(&mut s).expect("section0.xml 읽기");
    s
}

fn doc_with_one_section() -> Document {
    let mut doc = Document::default();
    if doc.sections.is_empty() {
        doc.sections.push(Default::default());
    }
    doc
}

#[test]
fn issue6872_numbering_emits_hancom_tokens() {
    let mut doc = doc_with_one_section();
    doc.sections[0].section_def.footnote_shape.numbering = FootnoteNumbering::RestartPage;
    doc.sections[0].section_def.endnote_shape.numbering = FootnoteNumbering::RestartSection;
    let xml = section0_xml(&doc);

    assert!(
        xml.contains(r#"<hp:numbering type="ON_PAGE""#),
        "쪽마다 재시작은 한컴 토큰 ON_PAGE 로 나가야 한다: {xml:.400}"
    );
    assert!(
        xml.contains(r#"<hp:numbering type="ON_SECTION""#),
        "구역마다 재시작은 ON_SECTION 으로 나가야 한다: {xml:.400}"
    );
    assert!(
        !xml.contains("RESTART_PAGE") && !xml.contains("RESTART_SECTION"),
        "한컴이 쓰지 않는 토큰을 내보내면 한글이 연속 번호로 떨어진다: {xml:.400}"
    );
}

#[test]
fn issue6872_source_empty_suffix_char_stays_empty() {
    let mut doc = doc_with_one_section();
    let fs = &mut doc.sections[0].section_def.footnote_shape;
    fs.number_format = NumberFormat::UserChar;
    fs.user_char = '*';
    fs.suffix_char = '\0';
    // 원본 HWPX 가 `suffixChar=""` 로 명시적으로 비운 경우.
    fs.deco_chars_from_source = true;
    let xml = section0_xml(&doc);

    assert!(
        xml.contains(r#"type="USER_CHAR" userChar="*" prefixChar="" suffixChar="""#),
        "원본이 비운 접미는 빈 채로 나가야 한다(`*` 가 `*)` 가 되지 않게): {xml:.600}"
    );
}

#[test]
fn issue6872_unset_ir_keeps_template_suffix() {
    // [#2742] 규약 — `'\0'` 이 **미지정**이면 템플릿 기본값 `)` 를 지킨다.
    // 축 2 수정이 이 계약을 깨지 않는지 같은 자리에서 고정한다.
    let doc = doc_with_one_section();
    let xml = section0_xml(&doc);

    assert!(
        xml.contains(r#"suffixChar=")""#),
        "IR 미설정은 템플릿 기본값 `)` 를 유지한다(#2742): {xml:.600}"
    );
    assert!(
        !xml.contains(r#"suffixChar="""#),
        "미설정을 빈 값으로 내보내면 #2742 계약이 깨진다: {xml:.600}"
    );
}
