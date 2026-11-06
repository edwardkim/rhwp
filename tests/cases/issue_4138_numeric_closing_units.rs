//! Issue #4138: 분할 저장 셀의 숫자·줄머리 금칙 소속을 독립 Print와 맞춘다.
//!
//! 검사 유형: 관계형. `14 | 0℃`의 줄 소속과 문단의 누락·중복 없는 순서를 검사한다.
//! 절대 px 좌표 고정형이 아니다. 기준 PDF는 issue4138-ascii-split-20261016-2024.pdf
//! 90쪽이며, 숫자·섭씨/퍼센트/영문 단위의 차이를 별도 한컴 2024 Print로 교차 확인했다.
//! 수정 전 Native 90쪽 89.85554%, fresh WASM 90.20277%는 이전 코드의 결과다.
//! 새 후보의 전 197쪽 Native/fresh WASM 98% 수용은 별도로 확인한다.
//! 주석 갱신: 2026-11-06 21:06 KST.

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
    let original = &source.cells.iter().find(|cell| cell.row == 2 && cell.col == 0)
        .unwrap().paragraphs[1201].text;
    let page = core.build_page_render_tree(89).unwrap();
    let mut placed = Vec::new();
    lines(&page.root, &mut placed);
    assert!(placed.iter().any(|line| line.trim_end().ends_with("보다 14")));
    assert!(placed.iter().any(|line| line.trim_start().starts_with("0℃")));
    let normalized = |value: &str| value.chars().filter(|ch| !ch.is_whitespace()).collect::<String>();
    assert_eq!(normalized(&placed.concat()), normalized(original),
        "원문 문단은 같은 페이지에 누락·중복 없이 한 번 표시됨");
}
