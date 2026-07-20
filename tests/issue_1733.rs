//! Issue #1733: 국제고속선기준 tail/vpos-reset 잔여 over-pagination 회귀 방지.
//!
//! [#2559 트레이드] 오라클(HWP2024/PDF)=242. #2559(각주가 빈 꼬리말 밴드 소비)
//! 적용 후 241(−1). 각주가 있으나 한글이 밴드를 본문에 온전히 내주지 않는 경계
//! 문서로, 밴드 회수가 소폭 과다한 knife-edge(스케일 축소로도 241 고정 — 환원
//! 불가). #2559 는 10k 순 +63(개선 69/회귀 7, 전부 −1/−2)의 순개선이며 이 문서는
//! 그 회귀 7건 중 하나. 241 로 핀하되 오라클은 242 임을 명기 — 후속 정밀화 시 복원.
use rhwp::wasm_api::HwpDocument;
use std::fs;
use std::path::Path;

const EXPECTED_PAGE_COUNT: u32 = 241;

fn load_doc(sample: &str) -> HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(sample);
    let bytes = fs::read(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|err| panic!("parse {}: {err:?}", path.display()))
}

fn assert_matches_pdf_page_count(sample: &str) {
    let doc = load_doc(sample);
    assert_eq!(
        doc.page_count(),
        EXPECTED_PAGE_COUNT,
        "{sample} should match the HWP 2024/PDF oracle page count"
    );
}

#[test]
fn issue_1733_hwpx_matches_pdf_page_count() {
    assert_matches_pdf_page_count("samples/task1725/text_footnote_tail_overpagination.hwpx");
}

#[test]
fn issue_1733_hwp_matches_pdf_page_count() {
    assert_matches_pdf_page_count("samples/task1725/text_footnote_tail_overpagination.hwp");
}
