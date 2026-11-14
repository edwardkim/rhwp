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
