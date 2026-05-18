//! Issue #898: exam_math.hwp 가운데 세로선 (직선 Shape) 끝과 바탕쪽 쪽번호 박스가
//! 거의 붙어 PDF (한컴 2022) 와 시각 차이.
//!
//! 원인: `compute_table_y_position` 의 Paper-relative 분기에서
//! `outer_margin_top` 이 산식에 누락. v_offset(101954 HU=359.5mm) + outer_margin_top
//! (1417 HU=5.0mm) = 364.5mm 가 가시 표 상단인데, 1417 HU 빠뜨려 5mm 위쪽 배치.
//!
//! 이 테스트는 페이지 1 바탕쪽 쪽번호 표 셀 상단 y 좌표가 outer_margin_top 반영
//! 후 위치 (≈1378 px) 임을 검증한다.

use std::fs;
use std::path::Path;

#[test]
fn master_page_table_includes_outer_margin_top() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/exam_math.hwp");
    let bytes =
        fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse exam_math.hwp");

    let svg = doc
        .render_page_svg_native(0)
        .expect("render exam_math.hwp page 1");

    // 바탕쪽 쪽번호 표 셀의 clipPath y 좌표 — outer_margin_top 적용 시 1378.28 px,
    // 미적용 회귀 시 1359.39 px.
    //
    // SVG 부동소수 표현 차이 흡수: y="1378" 이 포함되는지 확인.
    // 회귀(미적용) 값은 1359.x 이므로 "y=\"1378" 패턴 매칭 시 충분.
    let has_correct_y = svg.contains("y=\"1378.") || svg.contains("y=\"1378\"");
    assert!(
        has_correct_y,
        "바탕쪽 표 셀 y 좌표가 outer_margin_top 미반영 (회귀). SVG snippet:\n{}",
        svg.lines()
            .filter(|l| l.contains("cell-clip") && l.contains("1359"))
            .take(3)
            .collect::<Vec<_>>()
            .join("\n")
    );

    // 회귀 가드: 1359.x 좌표가 셀 클립에 나타나지 않아야 함
    let regression = svg
        .lines()
        .any(|l| l.contains("clipPath") && l.contains("cell-clip") && l.contains("y=\"1359."));
    assert!(
        !regression,
        "바탕쪽 표 셀이 outer_margin_top 미적용 y=1359 위치로 회귀."
    );
}
