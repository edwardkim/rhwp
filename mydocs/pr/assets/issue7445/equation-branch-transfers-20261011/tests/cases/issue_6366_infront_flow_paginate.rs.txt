//! [Issue #6366] 글 앞으로(`IN_FRONT_OF_TEXT`) 표가 `flowWithText=1` 이면
//! 쪽 분할에 참여한다.
//!
//! `samples/issue5792/2700727_animal_facility_standards.hwpx` 의 pi=9 표
//! (42행, 글 앞으로) 가 Page 직속 Shape 로만 올라가면 단 높이 525px 만 찬
//! 것으로 보고 쪼개지지 않는다. 한글 2020 정답지는 그 꼬리를 단독 쪽으로
//! 나누어 6쪽이다. 텍스트 총량 6,769 자는 같고 배치만 다르다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use std::path::Path;

const SAMPLE: &str = "samples/issue5792/2700727_animal_facility_standards.hwpx";

#[test]
fn issue_6366_infront_flow_table_matches_hangul_six_pages() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(&path).expect("read sample")).expect("open");
    assert_eq!(
        core.page_count(),
        6,
        "한글 2020 정답지 pdf/pr_open_20260821/2700727_animal_facility_standards-2020.pdf 는 6쪽 (결함 시 5쪽)"
    );
}
