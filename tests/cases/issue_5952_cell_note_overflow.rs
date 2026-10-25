//! [#5952] 편람 유의사항의 내용과 상자·사이드바·다음 제목의 관계를 보존한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn text(node: &RenderNode) -> String {
    let mut result = match &node.node_type {
        RenderNodeType::TextRun(run) => run.text.clone(),
        _ => String::new(),
    };
    for child in &node.children {
        result.push_str(&text(child));
    }
    result
}

fn compact(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

fn nodes(root: &RenderNode) -> Vec<&RenderNode> {
    let mut result = vec![root];
    for child in &root.children {
        result.extend(nodes(child));
    }
    result
}

fn note(root: &RenderNode) -> &RenderNode {
    let found: Vec<_> = nodes(root)
        .into_iter()
        .filter(|n| {
            matches!(n.node_type, RenderNodeType::Table(_))
                && text(n).contains("기안문 제목과 붙임물의 제목")
        })
        .collect();
    assert_eq!(found.len(), 1, "유의사항 상자의 누락·중복");
    found[0]
}

/// [#5952] 저장 2줄이 composed 1줄로 접혀 셀 내폭을 넘치면 fresh 재래핑한다.
/// source-side `#[cfg(test)]` 가 아니라 integration 으로 둔다 (RustUnitTier).
#[test]
fn collapsed_stored_two_lines_rewrap_when_over_cell() {
    use rhwp::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
    use rhwp::renderer::composer::{
        compose_paragraph, recompose_stored_single_line_if_overflowing,
    };
    use rhwp::renderer::style_resolver::ResolvedStyleSet;

    let styles = ResolvedStyleSet::default();
    let text = format!("※{}", "가".repeat(39));
    let n = text.chars().count();
    let collapsed = Paragraph {
        text: text.clone(),
        char_offsets: (0..n as u32).collect(),
        char_count: n as u32 + 1,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![LineSeg {
            text_start: 0,
            line_height: 800,
            baseline_distance: 640,
            ..Default::default()
        }],
        ..Default::default()
    };
    let stored_two = Paragraph {
        line_segs: vec![
            LineSeg {
                text_start: 0,
                line_height: 800,
                baseline_distance: 640,
                segment_width: 3000,
                ..Default::default()
            },
            LineSeg {
                text_start: 20,
                line_height: 800,
                baseline_distance: 640,
                segment_width: 3000,
                ..Default::default()
            },
        ],
        ..collapsed.clone()
    };
    let mut composed = compose_paragraph(&collapsed);
    assert_eq!(composed.lines.len(), 1, "1-ls 문단은 1줄로 시작한다");
    recompose_stored_single_line_if_overflowing(&mut composed, &stored_two, 40.0, &styles, 96.0);
    assert!(
        composed.lines.len() > 1,
        "유의사항 bullet 저장 2줄이 셀을 넘치면 재래핑돼야 함 (#5952)"
    );
}

/// 끝의 빈 LINE_SEG는 저장 행이 접힌 결함이 아니므로 재래핑하지 않는다.
#[test]
fn trailing_empty_stored_line_seg_is_preserved() {
    use rhwp::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
    use rhwp::renderer::composer::{
        compose_paragraph, recompose_stored_single_line_if_overflowing,
    };
    use rhwp::renderer::style_resolver::ResolvedStyleSet;

    let styles = ResolvedStyleSet::default();
    let text = "가".repeat(40);
    let n = text.chars().count();
    let stored_two = Paragraph {
        text: text.clone(),
        char_offsets: (0..n as u32).collect(),
        char_count: n as u32 + 1,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![
            LineSeg {
                text_start: 0,
                line_height: 800,
                baseline_distance: 640,
                segment_width: 3000,
                ..Default::default()
            },
            LineSeg {
                text_start: n as u32,
                line_height: 800,
                baseline_distance: 640,
                segment_width: 3000,
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    let mut composed = compose_paragraph(&stored_two);
    assert_eq!(
        composed.lines.len(),
        2,
        "빈 둘째 저장 행도 composed에 남는다"
    );
    recompose_stored_single_line_if_overflowing(&mut composed, &stored_two, 40.0, &styles, 96.0);
    assert!(
        composed.lines[1].runs.is_empty(),
        "끝의 빈 저장 행은 fresh 재래핑으로 채우지 않아야 함 (#5952)"
    );
}
