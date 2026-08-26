//! [Issue #6095] 바깥 1×1 셀 안 중첩 표의 host 문단 저장 vpos 가 표 **아래**
//! host 줄 좌표인데(3090867 앵커 vpos=33720 = 직전 흐름 + 중첩 표 높이), 이를
//! 표 상단으로 오독해 ① 페인트가 표를 그 좌표로 끌어내려 표 위에 +303px
//! 빈공간, ② 유닛 회계가 gap(328px)+중첩 행(302px)으로 표 높이를 이중 계상해
//! 조각 컷이 일러진다(used 953 vs 페인트 744) —
//! 본문(1)~3)·예시·주석)이 2쪽으로 밀렸다
//! (`samples/issue6095/3090867_icepack_levy_criteria.hwpx`, 한글 2020 2쪽).
//!
//! 수정: 점프가 중첩 표 선언 높이 규모면 post-table host 줄로 판별해
//! ① table_partial 의 전방 스냅 억제(표는 자연 흐름에 페인트),
//! ② 유닛 gap 에서 중첩 표 높이 차감. 결과: 갭 333→22.5px(한글 ≈30),
//! p1 에 1)~3)·직선보간법 예시·계산식·주석까지 수용, p2 = 예1)/예2)
//! (한글 기대 구성과 제목행 1행 차이 이내).
//!
//! 결함 상태에서는 중첩 표 상단 555px·p2 상단이 '적용'(2)의 꼬리)로 두
//! 어서션이 실패한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6095/3090867_icepack_levy_criteria.hwpx";

#[test]
fn issue_6095_nested_host_anchor_below_table_not_double_charged() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    assert_eq!(core.page_count(), 2, "한글 2020 정본은 2쪽이다");

    // 1쪽 — '나.' 둘째 줄(하단 222px) 바로 아래에 중첩 표(캡션 행)가 와야 한다.
    // 결함 시 표 상단이 555px(+303px 빈공간).
    let p1 = core.render_page_svg_native(0).expect("page 1 svg");
    let top = first_wide_rule_below(&p1, 225.0).expect("중첩 표 상단 괘선");
    assert!(
        (230.0..=290.0).contains(&top),
        "중첩 표 상단이 '나.' 둘째 줄 직하(244px 근방)여야 한다 (결함 시 555px): {top:.1}"
    );

    // 2쪽 — 예1) 블록에서 시작해야 한다 (결함 시 2)의 꼬리 '적용'부터).
    let p2 = core.render_page_svg_native(1).expect("page 2 svg");
    let head = text_glyphs_in_band(&p2, 0.0, 160.0);
    assert!(
        head.contains('예') && head.contains('1'),
        "2쪽 상단은 예1) 블록이어야 한다 (결함 시 2)의 꼬리): {head:?}"
    );
}

/// y_min 아래의 첫 전폭(>400px) 가로 괘선 y.
fn first_wide_rule_below(svg: &str, y_min: f64) -> Option<f64> {
    let mut best: Option<f64> = None;
    for chunk in svg.split("<line ").skip(1) {
        let Some(end) = chunk.find('>') else {
            continue;
        };
        let head = &chunk[..end];
        let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
            attr(head, "x1"),
            attr(head, "y1"),
            attr(head, "x2"),
            attr(head, "y2"),
        ) else {
            continue;
        };
        if (y1 - y2).abs() > 0.01 || (x2 - x1).abs() < 400.0 || y1 < y_min {
            continue;
        }
        best = Some(best.map_or(y1, |b: f64| b.min(y1)));
    }
    best
}

fn text_glyphs_in_band(svg: &str, y_min: f64, y_max: f64) -> String {
    let mut out = String::new();
    for chunk in svg.split("<text").skip(1) {
        let Some(tag_end) = chunk.find('>') else {
            continue;
        };
        let Some(y) = attr(&chunk[..tag_end], "y") else {
            continue;
        };
        if y < y_min || y > y_max {
            continue;
        }
        if let Some(close) = chunk[tag_end + 1..].find("</text>") {
            out.push_str(&chunk[tag_end + 1..tag_end + 1 + close]);
        }
    }
    out
}

fn attr(head: &str, name: &str) -> Option<f64> {
    let needle = format!("{name}=\"");
    let start = head.find(&needle)? + needle.len();
    let rest = &head[start..];
    let end = rest.find('"')?;
    rest[..end].parse().ok()
}
