//! 관계형·고정형: Justify의 마지막 줄도 같은 공백 한계·잔여 글자 사이 압축을 소비한다.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! HWP 148쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 327쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 148쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 327쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 독립 2024 Print의 148·327쪽에서 관측한 글리프/공백 전진 구간이다.
//! 절대 화면 좌표 핀이 아니며 원본 내용·소유와 실제 positions를 함께 검사한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    renderer::{
        layout::{EmbeddedTextMeasurer, TextMeasurer},
        render_tree::{RenderNode, RenderNodeType, TextRunNode},
    },
    DocumentCore,
};

fn collect<'a>(node: &'a RenderNode, out: &mut Vec<&'a TextRunNode>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push(run);
    }
    for child in &node.children {
        collect(child, out);
    }
}

fn unique<'a>(runs: &[&'a TextRunNode], prefix: &str) -> &'a TextRunNode {
    let found: Vec<_> = runs
        .iter()
        .copied()
        .filter(|run| run.display_or_text().starts_with(prefix))
        .collect();
    assert_eq!(found.len(), 1, "원본 쪽의 고유 내용: {prefix}");
    found[0]
}

fn advance(run: &TextRunNode, ch: char) -> f64 {
    let text = run.display_or_text();
    let index = text.chars().position(|c| c == ch).unwrap();
    let positions = run
        .layout_positions
        .clone()
        .unwrap_or_else(|| EmbeddedTextMeasurer.compute_char_positions(text, &run.style));
    positions[index + 1] - positions[index]
}

fn in_print_range(actual: f64, low: f64, high: f64, label: &str) {
    // 독립 Print에서 관찰한 전진 범위에 96dpi 기준 ±2px를 허용한다.
    let tolerance = 2.0;
    assert!(
        (low - tolerance..=high + tolerance).contains(&actual),
        "독립 Print의 {label} 전진 구간 {low}..{high}, 실제 {actual}"
    );
}

fn check(extension: &str, page: u32) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 383, "독립 Print 전체 쪽수");
    let tree = core.build_page_render_tree(page - 1).unwrap();
    let mut runs = Vec::new();
    collect(&tree.root, &mut runs);
    if page == 148 {
        let prefix = unique(&runs, "생산등록번호");
        let tail = unique(&runs, "시행일 및 접수일 등의 항목을 넣어 설계한다.");
        assert_eq!(prefix.section_index, tail.section_index);
        assert_eq!(prefix.para_index, tail.para_index);
        assert_eq!(prefix.char_shape_id, tail.char_shape_id);
        in_print_range(advance(prefix, '생'), 13.5993, 13.7608, "148쪽 한글");
        in_print_range(advance(tail, ' '), 5.7591, 5.9206, "148쪽 공백");
    } else {
        let run = unique(&runs, "관리에 관한 법률 시행령");
        let tail = unique(&runs, "조에 따른 등록정보로 관리하여야 한다.");
        assert_eq!(run.section_index, tail.section_index);
        assert_eq!(run.para_index, tail.para_index);
        assert_eq!(run.char_shape_id, tail.char_shape_id);
        in_print_range(advance(run, '관'), 11.5198, 11.6798, "327쪽 한글");
        in_print_range(advance(run, ' '), 5.2798, 5.2809, "327쪽 공백");
    }
}

#[test]
fn hwp_p148_justified_final_line_keeps_the_print_word_and_glyph_contract() {
    check("hwp", 148);
}

#[test]
fn hwpx_p148_justified_final_line_keeps_the_print_word_and_glyph_contract() {
    check("hwpx", 148);
}

#[test]
fn hwp_p327_justified_final_line_keeps_the_print_word_and_glyph_contract() {
    check("hwp", 327);
}

#[test]
fn hwpx_p327_justified_final_line_keeps_the_print_word_and_glyph_contract() {
    check("hwpx", 327);
}
