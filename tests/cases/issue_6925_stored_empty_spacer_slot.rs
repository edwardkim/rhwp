//! [#6925] 감싼 1칸 칸의 빈 문단을 0 높이로 접어 문단 간격이 저장·정본보다 짧아진다.
//!
//! ## 무엇이 문제였나
//!
//! `148751598` 브리핑 1쪽은 본문 전체가 1행×1열 표 안에 있고, 내용 문단 사이에 빈 문단이
//! 하나씩 들어 있다. 저장 사다리는 그 빈 문단에 자리를 준다.
//!
//! ```text
//!   p[1] 1. 금주…  vpos=  884           p[2] (빈) vpos= 3120 lh=1000 ls=492  → 슬롯 1492
//!   p[3] ◈ 금주…   vpos= 4612           p[4] (빈) vpos= 6848 lh= 600 ls=296  → 슬롯  896
//!   p[5] ㅇ 주요…   vpos= 7744           p[8] (빈) vpos=20412 lh= 800 ls=392  → 슬롯 1192
//!   p[9] 중첩 표    vpos=21604
//! ```
//!
//! 각 빈 문단의 **다음 문단 vpos − 자기 vpos** 가 `lh + ls` 와 정확히 같다 — 사다리가 그
//! 자리를 비워 둔 것이다. 그런데 `legacy_single_cell_empty_spacer` 계약이 이 빈 문단들을
//! 0 높이로 접어, 문단마다 12~19.9px 씩 잃고 1쪽 안에서 **67.3px** 이 누적됐다.
//!
//! 보존 판정(`preserve_forward_stored_empty_spacer`)에는 이미 "다음 줄 상자의 3/4 이상"
//! (`full_line_box`) 규칙이 있었지만, 이 문서의 1000/1500(67%)·600/1500(40%)은 거기서
//! 떨어졌다. 또 표를 host 하는 다음 문단은 `controls.is_empty()` 조건에 막혔다.
//!
//! ## 수정과 기대값
//!
//! HWP5 저장 조판에서는 **저장 슬롯이 정확한**(±2HU) 빈 문단을 보존한다. 기대값은 두 곳에서
//! 독립적으로 나온다.
//!
//! 1. **저장 사다리**: 렌더한 문단 간격이 사다리 간격과 같아야 한다(p1→p3 49.7px,
//!    p3→p5 41.8px).
//! 2. **한/글 2020 정본** `pdf/148751598-briefing-2020.pdf`(저장소 기존 정답지): 머리 표
//!    행 149.7px 에서 본문 표 첫 행 596.9px 까지가 **447.2px** 이다. 수정 전 rhwp 는
//!    388.2px(−59.0), 수정 후 451.5px(+4.3)로 정본에 붙는다.
//!
//! ## 이 시험이 잠그지 않는 것
//!
//! 머리 표 자체의 −3.0px 차이와 본문 잔차 3~6px 는 다른 축이다(#6924 계열). 여기서는
//! **문단 간격의 누적 손실**만 본다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6924/148751598-briefing.hwp";

fn core() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(path).expect("표본")).expect("문서 로드")
}

/// 1쪽의 (줄 상단, 텍스트) 목록.
fn page1_lines() -> Vec<(f64, String)> {
    let core = core();
    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    fn walk(node: &RenderNode, out: &mut Vec<(f64, String)>) {
        if matches!(node.node_type, RenderNodeType::TextLine(_)) {
            let mut text = String::new();
            fn runs(node: &RenderNode, text: &mut String) {
                if let RenderNodeType::TextRun(run) = &node.node_type {
                    text.push_str(run.display_or_text());
                }
                for child in &node.children {
                    runs(child, text);
                }
            }
            runs(node, &mut text);
            out.push((node.bbox.y, text));
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    let mut out = Vec::new();
    walk(&tree.root, &mut out);
    out.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    out
}

fn line_top(lines: &[(f64, String)], needle: &str) -> f64 {
    lines
        .iter()
        .find(|(_, text)| text.contains(needle))
        .unwrap_or_else(|| panic!("1쪽에서 {needle:?} 줄을 찾지 못했다"))
        .0
}

/// 내용 문단 사이 간격이 저장 사다리와 같다 — 빈 문단의 높이를 잃지 않는다.
#[test]
fn paragraph_gaps_follow_the_stored_ladder() {
    let lines = page1_lines();
    let first = line_top(&lines, "1. 금주 주요 보도계획");
    let second = line_top(&lines, "금주 보도 내용은 총 16건임");
    let third = line_top(&lines, "주요 내용은 2013년 온실가스");

    // 저장 사다리: p[1] 884 → p[3] 4612 → p[5] 7744 (HWPUNIT, /75 = px)
    let ladder_first = (4612.0 - 884.0) / 75.0; // 49.71px
    let ladder_second = (7744.0 - 4612.0) / 75.0; // 41.76px
    assert!(
        (second - first - ladder_first).abs() <= 0.5,
        "p1→p3 간격이 저장 사다리와 다르다: 렌더 {:.1}px, 사다리 {ladder_first:.1}px \
         (수정 전 29.8px — 빈 문단 p[2] 19.9px 소실)",
        second - first
    );
    assert!(
        (third - second - ladder_second).abs() <= 0.5,
        "p3→p5 간격이 저장 사다리와 다르다: 렌더 {:.1}px, 사다리 {ladder_second:.1}px \
         (수정 전 29.8px — 빈 문단 p[4] 11.9px 소실)",
        third - second
    );
}

/// 머리 표에서 본문 표 첫 행까지의 거리가 한/글 정본과 맞는다.
#[test]
fn distance_to_the_body_table_matches_the_hancom_oracle() {
    let lines = page1_lines();
    let head = line_top(&lines, "대변인 정례 브리핑");
    // 본문 표 첫 행의 `브리핑` 칸 — 머리 표 제목과 구별하려면 정확히 그 글자뿐인 줄이다.
    let table_row = lines
        .iter()
        .find(|(_, text)| text.trim() == "브리핑")
        .expect("본문 표 첫 행의 `브리핑` 칸")
        .0;
    let span = table_row - head;

    // 정본 `pdf/148751598-briefing-2020.pdf` 1쪽: 149.7px → 596.9px = 447.2px.
    const ORACLE_SPAN_PX: f64 = 447.2;
    assert!(
        (span - ORACLE_SPAN_PX).abs() <= 10.0,
        "머리 표→본문 표 거리가 정본과 다르다: rhwp {span:.1}px, 정본 {ORACLE_SPAN_PX:.1}px \
         (수정 전 388.2px — 문단 간격 누적 손실 −59.0px)"
    );
}

/// 쪽수는 정본과 같은 6쪽이다.
#[test]
fn page_count_matches_the_oracle() {
    assert_eq!(core().page_count(), 6, "한/글 2020 정본은 6쪽이다");
}
