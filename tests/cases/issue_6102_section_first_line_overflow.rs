//! [Issue #6102] 구역 첫 문단(구역정의·단정의·자리차지 표 동거)의 본문 첫 줄이
//! 한글 2020 보다 6자 늦게 끊겨 우단(737px)을 71~99px 넘던 결함 — 결재문서본문
//! 3건 (36360328·36310257·36444579).
//!
//! 근인 3중:
//! 1. 자리차지(비TAC TopAndBottom) 표 host 문단을 본문 프레임이 사양해 저장
//!    textpos 의 검증(admission)이 통째로 빠졌다 — 표는 줄 폭을 소비하지
//!    않으므로 폭-중립으로 통과시킨다.
//! 2. 비말미 저장 줄이 자기 폭을 넘는 텍스트를 담았다고 주장하면(이 계보의
//!    textpos 축 어긋남) 물리적으로 성립하지 않는데 stale 판정이 1.8× 문턱만
//!    보고 지나쳤다 — 비말미 줄 한정 6%+12px 초과를 stale 로 본다.
//! 3. 프레임 재래핑 산출 행은 HWP5 축인데 원본의 [#5961] `hwpx_axis_shift` 를
//!    물려받아 이중 보정됐다(fill 이 char 51 에 끊어도 +8 재보정으로 59) —
//!    재래핑 복제본은 보정폭을 0 으로 되돌린다.
//!
//! 수정 후 세 문서 모두 한글 2020 오라클과 줄 끝·분할 지점 일치
//! (737.3/744.6/737.3 vs 736.6/744.6/741.9), 쪽수 1·1·2 일치.
//! 결함 상태에서는 첫 줄 글리프가 x 780~820 까지 그려져 어서션이 실패한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

fn load(sample: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open")
}

/// 첫 쪽 본문 대역의 <text> 글리프 중 최대 x (글리프 시작 좌표).
fn max_glyph_x_in_band(svg: &str, y_min: f64, y_max: f64) -> f64 {
    let mut max_x: f64 = 0.0;
    for chunk in svg.split("<text").skip(1) {
        let Some(tag_end) = chunk.find('>') else {
            continue;
        };
        let head = &chunk[..tag_end];
        let (Some(x), Some(y)) = (attr(head, "x"), attr(head, "y")) else {
            continue;
        };
        if y < y_min || y > y_max {
            continue;
        }
        max_x = max_x.max(x);
    }
    max_x
}

fn assert_first_body_line_inside_margin(sample: &str, expected_pages: u32) {
    let core = load(sample);
    assert_eq!(
        core.page_count(),
        expected_pages,
        "{sample}: 한글 2020 쪽수와 같아야 한다"
    );
    let svg = core.render_page_svg_native(0).expect("page 1 svg");
    // 본문 첫 줄 대역(y 310~340). 우단 737px — 결함 시 마지막 글리프 시작 x 가
    // 780~820 (줄 끝 807~836). 글리프 폭 여유를 두고 760 을 상한으로 건다.
    let max_x = max_glyph_x_in_band(&svg, 310.0, 340.0);
    assert!(
        max_x > 0.0,
        "{sample}: 본문 첫 줄 대역에 글리프가 있어야 한다"
    );
    assert!(
        max_x < 760.0,
        "{sample}: 본문 첫 줄이 우단(737px) 안에서 끊겨야 한다 (결함 시 마지막 글리프 x 780~820): {max_x:.1}"
    );
}

#[test]
fn issue_6102_first_line_breaks_inside_margin_36360328() {
    assert_first_body_line_inside_margin(
        "samples/issue6102/36360328_vehicle_inspection_expense.hwpx",
        1,
    );
}

#[test]
fn issue_6102_first_line_breaks_inside_margin_36310257() {
    assert_first_body_line_inside_margin("samples/issue6102/36310257_overtime_report.hwpx", 1);
}

#[test]
fn issue_6102_first_line_breaks_inside_margin_36444579() {
    assert_first_body_line_inside_margin(
        "samples/issue6102/36444579_traffic_fine_exemption.hwpx",
        2,
    );
}

fn attr(head: &str, name: &str) -> Option<f64> {
    let needle = format!("{name}=\"");
    let start = head.find(&needle)? + needle.len();
    let rest = &head[start..];
    let end = rest.find('"')?;
    rest[..end].parse().ok()
}
