//! [#6318] `layout-anomaly` 글자 겹침 후보가 본문 밖(바탕쪽·머리말·꼬리말·각주)까지
//! 닿는지 고정한다.
//!
//! 종전 `scan_page` 는 페이지 트리에서 `Body` 하나만 순회했다. 본문 글자가 바탕쪽
//! 사이드바를 덮어도 짝이 애초에 후보가 아니라 신호가 0 이었고, 사람이 렌더 이미지를
//! 봐야만 알 수 있었다(#5952 의 "사이드바와 겹친다", PR #6083 검토 실사례).
//!
//! # 왜 합성 렌더 트리인가 — 실물 문서로 고정하려다 물러섰다
//!
//! 처음에는 편람 69쪽을 렌더해 "본문 x 바탕쪽 겹침 3건" 을 고정했다. 로컬(Windows)에서는
//! 통과했지만 **CI(Linux)에서는 같은 커밋이 0 건**이었다. 원인은 이 판정기가 아니라 그
//! 아래 조판이다 — 같은 문서·같은 커밋에서 첫 표의 높이가 로컬 248.547px 대 CI
//! 352.013px 로 갈리고(+103.467), 그 아래 요소가 통째로 같은 양만큼 밀린다. 그래서
//! 겹치던 짝이 CI 에서는 아예 만나지 않는다.
//!
//! 이 판정기의 계약은 "겹치면 잡는다" 이지 "이 문서 이 쪽이 겹친다" 가 아니다. 조판이
//! 플랫폼별로 갈리는 문제는 별도 축이므로, 규칙 자체는 조판에 의존하지 않는 합성
//! 트리로 고정한다. 실제 코퍼스에서의 건수는 `tests/cases/text_overlap_baseline.rs`
//! 래칫이 945 건 전수로 지킨다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::diagnostics::layout_anomaly::{scan_page, AnomalyOptions};
use rhwp::renderer::render_tree::{
    BoundingBox, PageNode, RectangleNode, RenderNode, RenderNodeType, TextLineNode, TextRunNode,
};

fn text_run(text: &str, x: f64, y: f64, w: f64, h: f64) -> RenderNode {
    RenderNode::new(
        100,
        RenderNodeType::TextRun(TextRunNode {
            text: text.to_string(),
            style: Default::default(),
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
            baseline: h * 0.8,
            field_marker: Default::default(),
            layout_positions: None,
            display_text: None,
        }),
        BoundingBox::new(x, y, w, h),
    )
}

fn text_line(x: f64, y: f64, w: f64, h: f64, runs: Vec<RenderNode>) -> RenderNode {
    let mut n = RenderNode::new(
        99,
        RenderNodeType::TextLine(TextLineNode::new(h, h * 0.8)),
        BoundingBox::new(x, y, w, h),
    );
    n.children = runs;
    n
}

fn body(bbox: BoundingBox, children: Vec<RenderNode>) -> RenderNode {
    let mut n = RenderNode::new(1, RenderNodeType::Body { clip_rect: None }, bbox);
    n.children = children;
    n
}

fn column(index: u16, bbox: BoundingBox, children: Vec<RenderNode>) -> RenderNode {
    let mut n = RenderNode::new(60 + u32::from(index), RenderNodeType::Column(index), bbox);
    n.children = children;
    n
}

fn page(width: f64, height: f64, children: Vec<RenderNode>) -> RenderNode {
    let mut root = RenderNode::new(
        0,
        RenderNodeType::Page(PageNode {
            page_index: 0,
            width,
            height,
            section_index: 0,
        }),
        BoundingBox::new(0.0, 0.0, width, height),
    );
    root.children = children;
    root
}

fn master_page(width: f64, height: f64, children: Vec<RenderNode>) -> RenderNode {
    let mut n = RenderNode::new(
        50,
        RenderNodeType::MasterPage,
        BoundingBox::new(0.0, 0.0, width, height),
    );
    n.children = children;
    n
}

/// 본문 글자가 바탕쪽 글자를 덮으면 잡는다 — 이 변경 전에는 후보조차 아니었다.
///
/// 편람 69쪽의 형상을 그대로 옮긴 것이다: 본문 줄이 오른쪽으로 넘쳐 오른쪽 여백의
/// 바탕쪽 세로 탭 글자와 같은 자리에 놓인다.
#[test]
fn body_text_overlapping_master_page_text_is_flagged() {
    let root = page(
        300.0,
        300.0,
        vec![
            body(
                BoundingBox::new(0.0, 0.0, 200.0, 300.0),
                vec![text_line(
                    10.0,
                    100.0,
                    180.0,
                    12.0,
                    vec![text_run("본문", 10.0, 100.0, 180.0, 12.0)],
                )],
            ),
            // 오른쪽 여백의 바탕쪽 사이드바 — 본문 줄 끝(190)과 x 170..190 에서 겹친다.
            master_page(300.0, 300.0, vec![text_run("탭", 170.0, 100.0, 50.0, 12.0)]),
        ],
    );

    let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
    assert_eq!(
        pa.text_overlap.len(),
        1,
        "본문 글자와 바탕쪽 글자의 겹침이 잡혀야 한다: {:?}",
        pa.text_overlap
    );
    assert!(
        (pa.text_overlap[0].overlap_w - 20.0).abs() < 1e-9,
        "겹침 폭은 20px 여야 한다: {:?}",
        pa.text_overlap[0]
    );
    assert!(pa.has_signal());
}

/// 서로 다른 단의 글자는 종전대로 짝짓지 않는다 — 넓힌 것은 "단 밖" 뿐이다.
///
/// 다단 조판에서 단끼리는 x 축이 나뉘어 있어 정상 조판에서도 나란히 놓인다.
/// 이 규칙이 풀리면 다단 문서 전반이 오탐으로 덮인다.
#[test]
fn different_columns_still_do_not_pair() {
    let make_column = |index: u16, text: &str| {
        column(
            index,
            BoundingBox::new(0.0, 0.0, 100.0, 300.0),
            vec![text_line(
                10.0,
                10.0,
                80.0,
                12.0,
                vec![text_run(text, 10.0, 10.0, 80.0, 12.0)],
            )],
        )
    };
    // 두 단의 글자가 완전히 같은 자리에 있어도 단이 다르면 짝이 아니다.
    let root = page(
        200.0,
        300.0,
        vec![body(
            BoundingBox::new(0.0, 0.0, 200.0, 300.0),
            vec![make_column(0, "가"), make_column(1, "나")],
        )],
    );

    let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
    assert!(
        pa.text_overlap.is_empty(),
        "다른 단의 글자는 종전대로 제외해야 한다: {:?}",
        pa.text_overlap
    );
}

/// 본문 밖에서는 글자만 모은다 — 바탕쪽 배경이 컨테이너 겹침 오탐을 만들지 않는다.
///
/// 바탕쪽은 종이 전체를 덮는 배경을 갖는 일이 흔하다(편람 바탕쪽의
/// `Image x=0..740.8 y=0..1014.4`). 컨테이너 후보(`flow`)로 넣으면 그 배경이 본문
/// 요소 전부와 겹치는 오탐이 된다.
#[test]
fn master_page_background_does_not_create_container_overlap() {
    let background = RenderNode::new(
        51,
        RenderNodeType::Rectangle(RectangleNode::new(0.0, Default::default(), None)),
        BoundingBox::new(0.0, 0.0, 300.0, 300.0),
    );
    let root = page(
        300.0,
        300.0,
        vec![
            body(
                BoundingBox::new(0.0, 0.0, 200.0, 300.0),
                vec![text_line(
                    10.0,
                    10.0,
                    80.0,
                    12.0,
                    vec![text_run("본문", 10.0, 10.0, 80.0, 12.0)],
                )],
            ),
            master_page(300.0, 300.0, vec![background]),
        ],
    );

    let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
    assert!(
        pa.overlap.is_empty(),
        "바탕쪽 배경이 컨테이너 겹침으로 잡히면 안 된다: {:?}",
        pa.overlap
    );
    assert!(
        pa.text_overlap.is_empty(),
        "글자가 아닌 배경은 글자 겹침 후보도 아니다: {:?}",
        pa.text_overlap
    );
}
