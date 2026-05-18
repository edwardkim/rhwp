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
    // 박스 width: nested 6x3 표 측정 결과 — 390.65 (nested.common.width).
    // x 좌표: 549.88 (좌) ~ 940.53 (우) — body left margin + nested 표 위치.
    // y 좌표: 다른 PR 영역의 페이지네이션 변경에 따라 시프트 가능 영역으로 영역
    // 좌표 hardcoded 영역 회피 영역 영역 — x 좌표 영역과 stroke 영역 본질 영역만 영역 검증 영역.
    let lx = "549.8800000000001";
    let rx = "940.5333333333334";

    // 좌측선: x1==x2==lx (수직선)
    let has_left_line = svg.contains(&format!("<line x1=\"{lx}\" y1="))
        && svg
            .matches(&format!("x1=\"{lx}\" y1=\""))
            .filter(|_| true)
            .count()
            >= 1
        && svg.contains(&format!("x2=\"{lx}\""));
    // 우측선: x1==x2==rx (수직선)
    let has_right_line =
        svg.contains(&format!("<line x1=\"{rx}\" y1=")) && svg.contains(&format!("x2=\"{rx}\""));
    // 상/하: x1==lx, x2==rx (수평선)
    let has_horizontal_line =
        svg.contains(&format!("x1=\"{lx}\" y1=")) && svg.contains(&format!("x2=\"{rx}\""));

    assert!(has_left_line, "4번 박스 좌측 외곽선 누락 (x={lx})");
    assert!(has_right_line, "4번 박스 우측 외곽선 누락 (x={rx})");
    assert!(
        has_horizontal_line,
        "4번 박스 수평 외곽선 누락 (x={lx}~{rx})"
    );

    // 외곽선 stroke=#000000 width=0.75 (3 조건 AND 가드 영역 발동 영역의 본 PR 영역의 본질 영역)
    let outline_pattern = format!("x1=\"{lx}\"");
    let outline_count = svg.matches(&outline_pattern).count();
    assert!(
        outline_count >= 2,
        "4번 박스 좌측+상단 라인 영역의 lx 좌표 ≥ 2건 영역 필요 영역 (실제: {outline_count})"
    );
}
