//! [Issue #6117] layer 재생 경로의 장식선(밑줄/취소선) 트림 대상 선정 계약.
//!
//! #6028 이 정한 규칙 — soft-wrap 이 소비한 줄-말미 공백은 장식선 길이에서 뺀다 —
//! 은 RenderNode 경로에만 배선돼 있었다. studio 는 layer tree 를 그대로 재생하므로
//! 그 배선이 없어 밑줄이 표 칸 괘선을 넘었다. 이제 LayerBuilder가 최종
//! TextDecoration에 트림을 발행하며, 이 테스트가 그 결정을 고정한다(캔버스 픽셀 검증은
//! `rhwp-studio/e2e/issue-6117-cell-underline-canvas2d.test.mjs`).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::paint::{LayerBuilder, LayerNode, LayerNodeKind, PaintOp, RenderProfile};
use rhwp::renderer::render_tree::{
    BoundingBox, PageNode, PageRenderTree, RenderNode, RenderNodeType, TextLineNode, TextRunNode,
};
use rhwp::renderer::TextStyle;

fn run(text: &str, para_end: bool) -> TextRunNode {
    TextRunNode {
        text: text.to_string(),
        style: TextStyle {
            underline: rhwp::model::style::UnderlineType::Bottom,
            ..Default::default()
        },
        char_shape_id: None,
        para_shape_id: None,
        section_index: None,
        para_index: None,
        char_start: None,
        cell_context: None,
        is_para_end: para_end,
        is_line_break_end: false,
        rotation: 0.0,
        is_vertical: false,
        char_overlap: None,
        border_fill_id: 0,
        baseline: 12.0,
        field_marker: Default::default(),
        layout_positions: None,
        display_text: None,
    }
}

fn published_trims(runs: Vec<TextRunNode>) -> Vec<usize> {
    let bbox = BoundingBox::new(0.0, 0.0, 10.0, 10.0);
    let mut tree = PageRenderTree::new(0, 20.0, 20.0);
    tree.root.node_type = RenderNodeType::Page(PageNode {
        page_index: 0,
        width: 20.0,
        height: 20.0,
        section_index: 0,
    });
    let mut line = RenderNode::new(
        1,
        RenderNodeType::TextLine(TextLineNode::new(10.0, 8.0)),
        bbox,
    );
    for (index, run) in runs.into_iter().enumerate() {
        line.children.push(RenderNode::new(
            10 + index as u32,
            RenderNodeType::TextRun(run),
            bbox,
        ));
    }
    tree.root.children.push(line);
    let layer = LayerBuilder::new(RenderProfile::Screen).build(&tree);
    fn collect(node: &LayerNode, trims: &mut Vec<usize>) {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => {
                for child in children {
                    collect(child, trims);
                }
            }
            LayerNodeKind::ClipRect { child, .. } => collect(child, trims),
            LayerNodeKind::Leaf { ops } => trims.extend(ops.iter().filter_map(|op| match op {
                PaintOp::TextDecoration {
                    trim_trailing_spaces,
                    ..
                } => Some(*trim_trailing_spaces),
                _ => None,
            })),
        }
    }
    let mut trims = Vec::new();
    collect(&layer.root, &mut trims);
    trims
}

/// soft-wrap 줄의 마지막 가시 run 의 말미 공백이 트림 대상이다.
#[test]
fn issue_6117_trims_trailing_spaces_of_the_last_visible_run() {
    assert_eq!(
        published_trims(vec![run("가나", false), run("다라  ", false)]),
        vec![0, 2]
    );
}

/// 문단 끝 줄의 말미 공백은 저자 콘텐츠(밑줄 서명란 등)라 트림하지 않는다.
#[test]
fn issue_6117_keeps_author_spaces_on_a_paragraph_end_run() {
    assert_eq!(published_trims(vec![run("서명란   ", true)]), vec![0]);
}

/// 공백뿐인 꼬리 run 은 건너뛰고 그 앞의 가시 run 을 본다.
#[test]
fn issue_6117_ignores_space_only_runs_after_the_text() {
    assert_eq!(
        published_trims(vec![run("본문  ", false), run("   ", false)]),
        vec![2, 0]
    );
}
