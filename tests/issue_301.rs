//! Issue #301: exam_math.hwp 12쪽 좌측 #29 정규분포 z-table의 셀 수식이
//! SVG에 두 번 그려지는 회귀 버그.
//!
//! 셀 구조: text="" + Equation 컨트롤 1개. paragraph_layout(Task #287의
//! 빈-runs 경로)과 table_layout(`has_text_in_para=false` 분기)이 같은
//! 수식을 각각 emit하여 중복.
//!
//! 이 테스트는 z-table 셀 값(`0.1915`, `0.3413`, `0.4332`, `0.4772`)이
//! SVG에 각 1회만 출현함을 검증한다.

use std::fs;
use std::path::Path;

#[test]
fn z_table_equations_rendered_once() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/exam_math.hwp");
    let bytes =
        fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse exam_math.hwp");

    // 페이지 12 = index 11
    let svg = doc
        .render_page_svg_native(11)
        .expect("render exam_math.hwp page 12");

    for value in ["0.1915", "0.3413", "0.4332"] {
        let count = svg.matches(value).count();
        assert_eq!(
            count, 1,
            "z-table value {value:?} expected 1 occurrence, found {count} (이중 렌더링 회귀)"
        );
    }

    // 0.4772는 #30 문제의 답 등 다른 곳에도 등장하므로 별도 임계
    // (수정 전: 3회, 수정 후: 2회 기대)
    let v4772 = svg.matches("0.4772").count();
    assert_eq!(
        v4772, 2,
        "0.4772 expected 2 occurrences (z-table 1 + 다른 위치 1), found {v4772}"
    );
}
