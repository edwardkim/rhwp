//! 관계형: 가운데 정렬의 원래 글자·공백 fit을 공백 전용 압축으로 바꾸지 않는다.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! HWP 46쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 46쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 두 원본 46쪽의 독립 Print 전진과 같은 글줄의 원문·소유를 보호한다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    renderer::{
        layout::{EmbeddedTextMeasurer, TextMeasurer},
        render_tree::{RenderNode, RenderNodeType, TextRunNode},
    },
    DocumentCore,
};
const TEXT: &str = "기업 간 거래, 기업 간 거래(B2B)";
fn runs<'a>(node: &'a RenderNode, result: &mut Vec<&'a TextRunNode>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        result.push(run);
    }
    for child in &node.children {
        runs(child, result);
    }
}
fn lines<'a>(node: &'a RenderNode, result: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let mut current = Vec::new();
        runs(node, &mut current);
        let text: String = current.iter().map(|run| run.display_or_text()).collect();
        if text == TEXT {
            result.push(node);
        }
    }
    for child in &node.children {
        lines(child, result);
    }
}
fn positions(run: &TextRunNode) -> Vec<f64> {
    run.layout_positions.clone().unwrap_or_else(|| {
        EmbeddedTextMeasurer.compute_char_positions(run.display_or_text(), &run.style)
    })
}
fn in_range(value: f64, low: f64, high: f64, label: &str) {
    // 독립 Print의 두 원점 차이에는 600dpi 한 장치 픽셀의 양자화 오차가 있다.
    let hu = 96.0 / 7200.0 + 96.0 / 600.0;
    assert!(
        (low - hu..=high + hu).contains(&value),
        "독립 Print의 {label}: {low}..{high}, 실제 {value}"
    );
}
fn check(extension: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 383, "독립 Print 전체 쪽수");
    let tree = core.build_page_render_tree(45).unwrap();
    let mut found = Vec::new();
    lines(&tree.root, &mut found);
    assert_eq!(
        found.len(),
        1,
        "원래 마지막 글줄이 누락·중복 없이 한 번 표시된다"
    );
    let mut actual = Vec::new();
    runs(found[0], &mut actual);
    let found: Vec<_> = actual
        .iter()
        .filter(|r| {
            r.display_or_text()
                .starts_with("기업 간 거래, 기업 간 거래(")
        })
        .collect();
    assert_eq!(found.len(), 1, "원래 고유한 한국어 run");
    let run = found[0];
    for part in &actual {
        assert_eq!(part.section_index, run.section_index);
        assert_eq!(part.para_index, run.para_index);
        assert!(part.cell_context.is_some(), "같은 표 셀 소유");
    }
    let pos = positions(run);
    in_range(pos[1] - pos[0], 9.4399, 9.6009, "46쪽 한글 전진");
    let i = run
        .display_or_text()
        .chars()
        .position(|c| c == ' ')
        .unwrap();
    in_range(pos[i + 1] - pos[i], 4.6399, 4.8009, "46쪽 내부 공백 전진");
}
#[test]
fn hwp_centered_condense_keeps_independent_glyph_and_space_advances() {
    check("hwp");
}
#[test]
fn hwpx_centered_condense_keeps_independent_glyph_and_space_advances() {
    check("hwpx");
}
