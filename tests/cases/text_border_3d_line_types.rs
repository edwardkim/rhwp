//! 입체선(14~17)으로 지정한 글자·문단 테두리는 긋지 않는다.
//!
//! rhwp로 만든 시험 문서를 한컴 한글 12.30(Mac)에서 인쇄한 PDF
//! (`pdf/border_line_types/`)에는 3D, 3D 반전, 3D 가는선, 3D 가는선 반전 네 종류의
//! 글자 테두리와 문단 테두리가 하나도 없다. 같은 문서의 실선·점선·원형 점선·물결선
//! 테두리는 모두 그려져 있다. 수정 전 rhwp는 네 종류를 모두 실선으로 그렸다.
//! 표 칸·쪽 테두리의 입체선은 대조 근거가 없어 그대로 둔다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "tests/fixtures/border_line_types/borders.hwp";

fn line_text(node: &RenderNode) -> String {
    node.children
        .iter()
        .filter_map(|child| match &child.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
            _ => None,
        })
        .collect()
}

fn is_stroke(node: &RenderNode) -> bool {
    matches!(
        node.node_type,
        RenderNodeType::Line(_) | RenderNodeType::Rectangle(_) | RenderNodeType::Path(_)
    )
}

fn collect<'a>(
    node: &'a RenderNode,
    lines: &mut Vec<(String, &'a RenderNode)>,
    strokes: &mut Vec<BoundingBox>,
) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        lines.push((line_text(node), node));
    }
    if is_stroke(node) {
        strokes.push(node.bbox);
    }
    for child in &node.children {
        collect(child, lines, strokes);
    }
}

#[test]
fn three_d_character_and_paragraph_borders_are_not_drawn() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core.build_page_render_tree(0).expect("page 1 render tree");
    let (mut lines, mut strokes) = (Vec::new(), Vec::new());
    collect(&page.root, &mut lines, &mut strokes);

    for kind in 14..=17 {
        // 글자 테두리: 테두리선은 그 글줄의 자식으로 들어간다.
        let label = format!("{kind} 3D");
        let (_, line) = lines
            .iter()
            .find(|(text, _)| text.starts_with(&label))
            .unwrap_or_else(|| panic!("{label} 글줄"));
        assert!(
            !line.children.iter().any(is_stroke),
            "{label} 글자 테두리를 그렸다"
        );

        // 문단 테두리: 글줄 둘레 1mm 간격에 놓일 선이 없어야 한다.
        let label = format!("paragraph border type {kind}");
        let (_, line) = lines
            .iter()
            .find(|(text, _)| text.contains(&label))
            .unwrap_or_else(|| panic!("{label} 글줄"));
        let b = line.bbox;
        let around = BoundingBox::new(b.x - 8.0, b.y - 8.0, b.width + 16.0, b.height + 16.0);
        assert!(
            !strokes.iter().any(|s| s.intersects(&around)),
            "{label} 문단 테두리를 그렸다"
        );
    }

    // 대조: 같은 문서의 실선 문단 테두리는 그대로 그린다.
    let (_, solid) = lines
        .iter()
        .find(|(text, _)| text.contains("paragraph border type 1"))
        .expect("실선 문단");
    let b = solid.bbox;
    let around = BoundingBox::new(b.x - 8.0, b.y - 8.0, b.width + 16.0, b.height + 16.0);
    assert!(
        strokes.iter().any(|s| s.intersects(&around)),
        "실선 문단 테두리"
    );
}
