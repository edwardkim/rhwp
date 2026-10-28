//! 관계형·고정형: 작성된 말미 공백 때문에 가용 폭 안의 마지막 줄을 압축하지 않는다.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! HWP 373쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 373쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 두 원본 373쪽의 독립 Print 전진과 같은 글줄의 원문·소유를 보호한다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    renderer::{
        layout::{EmbeddedTextMeasurer, TextMeasurer},
        render_tree::{RenderNode, RenderNodeType, TextRunNode},
    },
    DocumentCore,
};
const TEXT: &str = "이상의 크기로 뒤쪽에 작성란이 있음을 강조하여 표시한다. ";
fn runs<'a>(node: &'a RenderNode, result: &mut Vec<&'a TextRunNode>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        result.push(run);
    }
    for child in &node.children {
        runs(child, result);
    }
}
fn run_nodes<'a>(node: &'a RenderNode, result: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::TextRun(_)) {
        result.push(node);
    }
    for child in &node.children {
        run_nodes(child, result);
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
    // 독립 Print에서 관찰한 전진 범위에 96dpi 기준 ±2px를 허용한다.
    let tolerance = 2.0;
    assert!(
        (low - tolerance..=high + tolerance).contains(&value),
        "독립 Print의 {label}: {low}..{high}, 실제 {value}"
    );
}
fn check(extension: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 383, "독립 Print 전체 쪽수");
    let tree = core.build_page_render_tree(372).unwrap();
    let mut found = Vec::new();
    lines(&tree.root, &mut found);
    assert_eq!(
        found.len(),
        1,
        "원래 마지막 글줄이 누락·중복 없이 한 번 표시된다"
    );
    let mut actual = Vec::new();
    runs(found[0], &mut actual);
    assert_eq!(actual.len(), 3, "원래 세 글자모양의 글줄 소유");
    assert_eq!(actual[0].display_or_text(), "이");
    assert_eq!(
        actual[1].display_or_text(),
        "상의 크기로 뒤쪽에 작성란이 있음을 강조하여 표시한"
    );
    assert_eq!(actual[2].display_or_text(), "다. ");
    for run in &actual {
        assert_eq!(run.section_index, actual[0].section_index);
        assert_eq!(run.para_index, actual[0].para_index);
        assert!(run.cell_context.is_some(), "원래 표 셀 소유");
    }
    let mid = positions(actual[1]);
    in_range(mid[1] - mid[0], 10.5599, 10.7205, "373쪽 한글 전진");
    let i = actual[1]
        .display_or_text()
        .chars()
        .position(|c| c == ' ')
        .unwrap();
    in_range(mid[i + 1] - mid[i], 5.5999, 5.7608, "373쪽 내부 공백 전진");
    let mut nodes = Vec::new();
    run_nodes(found[0], &mut nodes);
    let first_x = nodes[0].bbox.x + positions(actual[0])[0];
    let dot_x = nodes[2].bbox.x + positions(actual[2])[1];
    assert!(
        ((dot_x - first_x) - 290.88).abs() <= 2.0,
        "독립 Print의 첫 글자와 마지막 점의 원점 간격: 기대 290.88±2px, 실제 {}",
        dot_x - first_x
    );
    let first = positions(actual[0]);
    in_range(
        first[1] - first[0],
        11.0399,
        11.2009,
        "373쪽 다른 글자모양의 첫 글자 전진",
    );
}
#[test]
fn hwp_plain_terminal_space_keeps_the_fitting_visible_last_line() {
    check("hwp");
}
#[test]
fn hwpx_plain_terminal_space_keeps_the_fitting_visible_last_line() {
    check("hwpx");
}
