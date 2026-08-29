//! 흐름 겹침 판정의 글자-상자 의미론 — 공개 `scan_page` 경유 행동 판.
//!
//! 종전 src 쪽 white-box 시험 8개(`flow_extent_bbox`/`glyph_band_bbox` 기하 단위시험)를
//! 검출/비검출 **경계**로 옮긴 판이다. 각 의미가 판정 결과를 실제로 바꾸는 지점에서
//! 검증하므로 사용자-가시 계약이 된다. 대응:
//! - 줄 간격 걷어내기(strips_line_leading·keeps_box_without_leading·unions 세로):
//!   상자는 겹치되 글자 상자는 안 겹치는 쌍 → 비검출, 간격 없는 대조군 → 검출
//! - 런 합집합(unions 가로): 두 런 사이 틈을 겹는 상대 → 검출(합집합이 틈을 포함)
//! - 비텍스트 노드 원상자(leaves_non_text_nodes_alone·only_applies_to_text_runs):
//!   표는 자기 bbox 그대로 → 줄 간격 안 걷은 겹침도 검출
//! - 안 넓힘(never_grows_the_box): font_size > 줄높이 여도 상자 밖을 침범하지 않음
//! - font_size=0(needs_a_font_size): 근거 없으면 안 걷음 → 상자 겹침 그대로 검출
//! - 무런 줄 fallback: 스캔 경로에서는 무런 줄이 애초에 겹침 후보가 아니므로
//!   (후보 조건 = 보이는 런 보유) 방어 분기다 — 여기서는 후보 조건 자체를 검증한다.

use rhwp::diagnostics::layout_anomaly::{scan_page, AnomalyOptions};
use rhwp::renderer::render_tree::{
    BoundingBox, FieldMarkerType, PageNode, RenderNode, RenderNodeType, TableNode, TextLineNode,
    TextRunNode,
};

fn page_root(children: Vec<RenderNode>) -> RenderNode {
    let mut body = RenderNode::new(
        1,
        RenderNodeType::Body { clip_rect: None },
        BoundingBox::new(0.0, 0.0, 1000.0, 1000.0),
    );
    body.children = children;
    let mut root = RenderNode::new(
        0,
        RenderNodeType::Page(PageNode {
            page_index: 0,
            width: 1000.0,
            height: 1000.0,
            section_index: 0,
        }),
        BoundingBox::new(0.0, 0.0, 1000.0, 1000.0),
    );
    root.children.push(body);
    root
}

fn run(text: &str, x: f64, y: f64, w: f64, h: f64, font_size: f64) -> RenderNode {
    let mut style = rhwp::renderer::TextStyle::default();
    style.font_size = font_size;
    RenderNode::new(
        100,
        RenderNodeType::TextRun(TextRunNode {
            text: text.to_string(),
            style,
            char_shape_id: None,
            para_shape_id: None,
            section_index: None,
            para_index: None,
            char_start: None,
            cell_context: None,
            is_para_end: false,
            is_line_break_end: false,
            rotation: 0.0,
            is_vertical: false,
            char_overlap: None,
            border_fill_id: 0,
            baseline: 0.0,
            field_marker: FieldMarkerType::default(),
            layout_positions: None,
            display_text: None,
        }),
        BoundingBox::new(x, y, w, h),
    )
}

fn line_with_runs(x: f64, y: f64, w: f64, h: f64, runs: Vec<RenderNode>) -> RenderNode {
    let mut line = RenderNode::new(
        99,
        RenderNodeType::TextLine(TextLineNode::new(h, h * 0.8)),
        BoundingBox::new(x, y, w, h),
    );
    line.children = runs;
    line
}

fn overlap_count(children: Vec<RenderNode>) -> usize {
    let root = page_root(children);
    scan_page(1, &root, 10, &AnomalyOptions::default())
        .overlap
        .len()
}

/// 줄 상자는 8px 겹치지만 글자 상자(fs=12, 위아래 8px 간격 제거)는 안 겹친다 → 비검출.
/// 같은 기하에서 줄 간격이 없으면(fs=28) 상자 그대로라 겹침 → 검출.
#[test]
fn leading_strip_changes_the_verdict() {
    let stripped = overlap_count(vec![
        line_with_runs(
            10.0,
            100.0,
            200.0,
            28.0,
            vec![run("가", 10.0, 100.0, 200.0, 28.0, 12.0)],
        ),
        line_with_runs(
            10.0,
            120.0,
            200.0,
            28.0,
            vec![run("나", 10.0, 120.0, 200.0, 28.0, 12.0)],
        ),
    ]);
    assert_eq!(stripped, 0, "줄 간격을 걷어낸 글자 상자끼리는 안 겹친다");

    let raw = overlap_count(vec![
        line_with_runs(
            10.0,
            100.0,
            200.0,
            28.0,
            vec![run("가", 10.0, 100.0, 200.0, 28.0, 28.0)],
        ),
        line_with_runs(
            10.0,
            120.0,
            200.0,
            28.0,
            vec![run("나", 10.0, 120.0, 200.0, 28.0, 28.0)],
        ),
    ]);
    assert_eq!(
        raw, 1,
        "줄 간격이 없으면(=글자 상자=줄 상자) 8px 겹침이 잡혀야 한다"
    );
}

/// font_size=0 이면 걷어낼 근거가 없으므로 상자 그대로 → 겹침이 그대로 검출된다.
#[test]
fn zero_font_size_keeps_the_box() {
    let n = overlap_count(vec![
        line_with_runs(
            10.0,
            100.0,
            200.0,
            28.0,
            vec![run("가", 10.0, 100.0, 200.0, 28.0, 0.0)],
        ),
        line_with_runs(
            10.0,
            120.0,
            200.0,
            28.0,
            vec![run("나", 10.0, 120.0, 200.0, 28.0, 0.0)],
        ),
    ]);
    assert_eq!(n, 1);
}

/// font_size 가 줄높이보다 커도 상자를 넓히지 않는다 — 3px 떨어진 이웃과 겹치지 않는다.
#[test]
fn oversized_font_never_grows_the_box() {
    let n = overlap_count(vec![
        line_with_runs(
            10.0,
            100.0,
            200.0,
            10.0,
            vec![run("가", 10.0, 100.0, 200.0, 10.0, 40.0)],
        ),
        line_with_runs(
            10.0,
            113.0,
            200.0,
            10.0,
            vec![run("나", 10.0, 113.0, 200.0, 10.0, 40.0)],
        ),
    ]);
    assert_eq!(
        n, 0,
        "글자 상자를 넓히는 방향으로 틀리면 없는 겹침을 만든다"
    );
}

/// 줄의 흐름 범위는 런들의 **합집합**이다 — 두 런 사이 틈에 낀 상대도 겹침으로 잡는다.
#[test]
fn flow_extent_is_the_union_across_runs() {
    let n = overlap_count(vec![
        line_with_runs(
            10.0,
            100.0,
            300.0,
            20.0,
            vec![
                run("가", 10.0, 100.0, 50.0, 20.0, 20.0),
                run("나", 200.0, 100.0, 50.0, 20.0, 20.0),
            ],
        ),
        // 두 런 사이 틈(60..200) 한가운데, 같은 세로 대역.
        line_with_runs(
            80.0,
            100.0,
            60.0,
            20.0,
            vec![run("다", 80.0, 100.0, 60.0, 20.0, 20.0)],
        ),
    ]);
    assert_eq!(n, 1, "합집합이 아니라 런별 상자였다면 틈의 상대를 놓친다");
}

/// 표는 글자 상자 축소 없이 자기 bbox 그대로 흐름 범위다.
#[test]
fn table_uses_its_own_bbox() {
    let table = RenderNode::new(
        7,
        RenderNodeType::Table(TableNode {
            row_count: 1,
            col_count: 1,
            border_fill_id: 0,
            section_index: None,
            para_index: None,
            control_index: None,
            cell_context: None,
        }),
        BoundingBox::new(10.0, 104.0, 200.0, 40.0),
    );
    let n = overlap_count(vec![
        line_with_runs(
            10.0,
            100.0,
            200.0,
            28.0,
            vec![run("가", 10.0, 100.0, 200.0, 28.0, 12.0)],
        ),
        table,
    ]);
    assert_eq!(n, 1, "표가 글자-상자 축소를 받았다면 이 겹침은 사라졌을 것");
}

/// 보이는 런이 없는 줄은 애초에 겹침 후보가 아니다 — 후보 조건이 fallback 을 대신한다.
#[test]
fn runless_line_is_not_a_candidate() {
    let n = overlap_count(vec![
        line_with_runs(10.0, 100.0, 200.0, 28.0, vec![]),
        line_with_runs(
            10.0,
            110.0,
            200.0,
            28.0,
            vec![run("가", 10.0, 110.0, 200.0, 28.0, 28.0)],
        ),
    ]);
    assert_eq!(n, 0, "무런 줄이 후보로 새면 유령 겹침이 생긴다");
}
