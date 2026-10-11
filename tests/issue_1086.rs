//! 이슈 #1086: HWP3 유래 문서의 쪽 나눔과 RowBreak·행 병합의 과도한 분할 회귀 검사.
//!
//! `samples/k-water-rfp.hwp`의 정상 한컴2022 PDF는 27쪽이다.
//! `samples/hwp3-sample16-hwp5.hwp`의 정상 한컴 HWP5 변환 출력은 64쪽이다.

use std::fs;
use std::path::Path;

fn page_count(rel_path: &str) -> usize {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel_path);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {rel_path}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {rel_path}: {e:?}"));
    doc.page_count() as usize
}

#[test]
fn task1086_k_water_rfp_page_count_matches_hancom_pdf() {
    assert_eq!(page_count("samples/k-water-rfp.hwp"), 27);
}

// hwpspec의 실패한 쪽수·쪽 소속 검사는 #7445로 이관했다.
// upstream/devel 74f9b71b1의 실제 출력도 6개 확인 쪽 모두 90% 미만이다.
// 원문/PDF와 정상 검사 보존 및 복원 조건: mydocs/pr/assets/issue7445/README.md.
