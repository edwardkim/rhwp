//! Issue #546: exam_science.hwp 2페이지 페이지네이션 회귀 (PR #506 origin).
//!
//! 본질: PR #506 (`82e41ba`) 의 typeset.rs `wrap_around_pic_bottom_px` 보정이
//! 2단 레이아웃 + Square wrap 그림 케이스에서 후속 paragraph 들의 누적 height 를
//! 그림 하단으로 advance 하면서 페이지 분리 부작용 발생.
//!
//! 정정: `82e41ba` 의 typeset.rs +36 + layout.rs +58 (총 +94) revert.
//!
//! Task #460 보완5 의 본 의도 (HWP3 Square wrap 그림 아래 텍스트 y위치 정합) 는
//! 별도 task 에서 페이지네이션 안전한 방식으로 재시도 필요.

use std::fs;
use std::path::Path;

#[test]
fn issue_546_exam_science_p2_pagination_restored() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/exam_science.hwp");
    let bytes = fs::read(&hwp_path).expect("read exam_science.hwp");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse exam_science.hwp");

    // v0.7.9 정답지: 4 페이지
    // 회귀 시: 6 페이지 (p2 본문 누락으로 paragraph 들이 다른 페이지로 분산)
    assert_eq!(
        doc.page_count(),
        4,
        "exam_science.hwp 는 4 페이지여야 함 (PR #506 회귀 시 6)"
    );
}
