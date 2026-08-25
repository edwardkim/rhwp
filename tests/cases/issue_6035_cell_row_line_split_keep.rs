//! [Issue #6035] CELL 분할(인트라-로우 허용) 표에서 쪽 경계의 행을 셀 안에서 줄
//! 단위로 나누지 않고 통째로 밀어 5쪽 하단 31pt 를 비운다
//! (`samples/issue6035/2804253_cosmetics_gmp_checklist.hwpx` 147행 평가표,
//! 한글 2020 = 11쪽 vs rhwp = 12쪽).
//!
//! 기전: 잔여 41.3px 에 '다. 원자재…' 행의 첫 줄(20.8px)이 들어가고
//! `advance_row_cut` 도 그 1줄 컷을 고르지만, 25px 고아 가드
//! (`row_split_meets_min_top_keep`)가 큰 글줄(10pt+)의 정상 1줄 유지를 상시
//! 기각해 행 전체가 다음 쪽으로 갔다. 한글은 그 자리에서 첫 줄만 남긴다 —
//! 저장 사다리가 그 흔적을 담고 있다(셀 문단 lineseg 0/1560/1560, 비전진
//! 동일-vpos 연속쌍 = 저장 시점 쪽 경계 줄-단위 분할; horz 동일이라 좌우분할
//! 아님).
//!
//! 수정: HWPX 저장 프로파일에서 그 저장 흔적이 있는 행의 완결 유닛 ≥1 컷에는
//! 고아 가드를 적용하지 않는다 (`row_has_stored_same_vpos_split_signal`).
//! 예산 초과 컷의 재시도/이월 판정은 종전 그대로다.
//!
//! 결함 상태에서는 12쪽 + 5쪽 말미가 '하는가?'(직전 행 꼬리, baseline
//! 1039.1px)로 끝나 세 어서션이 실패한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6035/2804253_cosmetics_gmp_checklist.hwpx";

#[test]
fn issue_6035_cell_break_row_keeps_first_line_on_page_tail() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    assert_eq!(
        core.page_count(),
        11,
        "한글 2020 정본은 11쪽이다 (결함 시 12쪽)"
    );

    // 5쪽 마지막 글줄 = 분할 행의 첫 줄 '다. 원자재, … 구획된 장소'.
    // 결함 시 직전 행 꼬리 '하는가?' (baseline 1039.1px)가 마지막이다.
    let p5 = core.render_page_svg_native(4).expect("page 5 svg");
    let (p5_last_y, p5_last_text) = last_text_row(&p5);
    assert!(
        (1050.0..=1070.0).contains(&p5_last_y),
        "5쪽 마지막 글줄 baseline 이 분할 첫 줄 자리(1058.9px)여야 한다 (결함 시 1039.1): {p5_last_y:.1}"
    );
    assert!(
        p5_last_text.contains("원자재") && p5_last_text.contains("구획된"),
        "5쪽 마지막 글줄은 '다. 원자재 … 구획된 장소' 첫 줄이어야 한다: {p5_last_text:?}"
    );

    // 6쪽 첫 글줄 = 같은 행의 이어짐 '에서 보관하고 있는가? (다만…'.
    // 결함 시 행 전체가 내려와 '다. 원자재…' 로 시작한다.
    let p6 = core.render_page_svg_native(5).expect("page 6 svg");
    let (_, p6_first_text) = first_text_row(&p6);
    assert!(
        p6_first_text.contains("보관하고"),
        "6쪽 첫 글줄은 행 이어짐 '에서 보관하고 있는가?…' 여야 한다 (결함 시 '다. 원자재…'): {p6_first_text:?}"
    );
}

fn last_text_row(svg: &str) -> (f64, String) {
    text_row(svg, false)
}

fn first_text_row(svg: &str) -> (f64, String) {
    text_row(svg, true)
}

/// 같은 baseline(±1px) 글리프들을 한 줄로 묶어 첫/마지막 줄을 돌려준다.
fn text_row(svg: &str, first: bool) -> (f64, String) {
    let mut glyphs: Vec<(f64, String)> = Vec::new();
    for chunk in svg.split("<text").skip(1) {
        let Some(tag_end) = chunk.find('>') else {
            continue;
        };
        let Some(y) = attr(&chunk[..tag_end], "y") else {
            continue;
        };
        let Some(close) = chunk[tag_end + 1..].find("</text>") else {
            continue;
        };
        glyphs.push((y, chunk[tag_end + 1..tag_end + 1 + close].to_string()));
    }
    assert!(!glyphs.is_empty(), "svg 에 텍스트가 없다");
    let target =
        glyphs
            .iter()
            .map(|(y, _)| *y)
            .fold(if first { f64::MAX } else { f64::MIN }, |acc, y| {
                if first {
                    acc.min(y)
                } else {
                    acc.max(y)
                }
            });
    let mut row = String::new();
    for (y, text) in &glyphs {
        if (y - target).abs() <= 1.0 {
            row.push_str(text);
        }
    }
    (target, row)
}

fn attr(head: &str, name: &str) -> Option<f64> {
    let needle = format!("{name}=\"");
    let start = head.find(&needle)? + needle.len();
    let rest = &head[start..];
    let end = rest.find('"')?;
    rest[..end].parse().ok()
}
