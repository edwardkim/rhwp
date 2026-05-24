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

/// SVG 문자열에서 `<line>` 요소의 좌표와 점선 여부를 추출한다.
/// 반환: `(x1, y1, x2, y2, dashed)` — `dashed` 는 `stroke-dasharray` 보유 여부.
fn parse_lines(svg: &str) -> Vec<(f64, f64, f64, f64, bool)> {
    let mut out = Vec::new();
    for seg in svg.split("<line ").skip(1) {
        let head = &seg[..seg.find('>').unwrap_or(seg.len())];
        let get = |k: &str| -> Option<f64> {
            let p = head.find(&format!("{k}=\""))? + k.len() + 2;
            let rest = &head[p..];
            rest[..rest.find('"')?].parse().ok()
        };
        if let (Some(x1), Some(y1), Some(x2), Some(y2)) =
            (get("x1"), get("y1"), get("x2"), get("y2"))
        {
            let dashed = head.contains("stroke-dasharray");
            out.push((x1, y1, x2, y2, dashed));
        }
    }
    out
}

/// #1043 회귀 가드: 중첩 표(1×1 wrapper) 외곽 테두리 누락 정정 (HWP5 케이스).
///
/// `samples/k-water-rfp.hwp` 안에는 외곽 1×1 wrapper 표 안에 내부 표가 든 자료 박스
/// 구조가 있다. 내부 표의 외곽 격자는 점선(`stroke-dasharray`)으로, wrapper 외곽
/// 테두리는 그 위에 겹치는 실선으로 그려진다. off-by-one lookup 버그에서는 wrapper
/// 외곽 borderFill 을 한 칸 어긋나게 읽어(NONE) 실선 외곽선이 통째로 누락되고 내부 표
/// 점선만 남았다. 정정 후에는 점선 외곽과 같은 y 에 실선 외곽선이 존재해야 한다.
///
/// 가드: 전폭(>500px) 수평선 중 **점선과 y 가 일치하는 실선**이 ≥1 존재하는지 확인한다.
/// 좌표를 hardcode 하지 않고 "외곽 박스 = 내부 표 외곽" 관계로 판정하므로, 무관한
/// 다른 표의 실선(겹치는 점선 없음)이나 페이지네이션 시프트에 영향받지 않는다.
/// (버그: 일치 0건 → 실패 / 정정: 상·하 2건 일치 → 통과)
#[test]
fn nested_table_border_kwater_rfp_outer_outline_present() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join("samples/k-water-rfp.hwp");
    let bytes = fs::read(&path).expect("read k-water-rfp.hwp");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse k-water-rfp.hwp");

    let mut matched_pages = Vec::new();
    for page_idx in 0..doc.page_count() {
        let svg = doc
            .render_page_svg(page_idx)
            .unwrap_or_else(|e| panic!("render_page_svg page {}: {e:?}", page_idx + 1));
        let lines = parse_lines(&svg);
        // 전폭(>500px) 수평선만 추려 점선/실선 y 집합으로 분리한다.
        let is_wide_horiz =
            |x1: f64, y1: f64, x2: f64, y2: f64| (y1 - y2).abs() < 0.01 && (x2 - x1).abs() > 500.0;
        let dashed_ys: Vec<f64> = lines
            .iter()
            .filter(|(x1, y1, x2, y2, dashed)| *dashed && is_wide_horiz(*x1, *y1, *x2, *y2))
            .map(|(_, y1, ..)| *y1)
            .collect();
        // 점선(내부 표 외곽 격자)과 y 가 일치(±1px)하는 실선(wrapper 외곽 테두리) 개수.
        let outer_solid_on_inner = lines
            .iter()
            .filter(|(x1, y1, x2, y2, dashed)| !*dashed && is_wide_horiz(*x1, *y1, *x2, *y2))
            .filter(|(_, y1, ..)| dashed_ys.iter().any(|dy| (dy - *y1).abs() < 1.0))
            .count();
        if outer_solid_on_inner >= 1 {
            matched_pages.push((page_idx + 1, outer_solid_on_inner));
        }
    }

    assert!(
        !matched_pages.is_empty(),
        "wrapper 외곽 실선 테두리 누락 (내부 표 점선 외곽과 겹치는 전폭 실선 0건)"
    );
}
