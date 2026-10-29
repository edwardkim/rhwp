//! HWP5 원본에서 만든 HWPX 도 쪽을 넘는 1행 표를 원본 HWP 와 같은 자리에서 나눈다.
//!
//! `hwpx_sample2.hwp` 19쪽의 1×1 RowBreak 표(문단 182)는 선언 높이(74361HU, 991.5px)보다
//! 셀 내용(88696HU)이 크다. 한컴 2024 는 원본 HWP 도, rhwp 가 저장한 HWPX 도 이 표를 19쪽
//! 끝에서 나눠 20쪽을 "⑧ 선착순의 방법으로…" 로 시작한다(`pdf/hwpx_sample2-2024.pdf`,
//! `pdf/hwpx_sample2-rhwp-hwpx-2024.pdf`). 종전 rhwp 는 HWP5 출처 HWPX 에만 #1891 의 1행
//! 선언 높이 특례를 적용해 표를 통째로 19쪽에 두고 20쪽 내용까지 그 쪽에 그렸다.

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/hwpx_sample2.hwp";

fn open(bytes: &[u8]) -> DocumentCore {
    DocumentCore::from_bytes(bytes).expect("문서 열기")
}

/// 쪽마다 공백을 뺀 본문.
fn page_texts(core: &DocumentCore) -> Vec<String> {
    (0..core.page_count())
        .map(|page| {
            core.extract_page_text_native(page)
                .expect("쪽 본문")
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect()
        })
        .collect()
}

#[test]
fn hwp5_origin_hwpx_splits_the_tall_single_row_table_like_the_source() {
    let source = open(&std::fs::read(SAMPLE).expect("샘플 읽기"));
    let hwpx = open(&source.export_hwpx_native().expect("HWPX 저장"));

    let source_pages = page_texts(&source);
    let hwpx_pages = page_texts(&hwpx);
    assert_eq!(source_pages.len(), 29, "한컴 PDF 29쪽");
    assert_eq!(hwpx_pages.len(), source_pages.len(), "HWPX 저장 뒤 쪽수");

    // 한컴 Print 의 쪽 경계: 19쪽은 표 셀의 "…해당 지자체(시･군･자치구)로부터 받아서 제출" 로
    // 끝나고, 20쪽은 같은 셀의 ⑧ 항목으로 시작한다.
    for (label, pages) in [("원본 HWP", &source_pages), ("HWPX 저장본", &hwpx_pages)] {
        assert!(
            pages[18].contains("부터받아서제출") && !pages[18].contains("⑧선착순의방법으로"),
            "{label} 19쪽: 표가 19쪽 끝에서 나뉘어야 한다"
        );
        assert!(
            pages[19].starts_with("⑧선착순의방법으로"),
            "{label} 20쪽: 표의 남은 셀 내용으로 시작해야 한다"
        );
    }

    // 쪽 소속 전체가 원본과 같다 — 표 내용의 누락·중복 없이 같은 쪽에 놓인다.
    for (page, (a, b)) in source_pages.iter().zip(&hwpx_pages).enumerate() {
        assert_eq!(a, b, "{}쪽 본문", page + 1);
    }
}
