//! 관계형: #4138의 `14 | 0℃` 줄 소속과 문단의 누락·중복 없는 순서를 검사한다.
//! 주석 갱신: 2026-11-06T22:13:05+09:00; 직전 파일 커밋: 2026-11-06T21:09:45+09:00 (fa0436f495).
//! 2026-11-06 raw(192DPI, PDFium, 2px): 90쪽 Native 99.99501% / fresh WASM 100.00000%.
//! 전197쪽 최저 91.17674% / 91.29951%; 양쪽 각7쪽은 98% 미만으로 보정 중이다.
//! 절대 px 좌표 검사는 별도 파일로 유지하며 이 관계 검사는 전체98% 완료를 뜻하지 않는다.
//! 근거: mydocs/pr/assets/issue4138/numeric-closing-20261106/README.md.

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn text(node: &RenderNode) -> String {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        return run.display_or_text().to_owned();
    }
    node.children.iter().map(text).collect()
}

fn lines(node: &RenderNode, found: &mut Vec<String>) {
    if matches!(&node.node_type, RenderNodeType::TextLine(meta)
        if meta.para_index == Some(1201))
    {
        found.push(text(node));
    }
    for child in &node.children {
        lines(child, found);
    }
}

#[test]
fn split_temperature_retains_the_print_line_membership_and_all_text() {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "mydocs/pr/assets/issue7445/completed-20261009/regression-policy-20261015/issue4138-current-production/saved-split-1x2.hwp",
    );
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let Control::Table(source) = &core.document().sections[0].paragraphs[0].controls[2] else {
        panic!("분할 저장 원본 표");
    };
    let original = &source
        .cells
        .iter()
        .find(|cell| cell.row == 2 && cell.col == 0)
        .unwrap()
        .paragraphs[1201]
        .text;
    let page = core.build_page_render_tree(89).unwrap();
    let mut placed = Vec::new();
    lines(&page.root, &mut placed);
    assert!(placed
        .iter()
        .any(|line| line.trim_end().ends_with("보다 14")));
    assert!(placed
        .iter()
        .any(|line| line.trim_start().starts_with("0℃")));
    let normalized = |value: &str| {
        value
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect::<String>()
    };
    assert_eq!(
        normalized(&placed.concat()),
        normalized(original),
        "원문 문단은 같은 페이지에 누락·중복 없이 한 번 표시됨"
    );
}
