//! [Issue #5723] 나란한 두 어울림 표 중 왼쪽만 60.8px 아래로 밀린다 (156630807 14쪽).
//!
//! 근인: 외곽 2×2 TAC 표의 오른쪽 셀에서, wrap 에 **밀려난** 호스트 줄의 저장
//! vpos(155.4px) 위에 SQUARE 중첩 표 높이(147.8px)를 다시 얹어 셀 내용을
//! 303.2px(한글 184px)로 과대측정 — 행이 307px 로 부풀고, 정확히 측정된 왼쪽
//! 셀(185.5px)만 vertAlign=CENTER 슬랙의 절반(60.8px)만큼 내려갔다.
//!
//! 수정: 호스트 줄 저장 vpos 가 표 높이 이상이면(표 공간이 줄 위에 이미 예약된
//! displaced-line 증거, #2226 의 표 판) PARA 기준 float 표의 para_top 가산을
//! 접는다(`cell_nested_controls_bottom`). 한글 오라클(COM PDF): 두 표 모두 상단
//! 정렬('순위' y=400.6 ↔ 수정 후 399.3).
//!
//! 픽스처는 원본 HWPX 구역1 문단 325..332(14쪽 차트 표 구간) 절단 + 스텁(48KB).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5723/coanchored_square_pair_center_slack.hwpx";

#[test]
fn issue_5723_square_pair_stays_level() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(0).expect("page 1 svg");

    // 두 차트 표의 위 괘선: 폭 265.2 / 321.8 의 수평선 y 를 찾는다.
    let top_of = |w: &str| -> f64 {
        let mut min_y = f64::INFINITY;
        for cap in svg.split("<line ").skip(1) {
            let head = &cap[..cap.find('>').unwrap_or(cap.len())];
            let (Some(x1), Some(x2), Some(y1)) = (
                attr_f64(head, "x1=\""),
                attr_f64(head, "x2=\""),
                attr_f64(head, "y1=\""),
            ) else {
                continue;
            };
            let width = (x2 - x1).abs();
            let want: f64 = w.parse().unwrap();
            if (width - want).abs() < 1.5 {
                min_y = min_y.min(y1);
            }
        }
        min_y
    };
    let left_top = top_of("265.2");
    let right_top = top_of("321.8");
    assert!(
        left_top.is_finite() && right_top.is_finite(),
        "두 차트 표의 괘선을 찾아야 한다: left={left_top}, right={right_top}"
    );
    // 결함 시 왼쪽이 60.8px 아래(453.6 vs 392.8)로 밀린다.
    assert!(
        (left_top - right_top).abs() <= 2.0,
        "나란한 두 어울림 표는 같은 높이에서 시작해야 한다 (결함 시 +60.8): \
         left={left_top:.1}, right={right_top:.1}"
    );
}

fn attr_f64(head: &str, key: &str) -> Option<f64> {
    let rest = head.split_once(key)?.1;
    rest[..rest.find('"')?].parse().ok()
}
