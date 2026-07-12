//! Issue #2019: 부동 개체(글상자·도형·표) 다수 별지 서식의 과분할 smoke guard.
//!
//! `samples/hwpx/issue2019_floating_form_74312.hwpx` (벤처투자 시행규칙 일부개정령안)은
//! 227개의 부동 글상자/도형(통과 wrap, tac=false) + 부동 표로 구성된 별지 서식 다수를
//! 담는다. 이들의 stored LINE_SEG vpos 는 텍스트 흐름 좌표가 아니라 개체의 섹션 절대 위치·
//! 높이를 인코딩한다.
//!
//! 회귀 (수정 전 버그, rhwp 81페이지 vs 한글 2022 18페이지 = 4.5× 과분할):
//! - ① `format_paragraph` 이 부동 앵커의 stored line_height(=개체 높이)를 흐름에 예약,
//! - ② 부동 폼 구분자(빈 문단 + 단나누기 + 같은 1단 ColumnDef)를 단일 단에서 페이지로 변환,
//! - ③ `process_multicolumn_break` 이 섹션 절대 vpos 를 zone 오프셋으로 사용 →
//!   candidate_offset 이 항상 페이지를 초과해 1단↔2단 zone 전환(71회)마다 새 페이지.
//!
//! 정정: `para_is_floating_overlay_anchor` 게이트로 세 경로에서 부동 폼 앵커의 흐름
//! footprint 를 0 으로 취급 + 누적 vpos(zone 높이가 페이지 초과)를 흐름 누적값으로 대체.
//!
//! 주의: 이 테스트는 한컴 PDF 시각 정합을 보장하지 않는다. 현재 수정은 81페이지 산란을
//! 막는 부분 완화이며, #2019 의 PDF 기준 시각 오정렬은 v3 작업에서 별도로 해결한다.

use std::fs;
use std::path::Path;

#[test]
fn issue_2019_partial_mitigation_keeps_catastrophic_overpagination_from_returning() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwpx_path = Path::new(repo_root).join("samples/hwpx/issue2019_floating_form_74312.hwpx");
    let bytes =
        fs::read(&hwpx_path).unwrap_or_else(|e| panic!("read {}: {}", hwpx_path.display(), e));

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse issue2019_floating_form_74312.hwpx");

    // 한글 2022 기준 페이지 수는 18쪽이다. 이 assert 는 페이지 수 폭증 재발만 막는다.
    // 서식 위치 정합은 pdf/issue2019 기준 PDF visual sweep 으로 별도 검증한다.
    let pages = doc.page_count();
    assert!(
        pages <= 20,
        "부동 폼 과분할 재발 — 페이지 수 {pages} (부분 완화 기대 ≤20, 한글 18, 수정 전 81)"
    );
}
