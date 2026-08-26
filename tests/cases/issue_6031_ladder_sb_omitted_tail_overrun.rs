//! [Issue #6031] 저장 사다리가 문단 위 간격(sb)을 안 담은 HWPX 문서에서 쪽-말미
//! 줄이 본문 하단 여백을 뚫고 그려진다
//! (`samples/issue6031/3249937_asset_management_rules.hwpx` 3쪽 +25.1pt ·
//! 6쪽 +12.1/+40.1pt, 60쪽 저장 linesegarray 문서).
//!
//! 기전(#5801 코어의 잔존 변형): typeset 은 vpos 스냅·트림으로 커서를 sb-누락
//! 사다리에 붙이는데(쪽 채움 회계 = 사다리), layout 페인트는 sb 를 가산한 흐름
//! (후방 스냅 상한 8px)이라 쪽 말미에서 +53.3px 표류한다. 이 착시 위에서
//! `saved_tail_vpos_fit`·일반 fit 이 꼬리 줄을 현재 쪽에 붙들고, 그 줄은 본문
//! 하단 밖에 그려진다. 한글 2020 fresh 흐름은 rhwp layout 흐름과 정합(p3 말미
//! 810.7px vs 실측 809.7pt)하고 꼬리 줄을 다음 쪽 첫 줄로 넘긴다.
//!
//! 수정: ① sb-누락 경계 확정 시(`stored_ladder_encodes_spacing_before` false)
//! 남은 열의 후방 스냅·트림을 dirty 로 철회해 판정·배치 좌표계를 일치시키고
//! ② dirty 에서 `saved_tail_vpos_fit` 를 차단, ③ 한줄문단 연속 구간(intra 표본
//! 부재)의 게이트 사각을 직전 줄 trailing ls 등가-일치 폴백으로 메운다.
//!
//! 결함 상태에서는 3쪽 마지막 baseline 1115.5px·6쪽 1132.8px 로 두 상한이
//! 실패한다. 총 쪽수는 수정 전후 한글과 같은 60 이다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6031/3249937_asset_management_rules.hwpx";

/// 본문 하단 = 위 20mm + 본문 1009.15px → 1084.7px. baseline 은 그 안이어야 한다.
const BODY_BOTTOM_PX: f64 = 1085.2;

#[test]
fn issue_6031_page_tail_lines_stay_inside_body_bottom() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    assert_eq!(core.page_count(), 60, "한글 2020 정본은 60쪽이다");

    // 3쪽 — '바. 매립지등 …' 줄이 본문 밖(+25.1pt, baseline 1115.5px)에 남으면 결함.
    let p3 = core.render_page_svg_native(2).expect("page 3 svg");
    let p3_last = max_text_baseline(&p3);
    assert!(
        p3_last <= BODY_BOTTOM_PX,
        "3쪽 마지막 글줄 baseline 이 본문 하단(1084.7px) 안이어야 한다 (결함 시 1115.5): {p3_last:.1}"
    );

    // 6쪽 — '1)'·'2)' 줄이 본문 밖(+12.1/+40.1pt, baseline 1132.8px)에 남으면 결함.
    let p6 = core.render_page_svg_native(5).expect("page 6 svg");
    let p6_last = max_text_baseline(&p6);
    assert!(
        p6_last <= BODY_BOTTOM_PX,
        "6쪽 마지막 글줄 baseline 이 본문 하단(1084.7px) 안이어야 한다 (결함 시 1132.8): {p6_last:.1}"
    );

    // 넘긴 줄이 4쪽 첫 줄이 됐는지 — 한글 정본: '바. 매립지등 관리자는 …' 이
    // 4쪽 y0=56.4pt(첫 글줄)다. 첫 글줄 밴드(y<100px)의 글리프에 '바'/'매'/'립'.
    let p4 = core.render_page_svg_native(3).expect("page 4 svg");
    let first_band = text_glyphs_in_band(&p4, 0.0, 100.0);
    for needle in ['바', '매', '립'] {
        assert!(
            first_band.contains(needle),
            "4쪽 첫 글줄에 '바. 매립지등…' 글리프({needle})가 있어야 한다: {first_band:?}"
        );
    }
}

/// SVG `<text>` 의 최대 baseline y.
fn max_text_baseline(svg: &str) -> f64 {
    let mut max_y: f64 = 0.0;
    for chunk in svg.split("<text").skip(1) {
        let Some(end) = chunk.find('>') else {
            continue;
        };
        if let Some(y) = attr(&chunk[..end], "y") {
            max_y = max_y.max(y);
        }
    }
    max_y
}

/// y 밴드 안의 `<text>` 글리프들을 이어붙인다.
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
