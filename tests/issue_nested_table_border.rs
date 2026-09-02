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
//!
//! [#6621] 상자 기하는 한/글 PDF 실측(4절→A3 높이 기준 균일 배율 0.9385, 왼쪽 위 기준)
//! 으로 못 박는다: 상자 실선 x 549.9~961.8 (= 상자 표 선언 폭 30894HU=411.9px),
//! y 325.1~695.5 (높이 370.4 = 안쪽 표 + 셀 안 여백 850HU×2 + 안쪽 표 om_bottom 283).
//! 종전 rhwp 는 안쪽 표 폭(390.65)으로 상자를 그려 오른쪽 선이 940.5 였다.

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
    // x 좌표: 549.88 (좌) ~ 961.8 (우) — body left margin + 상자 표 선언 폭 411.9px.
    // 한/글 PDF 실측 549.9 / 961.8. y 는 앞 내용의 페이지네이션에 따라 움직일 수 있어
    // 절대값 대신 상자 높이(아래)로 검증한다.
    let lx = "549.8800000000001";
    let rx = "961.8000000000002";

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

    // 좌측 세로선 + 상·하 수평선이 모두 lx 에서 시작하므로 lx 좌표는 3건 이상이다.
    let outline_pattern = format!("x1=\"{lx}\"");
    let outline_count = svg.matches(&outline_pattern).count();
    assert!(
        outline_count >= 3,
        "4번 박스 좌측·상단·하단 라인의 lx 좌표 ≥ 3건 필요 (실제: {outline_count})"
    );

    // 상자 높이: 좌측 세로선(x=lx) 중 가장 긴 것. 한/글 370.4 (325.1~695.5).
    let box_height = parse_lines(&svg)
        .into_iter()
        .filter(|(x1, _, x2, _, _)| (x1 - x2).abs() < 0.01 && (x1 - 549.88).abs() < 0.01)
        .map(|(_, y1, _, y2, _)| (y2 - y1).abs())
        .fold(0.0, f64::max);
    assert!(
        (box_height - 370.4).abs() < 1.0,
        "4번 박스 높이 370.4 (안쪽 표 + 셀 여백 850HU×2 + 안쪽 표 om_bottom 283): {box_height:.1}"
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
/// 가드: 전폭(>500px) 수평선 중 **점선 바깥쪽에 여백만큼 떨어진 실선**이 위·아래로 존재하는지
/// 확인한다. 좌표를 hardcode 하지 않고 "외곽 박스 = 내부 표 외곽 + 여백" 관계로 판정하므로,
/// 무관한 다른 표의 실선이나 페이지네이션 시프트에 영향받지 않는다.
///
/// [#6621] 여백은 저장값 그대로다: wrapper 셀 안 여백 위/아래 141HU + 안쪽 표 바깥 여백
/// 위/아래 141HU = 3.8px. 한/글 2022 PDF 17쪽 실측: 실선 583.2/1001.4, 점선 587.0/997.5.
/// 종전 rhwp 는 두 여백을 모두 버려 실선과 점선이 같은 y 였고(583.3/993.9), 상자가 7.5px
/// 짧았다. (버그: 실선 누락 또는 점선과 같은 y → 실패 / 정정: 위·아래 3.8px 바깥 → 통과)
#[test]
fn nested_table_border_kwater_rfp_outer_outline_present() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join("samples/k-water-rfp.hwp");
    let bytes = fs::read(&path).expect("read k-water-rfp.hwp");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse k-water-rfp.hwp");

    // 셀 안 여백 141HU + 안쪽 표 바깥 여백 141HU (96DPI px).
    let inset = 2.0 * 141.0 * 96.0 / 7200.0;
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
        let (Some(dashed_top), Some(dashed_bottom)) = (
            dashed_ys.iter().cloned().reduce(f64::min),
            dashed_ys.iter().cloned().reduce(f64::max),
        ) else {
            continue;
        };
        let solid_ys: Vec<f64> = lines
            .iter()
            .filter(|(x1, y1, x2, y2, dashed)| !*dashed && is_wide_horiz(*x1, *y1, *x2, *y2))
            .map(|(_, y1, ..)| *y1)
            .collect();
        // 점선 맨 위에서 여백만큼 위, 맨 아래에서 여백만큼 아래에 실선(wrapper 외곽)이 있다.
        let has_top = solid_ys
            .iter()
            .any(|sy| (dashed_top - sy - inset).abs() < 0.5);
        let has_bottom = solid_ys
            .iter()
            .any(|sy| (sy - dashed_bottom - inset).abs() < 0.5);
        if has_top && has_bottom {
            matched_pages.push(page_idx + 1);
        }
    }

    assert!(
        !matched_pages.is_empty(),
        "wrapper 외곽 실선 테두리 누락 (내부 표 점선 위·아래 {inset:.1}px 바깥의 전폭 실선 0쪽)"
    );
}
