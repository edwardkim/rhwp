//! Nested table 외부 1x1 wrapper 표 외곽 테두리 누락 정정 (exam_social.hwp p1 4번).
//!
//! `src/renderer/layout/table_layout.rs::layout_table` 의 1x1 wrapper 분기는
//! 외부 표를 무시하고 내부 표만 직접 layout 한다. 외부 표가 padding 과
//! border line 을 가진 자료 박스 외곽 테두리 역할인 경우 외곽선이 누락되었다.
//!
//! 정정: wrapper 분기 진입 시 외부 셀의 padding != 0 + border_fill 의 borders
//! 중 하나라도 None 아닌 경우, 외부 표의 size + border_fill 정보로 외곽 4개
//! 라인을 col_node 에 추가한다.
//!
//! 권위 자료: pi=15 4번 자료 박스 (외부 1x1 padding=850 + 내부 6x3 대화체).
//! 한컴2022 PDF (`pdf/exam_social-2022.pdf`) p1 우측 4번 영역 외곽 박스 시각 정합.

use std::fs;
use std::path::Path;

#[test]
fn nested_table_border_exam_social_p1_q4_outline_present() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/exam_social.hwp");
    let bytes = fs::read(&hwp_path).expect("read exam_social.hwp");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse exam_social.hwp");

    // 4 페이지 (PDF 정합)
    assert_eq!(doc.page_count(), 4, "exam_social.hwp 는 4 페이지");

    // 페이지 1 SVG 출력
    let svg = doc.render_page_svg(0).expect("render_page_svg");

    // 4번 자료 박스 외곽 4개 라인이 SVG 에 존재해야 한다.
    // 박스 size: nested 6x3 표 측정 결과 — width=390.65 (nested.common.width),
    // height=343.88 (nested layout 의 y_end - y_start).
    // 위치: x=549.88~940.53, y=331.53~675.41
    // (외부 표 IR size 411.92×370.32 사용 시 5번 박스와 위치 부정합 발생.)
    let has_top_line = svg.contains("y1=\"331.53333333333336\"")
        && svg.contains("x1=\"549.8800000000001\"")
        && svg.contains("x2=\"940.5333333333334\"");
    let has_bottom_line = svg.contains("y1=\"675.4133333333334\"")
        && svg.contains("y2=\"675.4133333333334\"");
    let has_left_line = svg.contains("x1=\"549.8800000000001\"")
        && svg.contains("x2=\"549.8800000000001\"")
        && svg.contains("y2=\"675.4133333333334\"");
    let has_right_line = svg.contains("x1=\"940.5333333333334\"")
        && svg.contains("x2=\"940.5333333333334\"");

    assert!(has_top_line, "4번 박스 상단 외곽선 누락");
    assert!(has_bottom_line, "4번 박스 하단 외곽선 누락");
    assert!(has_left_line, "4번 박스 좌측 외곽선 누락");
    assert!(has_right_line, "4번 박스 우측 외곽선 누락");
}
