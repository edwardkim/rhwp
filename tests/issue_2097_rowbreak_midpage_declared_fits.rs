//! #2097 중간 쪽 RowBreak 표의 짧은 저장 셀을 독립 출력으로 재검토한다.
//!
//! 이 합성 입력의 마지막 셀500HU는 저장 줄1200HU와 위아래 여백282HU를
//! 담지 못한다. 실측 초과16px 이하를 모두 측정 노이즈로 취급하던 기대는
//! 유효한 저장 프레임 증거가 아니었다. 동일 입력의 한컴2020 PDF는 큰 행과
//! 중간 행을1쪽, 마지막 행과 뒤 문단을2쪽에 둔다.
//! 독립 기준: pdf/issue2097/rowbreak-midpage-original-2020.pdf.
//! 실제 행·내용의 단일 소유와 물리 끝은 tests/cases의 같은 사례에서 검사한다.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/task2097/rowbreak_midpage_declared_fits.hwpx";

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e))
}

#[test]
fn issue_2097_short_terminal_cell_follows_hangul_page_ownership() {
    let doc = load_doc();
    assert_eq!(doc.page_count(), 2, "동일 입력 한컴2020 PDF의 전체2쪽");
    for page in 0..2 {
        assert!(
            doc.dump_page_items(Some(page)).contains("PartialTable"),
            "한컴처럼 마지막 행을2쪽으로 이월해야 함: p{}",
            page + 1
        );
    }
}
