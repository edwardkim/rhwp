//! [Issue #8310] 저장 줄이 일부만 있는 HWPX 를 HWP 로 저장해도 원본 HWPX 와 같은
//! 쪽 소속으로 다시 열린다.
//!
//! HWPX 로드는 저장 줄 없는 문단을 합성하고 구역 vpos 를 다시 계산한 뒤 원본 좌표를
//! `source_line_seg_vertical_pos` 로 남긴다(#5847). 종전 HWP 저장은 재계산 좌표와 합성 줄
//! (`LineSeg::TAG_IMPLEMENTATION_PROPERTY`)을 그대로 파일에 써서, 다시 연 HWP 가 원본과
//! 다른 조판 입력이 됐다. worklife_balance_index_156607916 은 원본·한컴 Print PDF
//! (`pdf/planet-review-20260917/worklife_balance_index_156607916-2020.pdf`) 6쪽에서
//! HWP 저장본이 7쪽이 됐다.

use rhwp::document_core::DocumentCore;
use rhwp::model::paragraph::Paragraph;

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

fn body_paragraphs(core: &DocumentCore) -> impl Iterator<Item = &Paragraph> {
    core.document()
        .sections
        .iter()
        .flat_map(|section| section.paragraphs.iter())
}

/// HWPX → HWP 저장본은 원본 HWPX 와 같은 쪽 수·쪽별 본문으로 열린다.
#[test]
fn partially_stored_hwpx_saved_as_hwp_keeps_page_ownership() {
    let original = load("samples/issue6145/worklife_balance_index_156607916.hwpx");
    assert_eq!(
        original.page_count(),
        6,
        "원본은 한컴 Print PDF 와 같은 6쪽"
    );

    let hwp = DocumentCore::from_bytes(&original.export_hwp_with_adapter().expect("HWP 저장"))
        .expect("HWP 다시 열기");
    assert_eq!(
        page_texts(&hwp),
        page_texts(&original),
        "HWP 저장본의 쪽 수·쪽별 본문이 원본 HWPX 와 같아야 한다"
    );
}

/// 원본 저장 줄의 좌표는 HWP 파일을 거쳐도 그대로 남는다.
#[test]
fn source_vertical_positions_survive_hwp_save() {
    let original = load("samples/issue6145/worklife_balance_index_156607916.hwpx");
    let hwp = DocumentCore::from_bytes(&original.export_hwp_with_adapter().expect("HWP 저장"))
        .expect("HWP 다시 열기");

    let mut compared = 0;
    for (source, saved) in body_paragraphs(&original).zip(body_paragraphs(&hwp)) {
        let Some(source_vpos) = &source.source_line_seg_vertical_pos else {
            continue;
        };
        assert_eq!(
            saved.source_line_seg_vertical_pos.as_ref(),
            Some(source_vpos),
            "원본 저장 줄 좌표가 HWP 저장 뒤에도 같아야 한다: {:?}",
            source.text
        );
        compared += 1;
    }
    assert!(compared > 0, "비교한 원본 저장 줄 문단이 있어야 한다");
}

/// 반례: 저장 줄이 모두 있는 native HWP5 는 HWP 로 다시 저장해도 줄과 쪽이 그대로다.
#[test]
fn native_hwp5_resave_keeps_stored_lines() {
    let original = load("samples/issue6034/2912735_court_report_form.hwp");
    let resaved = DocumentCore::from_bytes(&original.export_hwp_native().expect("HWP 저장"))
        .expect("HWP 다시 열기");
    assert_eq!(page_texts(&resaved), page_texts(&original));
    for (a, b) in body_paragraphs(&original).zip(body_paragraphs(&resaved)) {
        assert_eq!(
            a.line_segs, b.line_segs,
            "저장 줄이 바뀌면 안 된다: {:?}",
            a.text
        );
    }
}
