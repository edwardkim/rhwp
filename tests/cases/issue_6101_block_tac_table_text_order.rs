//! [Issue #6101] 문단의 블록(비인라인) TAC 표와 본문 텍스트가 저장 lineseg
//! **한 줄**에 함께 담긴 경우, 합성이 줄을 분리하지 않아 ① 레이아웃 TAC 폴백이
//! line0 전체 폭(=동거 텍스트)을 leading 으로 오산해 표가 텍스트 폭만큼
//! 우측으로 밀려 쪽 밖으로 잘리고(36361137 7쪽 x 626 vs 한글 69.7, 36501883
//! 1쪽 x 495.6 vs 한글 76.8) ② 표→텍스트 순서 문단(36361137)에서는 조판이
//! 텍스트 줄을 계상·발행하지 않아 본문("ㅇ 직무요건 …")이 통째로 소실됐다.
//!
//! 수정: 합성(compose_paragraph)이 블록 TAC 표 줄과 텍스트 줄을 분리한다 —
//! 한글 2020 오라클은 두 문서 모두 표를 줄 머리에, 텍스트를 표 아래 줄에 둔다.
//!
//! 결함 상태에서는 표 좌단 x 어서션(두 문서)과 텍스트 존재 어서션(36361137)이
//! 실패한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const FIREFIGHTER: &str = "samples/issue6101/36361137_firefighter_training_plan.hwpx";
const APPROVAL: &str = "samples/issue6101/36501883_approval_doc_body.hwpx";

fn load(sample: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open")
}

#[test]
fn issue_6101_table_after_control_text_renders_and_table_stays_left() {
    let core = load(FIREFIGHTER);
    // 한글 2020 은 11쪽. rhwp 는 본 수정 전후 모두 12쪽(+1 은 별도 흐름 축) —
    // 회귀 감지용 핀.
    assert_eq!(
        core.page_count(),
        12,
        "쪽수 핀(한글 11, 잔여 +1 은 별도 축)"
    );

    let svg = core.render_page_svg_native(6).expect("page 7 svg");

    // 표(입학학년/연령요건, 폭 633px)는 좌측 여백(한글 x=69.7)에서 시작해야
    // 한다. 결함 시 x≈626 에서 시작해 본문 우단을 534px 초과.
    let table_left = wide_rule_min_x(&svg, 600.0, 400.0, 520.0).expect("표 전폭 괘선");
    assert!(
        table_left < 100.0,
        "블록 TAC 표는 좌측 여백에서 시작해야 한다 (한글 69.7, 결함 시 626): {table_left:.1}"
    );

    // 표 뒤 본문 "ㅇ 직무요건 …" 은 소실되지 않고 표 아래(y≈505)에 그려져야
    // 한다. SVG 는 글리프당 <text> 라 대역 수집로 확인한다.
    let below_table = text_glyphs_in_band(&svg, 495.0, 530.0);
    assert!(
        ['직', '무', '요', '건'].iter().all(|ch| below_table.contains(*ch)),
        "표 뒤 본문 텍스트가 소실되면 안 된다 (결함 시 render-tree·export-text 모두 부재): {below_table:?}"
    );
}

#[test]
fn issue_6101_text_before_control_table_stays_left() {
    let core = load(APPROVAL);
    assert_eq!(core.page_count(), 2, "한글 2020 정본은 2쪽이다");

    let svg = core.render_page_svg_native(0).expect("page 1 svg");

    // 텍스트→표 순서(별표 53자 + 표 11×9)도 같은 서명 — 표는 좌측 여백(한글
    // x=76.8)에서 시작해야 한다. 결함 시 x≈495.6 으로 본문 우단을 422px 초과.
    let table_left = wide_rule_min_x(&svg, 600.0, 430.0, 770.0).expect("표 전폭 괘선");
    assert!(
        table_left < 110.0,
        "블록 TAC 표는 좌측 여백에서 시작해야 한다 (한글 76.8, 결함 시 495.6): {table_left:.1}"
    );
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

/// y 대역 [y_min, y_max] 안의 전폭(>min_w) 가로 괘선들 중 최소 x1.
fn wide_rule_min_x(svg: &str, min_w: f64, y_min: f64, y_max: f64) -> Option<f64> {
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
        if (y1 - y2).abs() > 0.01 || (x2 - x1).abs() < min_w || y1 < y_min || y1 > y_max {
            continue;
        }
        let left = x1.min(x2);
        best = Some(best.map_or(left, |b: f64| b.min(left)));
    }
    best
}

fn attr(head: &str, name: &str) -> Option<f64> {
    let needle = format!("{name}=\"");
    let start = head.find(&needle)? + needle.len();
    let rest = &head[start..];
    let end = rest.find('"')?;
    rest[..end].parse().ok()
}
