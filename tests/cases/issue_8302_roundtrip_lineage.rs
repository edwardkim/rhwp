//! [Issue #8302] 형식을 오가는 왕복 저장이 원래 계보를 뒤집지 않는다.
//!
//! rhwp 는 HWP5 → HWPX 산출에 `META-INF/rhwp-hwp5-origin`, HWPX → HWP 산출에
//! `/RhwpHwpxOrigin` 을 남겨 다시 열 때의 조판 호환 계약을 고른다. 종전에는 저장
//! 경로가 **지금 형식**만 보고 반대 계보 표식을 새로 붙였다.
//!
//! - HWP5 → HWPX → HWP: HWP5 계보 문서가 `hwpx_lineage` 로 다시 열려 2912735 가
//!   원본·한컴 Print PDF(`pdf/2912735_court_report_form-2020.pdf`) 2쪽에서 1쪽이 됐다.
//! - HWPX → HWP → HWPX: HWPX 계보 문서가 HWP5-origin HWPX 로 다시 열렸다.
//!   worklife_balance_index_156607916 은 원본·한컴 Print PDF
//!   (`pdf/planet-review-20260917/worklife_balance_index_156607916-2020.pdf`) 6쪽에서
//!   7쪽이 됐다(HWPX → HWP 한 단계의 저장 줄 계약은 #8310).

use rhwp::document_core::DocumentCore;
use rhwp::model::document::{HWP5_ORIGIN_HWPX_MARKER_PATH, HWPX_ORIGIN_STREAM_PATH};

fn load(path: &str) -> DocumentCore {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{path} 읽기 실패: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("{path} 파싱 실패: {e}"))
}

/// 쪽마다 본문 글 — 쪽 소속·순서·누락·중복을 함께 본다.
fn page_texts(core: &DocumentCore) -> Vec<String> {
    (0..core.page_count() as usize)
        .map(|page| core.page_text(page).expect("쪽 글"))
        .collect()
}

fn has_hwpx_lineage_marker(core: &DocumentCore) -> bool {
    core.document()
        .extra_streams
        .iter()
        .any(|(path, _)| path == HWPX_ORIGIN_STREAM_PATH)
}

fn has_hwp5_origin_marker(core: &DocumentCore) -> bool {
    core.document()
        .hwpx_aux_entry(HWP5_ORIGIN_HWPX_MARKER_PATH)
        .is_some()
}

fn reopen_as_hwpx(core: &DocumentCore) -> DocumentCore {
    DocumentCore::from_bytes(&core.export_hwpx_native().expect("HWPX 저장"))
        .expect("HWPX 다시 열기")
}

fn reopen_as_hwp(core: &DocumentCore) -> DocumentCore {
    DocumentCore::from_bytes(&core.export_hwp_with_adapter().expect("HWP 저장"))
        .expect("HWP 다시 열기")
}

/// HWP5 원본을 HWPX 로 저장한 뒤 그 HWPX 를 다시 HWP 로 저장해도 HWP5 계보로 열린다.
#[test]
fn hwp5_roundtrip_through_hwpx_keeps_hwp5_lineage_and_pages() {
    let original = load("samples/issue6034/2912735_court_report_form.hwp");
    assert_eq!(
        original.page_count(),
        2,
        "원본은 한컴 Print PDF 와 같은 2쪽"
    );

    let hwpx = reopen_as_hwpx(&original);
    assert!(has_hwp5_origin_marker(&hwpx));

    let roundtrip = reopen_as_hwp(&hwpx);
    assert!(
        !has_hwpx_lineage_marker(&roundtrip),
        "HWP5 계보 문서에 HWPX 계보 표식이 붙으면 안 된다"
    );
    assert!(!roundtrip.document().provenance.hwpx_lineage);
    assert_eq!(
        page_texts(&roundtrip),
        page_texts(&original),
        "왕복본의 쪽 수·쪽별 본문이 원본과 같아야 한다"
    );
}

/// 한컴 HWPX 를 HWP 로 저장한 뒤 그 HWP 를 다시 HWPX 로 저장해도 HWPX 계보로 열린다.
#[test]
fn hwpx_roundtrip_through_hwp_keeps_hwpx_lineage_and_pages() {
    let original = load("samples/issue6145/worklife_balance_index_156607916.hwpx");
    assert_eq!(
        original.page_count(),
        6,
        "원본은 한컴 Print PDF 와 같은 6쪽"
    );

    let hwp = reopen_as_hwp(&original);
    assert!(
        has_hwpx_lineage_marker(&hwp) && hwp.document().provenance.hwpx_lineage,
        "표식 없는 한컴 HWPX 에서 만든 HWP 는 HWPX 계보로 열린다"
    );

    let roundtrip = reopen_as_hwpx(&hwp);
    assert!(
        !has_hwp5_origin_marker(&roundtrip),
        "HWPX 계보 문서에 HWP5-origin 표식이 붙으면 안 된다"
    );
    assert_eq!(
        page_texts(&roundtrip),
        page_texts(&original),
        "왕복본의 쪽 수·쪽별 본문이 원본과 같아야 한다"
    );
}

/// 반례: native HWP5 를 HWPX 로 저장하면 여전히 HWP5-origin 표식을 붙인다.
#[test]
fn native_hwp5_export_still_marks_hwp5_origin() {
    let original = load("samples/issue6034/2912735_court_report_form.hwp");
    assert!(!original.document().provenance.hwpx_lineage);
    assert!(has_hwp5_origin_marker(&reopen_as_hwpx(&original)));
}
