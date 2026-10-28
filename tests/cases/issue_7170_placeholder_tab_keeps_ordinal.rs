//! [#7170] 저장 데이터가 없는 탭이 있어도 **뒤 탭의 확장이 밀리지 않는다.**
//!
//! `Paragraph::tab_extended` 는 문단 안 `'\t'` 순번으로 소비된다(서식기는 n 번째 탭에
//! n 번째 확장을 적는다). 두 서식기 모두 "저장 데이터 없음"을 폭 0 마커로 적는데
//! (HWP5 이진 `[0,…,0,0x0009]`, HWPX `<hp:tab width="0" leader="0" type="1"/>`),
//! 파서가 그 항목을 **버려서** 그 뒤 탭들이 한 칸씩 앞 확장을 쓰게 됐다.
//!
//! HWP3 는 탭마다 폭·점끌기를 인라인으로 담고(#7170 주 축, PR #7197) 폭·채움이 둘 다
//! 0 인 탭이 나올 수 있다. 그 탭 하나 때문에 같은 문단의 **멀쩡한 탭까지** 폭과 점끌기를
//! 통째로 잃던 것이 이 결함의 실제 피해다.
//!
//! 이 검사는 합성 계약 검사다 — 한컴 출력과의 일치 증거가 아니라, 자리표를 실은 뒤에도
//! 저장→재파스에서 탭 n 번째 확장이 n 번째 탭에 남는지를 고정한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::paragraph::{tab_ext_is_placeholder, TAB_EXT_PLACEHOLDER};

/// 두 번째·세 번째 탭이 실제로 담고 있는 저장값 (폭 HWPUNIT, 채움 3 = 점선).
const SECOND_TAB: [u16; 7] = [30000, 0, 0x0003, 0, 0, 0, 0x0009];
const THIRD_TAB: [u16; 7] = [40000, 0, 0x0203, 0, 0, 0, 0x0009];

/// `가\t나\t다\t53` 한 문단짜리 문서. 첫 탭만 저장 데이터가 없다.
fn document_with_placeholder_first_tab() -> rhwp::model::document::Document {
    let mut doc = rhwp::model::document::Document::default();
    // HWPX 서식기는 `charPrIDRef`/`paraPrIDRef` 참조가 DocInfo 에 있어야 내보낸다.
    doc.doc_info
        .char_shapes
        .push(rhwp::model::style::CharShape::default());
    doc.doc_info
        .para_shapes
        .push(rhwp::model::style::ParaShape::default());
    doc.doc_info
        .border_fills
        .push(rhwp::model::style::BorderFill::default());
    let mut section = rhwp::model::document::Section::default();
    let text = "가\t나\t다\t53";

    let mut para = rhwp::model::paragraph::Paragraph::new_empty();
    // 탭은 PARA_TEXT 에서 8 code unit 을 차지한다 — char_offsets/char_count 를 그 축으로 센다.
    let mut offsets = Vec::new();
    let mut units: u32 = 0;
    for ch in text.chars() {
        offsets.push(units);
        units += if ch == '\t' { 8 } else { ch.len_utf16() as u32 };
    }
    para.text = text.to_string();
    para.char_offsets = offsets;
    para.char_count = units;
    para.has_para_text = true;
    para.char_shapes = vec![rhwp::model::paragraph::CharShapeRef {
        start_pos: 0,
        char_shape_id: 0,
    }];
    para.tab_extended = vec![TAB_EXT_PLACEHOLDER, SECOND_TAB, THIRD_TAB];

    section.paragraphs.push(para);
    doc.sections.push(section);
    doc
}

fn roundtrip_tab_extended(doc: &rhwp::model::document::Document) -> (String, Vec<[u16; 7]>) {
    let bytes = rhwp::serializer::cfb_writer::serialize_hwp(doc).expect("HWP5 직렬화");
    let parsed = rhwp::parser::parse_document(&bytes).expect("HWP5 재파싱");
    let para = parsed
        .sections
        .first()
        .and_then(|s| s.paragraphs.first())
        .expect("문단 없음");
    (para.text.clone(), para.tab_extended.clone())
}

#[test]
fn placeholder_tab_does_not_shift_following_tab_extensions() {
    let (text, tabs) = roundtrip_tab_extended(&document_with_placeholder_first_tab());

    assert_eq!(text, "가\t나\t다\t53", "본문 글자가 왕복에서 달라졌다");
    assert_eq!(
        tabs.len(),
        3,
        "탭 3개인데 확장이 {}개다 — 자리표를 버리면 뒤 탭이 밀린다: {tabs:?}",
        tabs.len()
    );
    assert!(
        tab_ext_is_placeholder(&tabs[0]),
        "첫 탭은 저장 데이터가 없어야 한다: {:?}",
        tabs[0]
    );
    assert_eq!(
        tabs[1], SECOND_TAB,
        "두 번째 탭이 자기 폭·점끌기를 지켜야 한다"
    );
    assert_eq!(
        tabs[2], THIRD_TAB,
        "세 번째 탭이 자기 폭·점끌기를 지켜야 한다"
    );
}

/// HWPX 왕복도 같은 계약이다 — 서식기는 자리표를 `width="0" leader="0" type="1"` 마커로,
/// 파서는 그 마커를 다시 자리표로 옮긴다.
#[test]
fn placeholder_tab_keeps_ordinal_through_hwpx_roundtrip() {
    // HWPX 서식기는 `charPrIDRef` 등 DocInfo 참조를 요구한다 — HWP5 왕복으로 기본
    // DocInfo 가 채워진 문서를 HWPX 로 다시 내보낸다.
    let hwp5 = rhwp::serializer::cfb_writer::serialize_hwp(&document_with_placeholder_first_tab())
        .expect("HWP5 직렬화");
    let mut doc = rhwp::parser::parse_document(&hwp5).expect("HWP5 재파싱");
    // HWPX 서식기의 마커 표기(`width=0 leader=0 type=1`)를 그대로 들려 보낸다 — 이 표기를
    // 버리던 것이 HWPX 쪽 순번 밀림의 자리다.
    doc.sections[0].paragraphs[0].tab_extended[0] = [0, 0, 0x0100, 0, 0, 0, 0];
    let bytes = rhwp::serializer::hwpx::serialize_hwpx(&doc).expect("HWPX 직렬화");
    let parsed = rhwp::parser::hwpx::parse_hwpx(&bytes).expect("HWPX 재파싱");
    let para = parsed
        .sections
        .first()
        .and_then(|s| s.paragraphs.first())
        .expect("문단 없음");
    let tabs = &para.tab_extended;
    assert_eq!(
        tabs.len(),
        3,
        "HWPX 왕복에서 확장이 {}개다 — 자리표를 버리면 뒤 탭이 밀린다: {tabs:?}",
        tabs.len()
    );
    assert!(tab_ext_is_placeholder(&tabs[0]), "첫 탭: {:?}", tabs[0]);
    assert_eq!(tabs[1][0], SECOND_TAB[0], "두 번째 탭 폭");
    assert_eq!(tabs[1][2] & 0x00FF, 3, "두 번째 탭 점끌기");
    assert_eq!(tabs[2][0], THIRD_TAB[0], "세 번째 탭 폭");
    assert_eq!(tabs[2][2], THIRD_TAB[2], "세 번째 탭 종류·점끌기");
}

/// 자리표는 저장 폭이 아니다 — 폭 0 을 결과 위치로 읽으면 탭이 무폭이 된다(#1892).
#[test]
fn placeholder_is_not_a_stored_width() {
    assert!(tab_ext_is_placeholder(&TAB_EXT_PLACEHOLDER));
    // HWPX 서식기의 마커 표기(`width=0 leader=0 type=1`) 도 같은 자리표다.
    assert!(tab_ext_is_placeholder(&[0, 0, 0x0100, 0, 0, 0, 0]));
    assert!(!tab_ext_is_placeholder(&SECOND_TAB));
    assert!(!tab_ext_is_placeholder(&THIRD_TAB));
}
