//! 저장 줄 없는 문단에서 탭의 다음 명시적 왼쪽 정지가 줄 오른쪽 끝 밖이면, 한컴은 탭 앞의 글과
//! 탭 뒤의 글을 같은 줄에 두지 않는다.
//!
//! 재현물 `samples/tab_overflow/tab_stop_beyond_line.hwpx` (`samples/test-image.hwpx` 조합, 저장 줄 없음):
//! 왼쪽 여백 1140 HU, 오른쪽 여백으로 줄 폭 5500 HU, 탭 정지 8840·16540 HU(모두 줄 폭 밖), 11pt.
//! 대상 문단 셋 — `TABONLY`(탭 하나), `GA_TAB_X`(`가`+탭+`X`), `NOTAB`(`가`) — 앞뒤에 라벨 줄이 있다.
//!
//! 기준: `pdf/tab_overflow/tab_stop_beyond_line-webhwp-print.pdf` (웹한글기안기 인쇄, 줄 피치 11.04pt):
//! - `가`+탭+`X`: `가` / 탭 / `X` 세 줄. `X` 는 `가` 보다 두 줄 아래(22.08pt = 29.4px), 줄 처음(`가` 와 같은 x)
//! - 탭만 있는 문단과 탭 없는 문단은 한 줄
//!
//! 수정 전 rhwp 는 줄 폭 밖 정지를 단 시작 기준 줄 폭에서 잘라 `X` 를 탭 뒤 같은 줄에 두었다(한 줄).
//!
//! 이 규칙은 한/글 문서(호환 대상이 MS 워드가 아님)에만 적용한다. 워드 호환 문서는 한컴에서 줄
//! 나눔이 달라(`가`+탭 / `X` 두 줄) 이번 수정 대상이 아니고 종전 배치를 유지한다.
//! `samples/tab_overflow/tab_stop_beyond_line_msword.hwpx` 는 같은 재현물에서 `targetProgram` 만
//! `MS_WORD` 로 바꾼 것이다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn runs(node: &RenderNode, out: &mut Vec<(String, f64, f64)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        let text = run.text.trim().to_string();
        if !text.is_empty() {
            out.push((text, node.bbox.x, node.bbox.y));
        }
    }
    for child in &node.children {
        runs(child, out);
    }
}

fn layout() -> Vec<(String, f64, f64)> {
    layout_of("tab_stop_beyond_line.hwpx")
}

fn layout_of(name: &str) -> Vec<(String, f64, f64)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/tab_overflow")
        .join(name);
    let bytes = std::fs::read(&path).expect("재현물");
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    assert_eq!(core.page_count(), 1, "한 쪽");
    let tree = core.build_page_render_tree(0).unwrap();
    let mut out = Vec::new();
    runs(&tree.root, &mut out);
    out
}

fn find<'a>(runs: &'a [(String, f64, f64)], text: &str, nth: usize) -> &'a (String, f64, f64) {
    runs.iter()
        .filter(|(t, _, _)| t == text)
        .nth(nth)
        .unwrap_or_else(|| panic!("{text} #{nth}: {runs:?}"))
}

/// 라벨 줄 → 다음 라벨 줄 거리로 대상 문단의 줄 수를 잰다(줄 피치 14.72px).
fn lines_between(runs: &[(String, f64, f64)], label: &str, next: &str) -> f64 {
    (find(runs, next, 0).2 - find(runs, label, 0).2) / (11.04 * 96.0 / 72.0) - 1.0
}

#[test]
fn text_after_tab_with_stop_beyond_line_starts_two_lines_below() {
    let runs = layout();
    let ga = find(&runs, "가", 0);
    let x = find(&runs, "X", 0);
    let dy = x.2 - ga.2;
    assert!(
        (dy - 29.44).abs() <= 1.0,
        "`X` 는 `가` 보다 두 줄 아래(웹한글기안기 29.4px): {dy:.1}px"
    );
    assert!(
        (x.1 - ga.1).abs() <= 0.5,
        "`X` 는 줄 처음(`가` x {:.1}): {:.1}",
        ga.1,
        x.1
    );
    let lines = lines_between(&runs, "GA_TAB_X", "NEXT_GA_TAB_X");
    assert!(
        (lines - 3.0).abs() < 0.1,
        "`가`+탭+`X` 문단은 세 줄: {lines:.2}"
    );
}

#[test]
fn tab_only_and_plain_paragraphs_stay_one_line() {
    let runs = layout();
    for (label, next) in [("TABONLY", "NEXT_TABONLY"), ("NOTAB", "NEXT_NOTAB")] {
        let lines = lines_between(&runs, label, next);
        assert!(
            (lines - 1.0).abs() < 0.1,
            "{label} 문단은 한 줄: {lines:.2}"
        );
    }
}

#[test]
fn word_compat_document_does_not_take_hangul_three_line_rule() {
    let runs = layout_of("tab_stop_beyond_line_msword.hwpx");
    let lines = lines_between(&runs, "GA_TAB_X", "NEXT_GA_TAB_X");
    assert!(
        lines < 2.5,
        "워드 호환 문서의 `가`+탭+`X` 문단에는 한/글 세 줄 규칙을 쓰지 않는다: {lines:.2}"
    );
}
