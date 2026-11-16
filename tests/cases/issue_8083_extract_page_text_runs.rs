//! 관계형: render tree에서 보이는 본문·셀 텍스트의 내용 보존·순서·단일 출현.
//! 주석 갱신: 2026-11-14T19:53:53+09:00; 직전 파일 커밋: 2026-11-14T12:27:47+09:00, d268b792eda1aa678ac5d67d285455f6760a2754.
//! 텍스트 쿼리 계약이며 조판 좌표·페이지 수를 고정하지 않는다. 다른 입력의 시각 수치를 재인용하지 않는다. 근거: mydocs/pr/pr_9072_review.md.
//! #8083: `extract_page_text_native`가 독립 `TextRun`을 누락하지 않고 한 번씩만 추출하는지 검증한다.
//!
//! `TextLine`/`Equation` 외에 `Column`·페이지 루트 아래 직접 놓인 `TextRun`도 순서대로
//! 수집하고, 부모 노드 처리 후 중복 방문하지 않아야 한다.

use rhwp::document_core::DocumentCore;

#[test]
fn issue8083_tac_sample_text_preserves_rendered_run_order() {
    let core = DocumentCore::from_bytes(include_bytes!("../../samples/tac-case-001.hwp")).unwrap();
    let text = core.extract_page_text_native(0).unwrap();
    let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(text.contains("표 3 배치 시작"), "{text}");
    assert!(text.contains("표 다음"), "{text}");
    assert!(compact.contains("표3배치시작"), "{text}");
    assert!(compact.contains("표다음"), "{text}");

    let start_idx = compact.find("표3배치시작").unwrap();
    let after_idx = compact.find("표다음").unwrap();
    assert!(start_idx < after_idx, "{compact}");
    assert_eq!(compact.matches("표다음").count(), 1, "{compact}");
}
