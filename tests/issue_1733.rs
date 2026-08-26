//! Issue #1733: 국제고속선기준 tail/vpos-reset 잔여 over-pagination 회귀 방지.
//!
//! HWP 2020 MCP PDF 기준인 242쪽을 HWP와 HWPX 모두 유지한다. HWPX의 일반
//! 텍스트 LINE_SEG 0 vpos는 writer-local 재시작일 수 있으므로, 그 표식을
//! 무조건 물리 쪽 경계로 확장해 tail-only 쪽을 만들지 않아야 한다.

use rhwp::wasm_api::HwpDocument;
use std::fs;
use std::path::Path;

const HANCOM_PDF_PAGE_COUNT: u32 = 242;

fn load_doc(sample: &str) -> HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(sample);
    let bytes = fs::read(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|err| panic!("parse {}: {err:?}", path.display()))
}

fn assert_hancom_pdf_page_count(sample: &str) {
    let doc = load_doc(sample);
    assert_eq!(
        doc.page_count(),
        HANCOM_PDF_PAGE_COUNT,
        "{sample} should match the HWP 2020 MCP PDF page-count oracle"
    );
}

fn assert_hwpx_reset_fragment_keeps_its_source_page() {
    let doc = load_doc("samples/task1725/text_footnote_tail_overpagination.hwpx");
    let source_page = doc.dump_page_items(Some(56));
    let following_page = doc.dump_page_items(Some(57));

    assert!(
        source_page.contains("PartialParagraph  pi=1217  lines=0..3"),
        "HWPX reset 전 fragment는 저장된 57쪽 owner에 남아야 한다\n{source_page}"
    );
    assert!(
        following_page.contains("PartialParagraph  pi=1217  lines=3..5")
            && following_page.contains("FullParagraph  pi=1218"),
        "reset 다음 쪽은 pi=1217 tail과 후속 본문을 함께 가져야 한다\n{following_page}"
    );

    let local_cursor_page = doc.dump_page_items(Some(219));
    assert!(
        local_cursor_page.contains("FullParagraph  pi=4726")
            && local_cursor_page.contains("FullParagraph  pi=4731"),
        "현재 flow anchor와 맞지 않는 HWPX local reset은 별도 물리 쪽을 만들면 안 된다\n{local_cursor_page}"
    );
}

#[test]
fn issue_1733_hwpx_matches_hancom_pdf_page_count() {
    assert_hancom_pdf_page_count("samples/task1725/text_footnote_tail_overpagination.hwpx");
    assert_hwpx_reset_fragment_keeps_its_source_page();
}

#[test]
fn issue_1733_hwp_matches_hancom_pdf_page_count() {
    assert_hancom_pdf_page_count("samples/task1725/text_footnote_tail_overpagination.hwp");
}
