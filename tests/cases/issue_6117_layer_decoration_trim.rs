//! [Issue #6117] layer 재생 경로의 장식선(밑줄/취소선) 트림 대상 선정 계약.
//!
//! #6028 이 정한 규칙 — soft-wrap 이 소비한 줄-말미 공백은 장식선 길이에서 뺀다 —
//! 은 RenderNode 경로에만 배선돼 있었다. studio 는 layer tree 를 그대로 재생하므로
//! 그 배선이 없어 밑줄이 표 칸 괘선을 넘었다. 규칙 자체는
//! `layer_renderer::line_decoration_trim_target` 한 곳으로 모았고, 이 테스트가 그
//! 판별을 고정한다(캔버스 픽셀 검증은
//! `rhwp-studio/e2e/issue-6117-cell-underline-canvas2d.test.mjs`).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::paint::{LayerNode, PaintOp};
use rhwp::renderer::layer_renderer::line_decoration_trim_target;
use rhwp::renderer::render_tree::{BoundingBox, TextRunNode};
use rhwp::renderer::TextStyle;

fn run(text: &str, para_end: bool) -> TextRunNode {
    TextRunNode {
        text: text.to_string(),
        style: TextStyle::default(),
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
        display_text: None,
    }
}

fn leaf(runs: Vec<TextRunNode>) -> LayerNode {
    let bbox = BoundingBox::new(0.0, 0.0, 10.0, 10.0);
    let ops: Vec<PaintOp> = runs
        .into_iter()
        .map(|r| PaintOp::text_run(bbox, r))
        .collect();
    LayerNode::leaf(bbox, None, ops)
}

/// soft-wrap 줄의 마지막 가시 run 의 말미 공백이 트림 대상이다.
#[test]
fn issue_6117_trims_trailing_spaces_of_the_last_visible_run() {
    let children = vec![leaf(vec![run("가나", false), run("다라  ", false)])];
    let (_, trim) = line_decoration_trim_target(&children)
        .expect("말미 공백이 있는 soft-wrap 줄은 트림 대상이다");
    assert_eq!(trim, 2);
}

/// 문단 끝 줄의 말미 공백은 저자 콘텐츠(밑줄 서명란 등)라 트림하지 않는다.
#[test]
fn issue_6117_keeps_author_spaces_on_a_paragraph_end_run() {
    let children = vec![leaf(vec![run("서명란   ", true)])];
    assert!(
        line_decoration_trim_target(&children).is_none(),
        "문단 끝 줄의 말미 공백은 트림 대상이 아니다"
    );
}

/// 공백뿐인 꼬리 run 은 건너뛰고 그 앞의 가시 run 을 본다.
#[test]
fn issue_6117_ignores_space_only_runs_after_the_text() {
    let children = vec![leaf(vec![run("본문  ", false), run("   ", false)])];
    let (_, trim) = line_decoration_trim_target(&children)
        .expect("공백뿐인 꼬리 run 뒤에도 앞의 가시 run 이 대상이다");
    assert_eq!(trim, 2);
}
