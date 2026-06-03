//! 편집 삽입 탭의 HWP 직렬화 시 tab_extended 마커 검증 — issue #1244
//!
//! 증상: rhwp-studio에서 탭 삽입 후 HWP 저장 시 tab_extended가 Vec::new() 상태로
//! 폴백이 [0,0,0,0,0,0,0]으로 직렬화되어 ext[6]=0이 됨.
//! 한컴 편집기는 ext[6]≠0x0009이면 탭을 인식하지 못해 탭이 소멸.
//!
//! 수정: 폴백을 [0,0,0,0,0,0,0x0009]로 변경하여 마커를 올바르게 출력.

use rhwp::document_core::DocumentCore;

/// blank2010.hwp를 로드한다.
fn load_blank() -> Vec<u8> {
    std::fs::read("saved/blank2010.hwp").expect("saved/blank2010.hwp 없음")
}

/// 편집으로 삽입한 탭이 HWP 직렬화 후 재파싱 시 ext[6]=0x0009를 갖는지 확인한다.
#[test]
fn issue_1244_inserted_tab_has_marker_after_roundtrip() {
    let blank = load_blank();
    let mut core = DocumentCore::from_bytes(&blank).expect("blank 로드 실패");

    // 탭 문자 삽입 (section=0, para=0, offset=0)
    core.insert_text_native(0, 0, 0, "\t")
        .expect("탭 삽입 실패");

    // HWP 직렬화
    let hwp_bytes = core.export_hwp_native().expect("HWP 직렬화 실패");

    // 재파싱 — Document IR에 직접 접근하여 tab_extended 검증
    let doc = rhwp::parser::parse_hwp(&hwp_bytes).expect("재파싱 실패");
    let para = &doc.sections[0].paragraphs[0];

    assert!(
        !para.tab_extended.is_empty(),
        "탭을 삽입했으나 tab_extended가 비어 있음 — 직렬화 누락"
    );
    assert_eq!(
        para.tab_extended[0][6], 0x0009,
        "ext[6]이 0x0009가 아님: {:?}\n\
         한컴 편집기는 ext[6]=0x0009 마커가 없으면 탭을 인식하지 못함 (issue #1244)",
        para.tab_extended[0]
    );
}

/// 탭 여러 개를 삽입했을 때 모든 tab_extended 항목에 마커가 있는지 확인한다.
#[test]
fn issue_1244_multiple_inserted_tabs_all_have_marker() {
    let blank = load_blank();
    let mut core = DocumentCore::from_bytes(&blank).expect("blank 로드 실패");

    core.insert_text_native(0, 0, 0, "가\t나\t다")
        .expect("삽입 실패");

    let hwp_bytes = core.export_hwp_native().expect("HWP 직렬화 실패");
    let doc = rhwp::parser::parse_hwp(&hwp_bytes).expect("재파싱 실패");
    let para = &doc.sections[0].paragraphs[0];

    assert_eq!(
        para.tab_extended.len(),
        2,
        "탭 2개를 삽입했으나 tab_extended 항목 수가 다름: {}",
        para.tab_extended.len()
    );
    for (i, ext) in para.tab_extended.iter().enumerate() {
        assert_eq!(
            ext[6], 0x0009,
            "tab_extended[{i}][6]이 0x0009가 아님: {ext:?} (issue #1244)"
        );
    }
}
