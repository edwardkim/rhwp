//! 편집 삽입 탭의 HWP 직렬화 시 tab_extended 마커 검증 — issue #1244
//!
//! 증상: rhwp-studio에서 탭 삽입 후 HWP 저장 시 tab_extended가 Vec::new() 상태로
//! 폴백이 [0,0,0,0,0,0,0]으로 직렬화되어 ext[6]=0이 됨.
//! 한컴 편집기는 ext[6]≠0x0009이면 탭을 인식하지 못해 탭이 소멸.
//!
//! 수정: 폴백을 [0,0,0,0,0,0,0x0009]로 변경하여 마커를 올바르게 출력.

use rhwp::document_core::DocumentCore;
use rhwp::model::document::Document;
use rhwp::model::paragraph::tab_ext_is_placeholder;

/// blank2010.hwp를 로드한다.
fn load_blank() -> Vec<u8> {
    std::fs::read("saved/blank2010.hwp").expect("saved/blank2010.hwp 없음")
}

fn first_tab_extended(doc: &Document) -> [u16; 7] {
    for section in &doc.sections {
        for para in &section.paragraphs {
            if para.text.contains('\t') {
                return *para.tab_extended.first().unwrap_or_else(|| {
                    panic!(
                        "탭 문단을 찾았으나 tab_extended가 비어 있음: {:?}",
                        para.text
                    )
                });
            }
        }
    }
    panic!("문서에서 탭 문단을 찾지 못함")
}

/// 편집으로 삽입한 탭이 HWP 직렬화 후에도 탭으로 보존되는지 확인한다.
///
/// [#1892 계약 정제 / #7170 갱신] 직렬화기는 tab_extended 없는 탭에 null 마커
/// `[0,...,0,0x0009]` 를 방출한다(한컴이 탭으로 인식하는 데 필수 — #1244 원계약).
/// 파서는 그 마커를 **자리표로 실어** 탭 순번을 지키고(버리면 뒤 탭이 남의 확장을
/// 쓴다 — #7170), 소비자가 `tab_ext_is_placeholder` 로 걸러 저장 폭으로 읽지 않는다
/// (읽으면 레이아웃이 ext[0]=0 을 탭 결과 위치로 해석해 탭이 무폭 — #1892).
/// 직렬화기가 마커를 누락(ext[6]=0)하면 #1244 회귀이므로 ext[6] 도 함께 단언한다.
#[test]
fn issue_1244_inserted_tab_has_marker_after_roundtrip() {
    let blank = load_blank();
    let mut core = DocumentCore::from_bytes(&blank).expect("blank 로드 실패");

    // 탭 문자 삽입 (section=0, para=0, offset=0)
    core.insert_text_native(0, 0, 0, "\t")
        .expect("탭 삽입 실패");

    // HWP 직렬화
    let hwp_bytes = core.export_hwp_native().expect("HWP 직렬화 실패");

    // 재파싱 — Document IR에 직접 접근하여 검증
    let doc = rhwp::parser::parse_hwp(&hwp_bytes).expect("재파싱 실패");
    let para = &doc.sections[0].paragraphs[0];

    assert!(
        para.text.contains('\t'),
        "탭을 삽입했으나 재파싱 텍스트에 탭이 없음 — 마커 직렬화 누락 (issue #1244)"
    );
    assert_eq!(
        para.tab_extended.len(),
        1,
        "탭 1개의 자리표가 남아야 한다 (#7170 순번 계약): {:?}",
        para.tab_extended
    );
    assert_eq!(
        para.tab_extended[0][6], 0x0009,
        "직렬화기가 한컴이 요구하는 ext[6]=0x0009 마커를 빠뜨렸다 (#1244): {:?}",
        para.tab_extended
    );
    assert!(
        tab_ext_is_placeholder(&para.tab_extended[0]),
        "삽입 탭은 저장 폭이 없다 — 자리표로 읽혀야 한다 (#1892 탭 무폭 회귀 차단): {:?}",
        para.tab_extended
    );
}

/// 탭 여러 개를 삽입했을 때 전부 탭으로 보존되고 null 마커가 IR 로
/// 유입되지 않는지 확인한다 ([#1892 계약 정제] — 위 테스트 주석 참조).
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
        para.text.matches('\t').count(),
        2,
        "탭 2개를 삽입했으나 재파싱 텍스트의 탭 수가 다름: {:?}",
        para.text
    );
    assert_eq!(
        para.tab_extended.len(),
        2,
        "탭 2개의 자리표가 순번대로 남아야 한다 (#7170): {:?}",
        para.tab_extended
    );
    assert!(
        para.tab_extended.iter().all(tab_ext_is_placeholder),
        "삽입 탭은 저장 폭이 없다 — 전부 자리표여야 한다 (#1892): {:?}",
        para.tab_extended
    );
    assert!(
        para.tab_extended.iter().all(|ext| ext[6] == 0x0009),
        "한컴이 요구하는 ext[6]=0x0009 마커 누락 (#1244): {:?}",
        para.tab_extended
    );
}

/// HWPX에서 파싱된 탭 확장 정보가 HWP 저장 경로에서도 유지되는지 확인한다.
#[test]
fn issue_1244_hwpx_to_hwp_save_preserves_tab_extended_marker() {
    let hwpx_bytes = include_bytes!("../samples/hwpx/ref/ref_mixed.hwpx");

    let source_doc = rhwp::parser::hwpx::parse_hwpx(hwpx_bytes).expect("HWPX 파싱 실패");
    let source_ext = first_tab_extended(&source_doc);
    assert_eq!(
        source_ext[6], 0x0009,
        "HWPX 파서가 만든 tab_extended 마커가 0x0009가 아님: {source_ext:?}"
    );

    let mut core = DocumentCore::from_bytes(hwpx_bytes).expect("HWPX 로드 실패");
    let hwp_bytes = core.export_hwp_with_adapter().expect("HWPX→HWP 저장 실패");
    let saved_doc = rhwp::parser::parse_hwp(&hwp_bytes).expect("저장 HWP 재파싱 실패");
    let saved_ext = first_tab_extended(&saved_doc);

    assert_eq!(
        saved_ext[6], 0x0009,
        "HWPX→HWP 저장 후 ext[6] 마커가 0x0009가 아님: {saved_ext:?}"
    );
    assert_eq!(
        saved_ext[0], source_ext[0],
        "HWPX 탭 width가 HWP 저장 후 보존되지 않음: source={source_ext:?}, saved={saved_ext:?}"
    );
    assert_eq!(
        saved_ext[2], source_ext[2],
        "HWPX 탭 type/leader가 HWP 저장 후 보존되지 않음: source={source_ext:?}, saved={saved_ext:?}"
    );
}
