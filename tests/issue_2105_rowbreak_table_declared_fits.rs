//! #2105 쪽 시작 RowBreak 합성 표의 짧은 마지막 셀을 독립 출력으로 검사한다.
//!
//! 마지막 셀500HU는 저장 줄1200HU와 안여백282HU를 담지 못한다.
//! 선언69700HU가 본문에 들어간다는 사실만으로 가시 내용도 통째 수용한다고
//! 기대할 수 없다. 동일 입력의 한컴2020 PDF는 큰 행/중간 행을1쪽,
//! 마지막 행과 뒤 문단을2쪽에 둔다.
//! 독립 기준: pdf/issue2097/rowbreak-fragment-start-original-2020.pdf.
//! 실제 행 소유/물리 끝은 tests/cases의 공통 경계 검사에서 확인한다.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/task2105/rowbreak_table_declared_fits.hwpx";

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e))
}

#[test]
fn issue_2105_short_terminal_cell_at_fragment_start_follows_hangul() {
    let doc = load_doc();
    assert_eq!(doc.page_count(), 2, "동일 입력 한컴2020 PDF2쪽");
    for page in 0..2 {
        assert!(
            doc.dump_page_items(Some(page)).contains("PartialTable"),
            "쪽 시작 표도 실제 마지막 행을2쪽에 보존해야 함: p{}",
            page + 1
        );
    }
}
