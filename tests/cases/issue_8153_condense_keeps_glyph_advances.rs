//! #8153: 공백 최소값으로 들어가는 저장 줄은 원 글자·들여쓰기 전진을 압축하지 않는다.
//! 같은 원본·글자모양의 짧은 정상 줄을 대조군으로 사용한다. 절대 쪽 좌표 핀이 아니다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::{
    layout::{EmbeddedTextMeasurer, TextMeasurer},
    render_tree::{RenderNode, RenderNodeType, TextRunNode},
};
use rhwp::{model::control::Control, DocumentCore};

fn collect<'a>(node: &'a RenderNode, out: &mut Vec<&'a TextRunNode>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push(run);
    }
    for child in &node.children {
        collect(child, out);
    }
}

fn unique<'a>(runs: &[&'a TextRunNode], needle: &str) -> &'a TextRunNode {
    let found: Vec<_> = runs
        .iter()
        .copied()
        .filter(|run| run.display_or_text().contains(needle))
        .collect();
    assert_eq!(found.len(), 1, "원본 200쪽의 고유 내용: {needle}");
    found[0]
}

fn positions(run: &TextRunNode) -> Vec<f64> {
    run.layout_positions.clone().unwrap_or_else(|| {
        EmbeddedTextMeasurer.compute_char_positions(run.display_or_text(), &run.style)
    })
}

fn advance(run: &TextRunNode, character: char) -> f64 {
    let i = run
        .display_or_text()
        .chars()
        .position(|ch| ch == character)
        .unwrap();
    let positions = positions(run);
    positions[i + 1] - positions[i]
}

fn check(extension: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 383, "독립 2024 Print 전체 쪽수");
    let source = &core.document().sections[7].paragraphs[71];
    let table = source
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table),
            _ => None,
        })
        .unwrap();
    let source = &table.cells[8].paragraphs[3];
    let shape = &core.document().doc_info.para_shapes[source.para_shape_id as usize];
    assert_eq!((shape.attr1 >> 9) & 0x7f, 30, "원본의 공백 최소값");
    assert_eq!(source.line_segs.len(), 2);
    let tree = core.build_page_render_tree(199).unwrap();
    let mut runs = Vec::new();
    collect(&tree.root, &mut runs);
    let full = unique(&runs, "예외적으로");
    let short = unique(&runs, "및 관리 현황 보고");
    let same_owner: Vec<_> = runs
        .iter()
        .copied()
        .filter(|run| run.para_index == full.para_index && run.section_index == full.section_index)
        .collect();
    let prefixes: Vec<_> = same_owner
        .iter()
        .copied()
        .filter(|run| run.display_or_text().starts_with("  "))
        .collect();
    assert_eq!(prefixes.len(), 1, "같은 원문 줄의 선두 공백 소유");
    let prefix = prefixes[0];
    let pointer = unique(&same_owner, "☞");
    for other in [short, prefix] {
        assert_eq!(full.char_shape_id, other.char_shape_id);
        assert_eq!(full.style.font_family, other.style.font_family);
        assert_eq!(full.style.font_size, other.style.font_size);
        assert_eq!(full.style.ratio, other.style.ratio);
        assert_eq!(full.style.letter_spacing, other.style.letter_spacing);
    }
    // 독립 Print의 두 줄은 같은 '관'을 같은 자연 전진으로 그리고,
    // 긴 줄의 내부 공백만 줄인다. HU 한 단위는 같은 원본 메트릭의 오차 경계다.
    let hu = 96.0 / 7200.0;
    assert!(
        (advance(full, '관') - advance(short, '관')).abs() <= hu,
        "{extension}: 공백 압축이 글자의 전진을 바꾸면 안 된다: 긴 줄 {}, 정상 줄 {}",
        advance(full, '관'),
        advance(short, '관')
    );
    let natural_space = advance(short, ' ');
    assert!(
        advance(full, ' ') < natural_space,
        "실제 내부 공백이 압축을 소비한다"
    );
    let prefix_positions = positions(prefix);
    for span in prefix_positions[..3].windows(2) {
        assert!(
            (span[1] - span[0] - natural_space).abs() <= hu,
            "줄 선두 들여쓰기 공백은 자연 전진을 유지한다"
        );
    }
    assert!(
        (advance(pointer, '\u{a0}') - natural_space).abs() <= hu,
        "원본 nbSpace는 줄 나눔용 ASCII 공백 압축 대상이 아니다"
    );
    assert_eq!(full.display_or_text().matches("위원회").count(), 2);
    assert_eq!(short.display_or_text(), "및 관리 현황 보고");
}

#[test]
fn hwp_condense_changes_interior_spaces_and_keeps_source_glyph_advance() {
    check("hwp");
}

#[test]
fn hwpx_condense_changes_interior_spaces_and_keeps_source_glyph_advance() {
    check("hwpx");
}
