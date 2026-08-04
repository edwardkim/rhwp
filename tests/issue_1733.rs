//! Issue #1733: 국제고속선기준 tail/vpos-reset 잔여 over-pagination 회귀 방지.
//!
//! [#2559 트레이드] 한컴 2024/PDF 기준은 242쪽이지만, 빈 꼬리말 밴드를 각주에
//! 회수한 뒤 두 포맷 모두 241쪽이 된다. 각주가 있어도 한컴이 밴드를 본문에
//! 전부 내주지 않는 경계 문서이므로, 이 차이는 현재 알고 있는 잔여다. #2559의
//! 대표 과다분할 완화 효과를 되돌리지 않도록 241쪽을 명시적으로 고정하며, 후속
//! 각주-꼬리말 세분화에서 한컴 기준 242쪽으로 복원할 대상이다.

use rhwp::wasm_api::HwpDocument;
use std::fs;
use std::path::Path;

const HANCOM_PDF_PAGE_COUNT: u32 = 242;
const CURRENT_PAGE_COUNT_PIN: u32 = 241;

fn load_doc(sample: &str) -> HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(sample);
    let bytes = fs::read(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|err| panic!("parse {}: {err:?}", path.display()))
}

fn assert_current_page_count_pin(sample: &str) {
    let doc = load_doc(sample);
    assert_eq!(
        doc.page_count(),
        CURRENT_PAGE_COUNT_PIN,
        "{sample} should retain the documented #2559 page-count pin; HWP 2024/PDF oracle is {HANCOM_PDF_PAGE_COUNT}"
    );
}

#[test]
fn issue_1733_hwpx_retains_documented_page_count_pin() {
    assert_current_page_count_pin("samples/task1725/text_footnote_tail_overpagination.hwpx");
}

#[test]
fn issue_1733_hwp_retains_documented_page_count_pin() {
    assert_current_page_count_pin("samples/task1725/text_footnote_tail_overpagination.hwp");
}
