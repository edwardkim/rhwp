//! 고정형(독립 Print 좌표·쪽수)·관계형(줄 소속·같은 시작점·호환 대조).
//! 주석 갱신: 2026-11-17 19:34:36 KST. 직전 파일 변경: 2026-11-17T18:30:14+09:00, ad80367f2b5784c69e1570b1130dba339a019ba3.
//! 측정: 2026-11-17, source 2c5e3fe387, canonical 2px raw / 96DPI / compat2024.
//! 원본 1쪽 Native 100.00% / fresh WASM 100.00%, 두 전쪽 최저값도 각각 100.00%.
//! 독립 데스크톱 한컴 2024 Hancom PDF Print: pdf/pr-review-open-prs-20261117/tab_stop_beyond_line-2024.pdf.
//! 근거: mydocs/pr/assets/open_prs_20261117/cases/tab/{native,wasm}/candidate/silhouette.tsv 및 review·overlay.
//! 양쪽 review·overlay를 직접 판독한 뒤 고정 좌표를 추가했다. 웹한글기안기 PDF는 판정에 사용하지 않는다.
//! 원본: samples/tab_overflow/tab_stop_beyond_line.hwpx. 탭 없는 대조의 실제 피치로 줄 수를 센다.
//! 워드 호환 파생본은 기존 규칙 제외 관계만 검사하며 별도 PDF 수치로 주장하지 않는다.
//! 독립 Print의 가·X 좌표 ±2px, 세 줄 소속·같은 시작점과 정상 대조를 함께 보호한다.

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

/// 탭 없는 대조 문단의 실제 줄 피치로 대상 문단의 줄 수를 잰다.
fn lines_between(runs: &[(String, f64, f64)], label: &str, next: &str) -> f64 {
    let plain_pitch = find(runs, "NEXT_NOTAB", 0).2 - find(runs, "가", 1).2;
    assert!(plain_pitch > 0.0, "탭 없는 문단과 다음 라벨의 순서");
    (find(runs, next, 0).2 - find(runs, label, 0).2) / plain_pitch - 1.0
}

#[test]
fn text_after_tab_with_stop_beyond_line_starts_two_lines_below() {
    let runs = layout();
    let ga = find(&runs, "가", 0);
    let x = find(&runs, "X", 0);
    let dy = x.2 - ga.2;
    assert!(
        (dy - 29.248036).abs() <= 2.0,
        "`X` 는 `가`보다 두 줄 아래(독립 데스크톱 Print 29.248px, ±2px): {dy:.4}px"
    );
    assert!(
        (x.1 - ga.1).abs() <= 0.5,
        "`X` 는 줄 처음(`가` x {:.1}): {:.1}",
        ga.1,
        x.1
    );
    for (run, pdf_y_pt) in [(ga, 159.262748), (x, 181.198775)] {
        let expected_x = 96.427740 * 96.0 / 72.0;
        let expected_y = pdf_y_pt * 96.0 / 72.0;
        assert!(
            (run.1 - expected_x).abs() <= 2.0,
            "독립 Print 시작 x: {run:?}"
        );
        assert!(
            (run.2 - expected_y).abs() <= 2.0,
            "독립 Print 줄 y: {run:?}"
        );
    }
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
