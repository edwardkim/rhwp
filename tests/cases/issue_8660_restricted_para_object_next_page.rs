//! #8660: 쪽 영역 안으로 제한(`flowWithText`)한 문단 기준 개체가 본문 아래 끝을 넘으면
//! 한컴은 개체만 다음 쪽 본문 맨 위에 그리고, 기준 문단과 뒤 글은 원래 쪽에 남긴다.
//!
//! 기준: `pdf/issue8660/*-webhwp-print.pdf` (웹한글기안기 인쇄). 보통 줄 30개 뒤 기준 문단
//! `ANCHOR` 에 그림(1908×2067 HU) 또는 사각형을 문단 기준 세로 오프셋으로 붙였다.
//! - 아래 끝이 본문 끝에 딱 맞으면 1쪽(`picture_in_front_fits`)
//! - 1px 넘으면 2쪽, 그림은 2쪽 본문 위 끝(`picture_in_front_over_1px`)
//! - 사각형·자리차지도 같다(`rect_top_and_bottom_over_12px`)
//! - 2쪽에 뒤 글이 이어지면 자리차지 그림 아래에서 글이 시작한다
//!   (`picture_top_and_bottom_over_12px_text_follows`, 2쪽 첫 줄이 그림 높이 20.67pt 만큼 내려감)
//! - 제한을 끄면 같은 쪽에 그대로 둔다(`picture_unrestricted_over_12px`, 1쪽)
//!
//! 수정 전 rhwp 는 그리기 단계에서 개체를 같은 쪽 아래 끝으로 끌어올려 쪽이 하나 모자랐다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::{
    renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType},
    DocumentCore,
};

fn load(name: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue8660")
        .join(format!("{name}.hwp"));
    let bytes = std::fs::read(&path).expect("#8660 재현물");
    DocumentCore::from_bytes(&bytes).unwrap()
}

fn collect<'a>(
    node: &'a RenderNode,
    pick: &dyn Fn(&RenderNode) -> bool,
    out: &mut Vec<&'a RenderNode>,
) {
    if pick(node) {
        out.push(node);
    }
    for child in &node.children {
        collect(child, pick, out);
    }
}

fn boxes(core: &DocumentCore, page: u32, pick: &dyn Fn(&RenderNode) -> bool) -> Vec<BoundingBox> {
    let tree = core.build_page_render_tree(page).unwrap();
    let mut out = Vec::new();
    collect(&tree.root, pick, &mut out);
    out.into_iter().map(|node| node.bbox.clone()).collect()
}

fn body(core: &DocumentCore, page: u32) -> BoundingBox {
    boxes(core, page, &|node| {
        matches!(node.node_type, RenderNodeType::Body { .. })
    })
    .into_iter()
    .next()
    .expect("본문 영역")
}

fn is_image(node: &RenderNode) -> bool {
    matches!(node.node_type, RenderNodeType::Image(_))
}

fn is_rect(node: &RenderNode) -> bool {
    matches!(node.node_type, RenderNodeType::Rectangle(_))
}

fn text_lines(core: &DocumentCore, page: u32) -> Vec<BoundingBox> {
    boxes(core, page, &|node| {
        matches!(node.node_type, RenderNodeType::TextLine(_))
    })
}

fn assert_object_moved_to_next_page_top(name: &str, pick: &dyn Fn(&RenderNode) -> bool) {
    let core = load(name);
    assert_eq!(core.page_count(), 2, "{name}: 웹한글기안기는 2쪽");
    assert!(
        boxes(&core, 0, pick).is_empty(),
        "{name}: 1쪽에는 개체가 없어야 한다"
    );
    let objects = boxes(&core, 1, pick);
    let object = objects.first().expect("2쪽 개체");
    let top = body(&core, 1).y;
    assert!(
        (object.y - top).abs() < 1.0,
        "{name}: 개체는 2쪽 본문 위 끝({top:.2}px)에 놓인다: {:.2}px",
        object.y
    );
    assert!(
        !text_lines(&core, 0).is_empty(),
        "{name}: 기준 문단 글은 1쪽에 남는다"
    );
}

#[test]
fn restricted_picture_that_fits_stays_on_the_anchor_page() {
    let core = load("picture_in_front_fits");
    assert_eq!(core.page_count(), 1, "본문 끝에 딱 맞으면 1쪽");
    assert_eq!(boxes(&core, 0, &is_image).len(), 1);
}

#[test]
fn restricted_picture_over_body_bottom_moves_to_next_page_top() {
    assert_object_moved_to_next_page_top("picture_in_front_over_1px", &is_image);
}

#[test]
fn restricted_top_and_bottom_rect_moves_to_next_page_top() {
    assert_object_moved_to_next_page_top("rect_top_and_bottom_over_12px", &is_rect);
}

#[test]
fn unrestricted_picture_stays_on_the_anchor_page() {
    let core = load("picture_unrestricted_over_12px");
    assert_eq!(core.page_count(), 1, "제한을 끈 그림은 같은 쪽에 남는다");
    assert_eq!(boxes(&core, 0, &is_image).len(), 1);
}

#[test]
fn following_text_starts_below_moved_top_and_bottom_picture() {
    let core = load("picture_top_and_bottom_over_12px_text_follows");
    assert_eq!(core.page_count(), 2);
    let images = boxes(&core, 1, &is_image);
    let image = images.first().expect("2쪽 그림");
    let top = body(&core, 1).y;
    assert!((image.y - top).abs() < 1.0, "그림은 2쪽 본문 위 끝");
    let first_line = text_lines(&core, 1)
        .into_iter()
        .map(|line| line.y)
        .fold(f64::INFINITY, f64::min);
    assert!(
        first_line >= image.y + image.height - 0.5,
        "자리차지 그림 아래에서 2쪽 글이 시작한다: 그림 아래 {:.2}px, 첫 줄 {first_line:.2}px",
        image.y + image.height
    );
}

#[test]
fn restricted_picture_of_table_host_paragraph_moves_to_next_page_top() {
    // 기준 문단이 글자처럼 표를 품어도 같다. 표는 1쪽에 남고 그림만 2쪽 맨 위로 간다.
    assert_object_moved_to_next_page_top("picture_in_front_with_table_host_over_12px", &is_image);
    let core = load("picture_in_front_with_table_host_over_12px");
    assert!(
        !boxes(&core, 0, &|node| matches!(
            node.node_type,
            RenderNodeType::Table(_)
        ))
        .is_empty(),
        "기준 문단의 표는 1쪽에 남는다"
    );
}

// ── 넘기지 않아야 하는 경우와 앞에 표가 있는 경우(가드) ──
// 보통 줄 20개 뒤 글자처럼 표(높이 12000 HU) 문단, 그 뒤 기준 문단에 같은 그림을 붙였다.
// 웹한글기안기: 그림 아래 끝이 본문 끝보다 6px 위면 1쪽에 그대로, 6px 넘으면 2쪽 맨 위.
// 앞에 표가 있어도 판정 기준은 같다(표 높이까지 더한 기준 문단 위치).

#[test]
fn restricted_picture_after_table_that_fits_stays_on_the_anchor_page() {
    let core = load("picture_in_front_after_table_fits_6px");
    assert_eq!(core.page_count(), 1, "본문 끝 6px 위에서 끝나면 1쪽");
    let images = boxes(&core, 0, &is_image);
    let image = images.first().expect("1쪽 그림");
    let body = body(&core, 0);
    assert!(
        image.y + image.height <= body.y + body.height + 0.5,
        "그림은 1쪽 본문 안에 그대로 있다: 그림 아래 {:.2}px, 본문 아래 {:.2}px",
        image.y + image.height,
        body.y + body.height
    );
}

#[test]
fn restricted_picture_after_table_over_body_bottom_moves_to_next_page_top() {
    assert_object_moved_to_next_page_top("picture_in_front_after_table_over_6px", &is_image);
}
